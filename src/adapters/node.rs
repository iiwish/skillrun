use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::adapters::{ActionRunOutput, ActionRunRequest};
use crate::schemas::Schemas;

pub fn extract_schemas(capsule_dir: &Path, action_path: &Path) -> Result<Schemas, String> {
    let action_path = action_path
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", action_path.display()))?;
    let script = r#"
import { pathToFileURL } from "node:url";

const actionPath = process.argv[1];
const action = await import(pathToFileURL(actionPath).href);

function schemaFor(name) {
  if (!Object.prototype.hasOwnProperty.call(action, name)) {
    throw new Error(`missing ${name} export`);
  }

  const schema = action[name];
  if (schema === null || typeof schema !== "object" || Array.isArray(schema)) {
    throw new Error(`${name} must export a JSON Schema object`);
  }

  return schema;
}

process.stdout.write(JSON.stringify({
  input: schemaFor("inputSchema"),
  output: schemaFor("outputSchema")
}));
"#;

    let mut command = Command::new("node");
    command
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .arg(&action_path)
        .current_dir(capsule_dir);
    apply_process_env(&mut command);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let timeout = metadata_timeout();
    let output = run_with_timeout(command, timeout).map_err(|error| {
        format!(
            "failed to run Node metadata extractor for {}: {error}",
            action_path.display()
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "node metadata extraction failed for {}: {}",
            action_path.display(),
            stderr.trim()
        ));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("node metadata output was not valid JSON: {error}"))
}

pub fn run_action(request: &ActionRunRequest<'_>) -> Result<ActionRunOutput, String> {
    let action_path = request.capsule_dir.join(request.entrypoint);
    let action_path = action_path
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", action_path.display()))?;
    let script = r#"
import fs from "node:fs";
import { pathToFileURL } from "node:url";

const actionPath = process.argv[1];
const contextPath = process.env.SKILLRUN_CONTEXT_JSON;
const inputPath = process.env.SKILLRUN_INPUT_JSON;
const outputPath = process.env.SKILLRUN_OUTPUT_JSON;

function writeEnvelope(envelope) {
  fs.writeFileSync(outputPath, JSON.stringify(envelope, null, 2), "utf8");
}

function writeError(code, message, recoverable = false, llmHint = undefined) {
  const error = { code, message, recoverable };
  if (llmHint) {
    error.llm_hint = llmHint;
  }
  writeEnvelope({
    ok: false,
    error,
    display: { markdown: message }
  });
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function jsonTypeOf(value) {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  if (Number.isInteger(value)) return "integer";
  return typeof value;
}

function typeMatches(value, expected) {
  if (expected === "number") return typeof value === "number" && Number.isFinite(value);
  if (expected === "integer") return Number.isInteger(value);
  if (expected === "object") return isPlainObject(value);
  if (expected === "array") return Array.isArray(value);
  if (expected === "null") return value === null;
  return typeof value === expected;
}

function validate(schema, value, path = "$") {
  const errors = [];
  if (!isPlainObject(schema)) {
    return errors;
  }

  const expectedType = schema.type;
  if (expectedType !== undefined) {
    const allowed = Array.isArray(expectedType) ? expectedType : [expectedType];
    if (!allowed.some((item) => typeMatches(value, item))) {
      errors.push(`${path}: expected ${allowed.join(" or ")}, got ${jsonTypeOf(value)}`);
      return errors;
    }
  }

  if (schema.enum && Array.isArray(schema.enum)) {
    const encoded = JSON.stringify(value);
    if (!schema.enum.some((item) => JSON.stringify(item) === encoded)) {
      errors.push(`${path}: value must be one of ${schema.enum.map((item) => JSON.stringify(item)).join(", ")}`);
    }
  }

  if (typeof value === "string" && typeof schema.minLength === "number" && value.length < schema.minLength) {
    errors.push(`${path}: string length must be at least ${schema.minLength}`);
  }

  if (typeof value === "number" && typeof schema.minimum === "number" && value < schema.minimum) {
    errors.push(`${path}: number must be at least ${schema.minimum}`);
  }

  if (isPlainObject(value)) {
    const required = Array.isArray(schema.required) ? schema.required : [];
    for (const key of required) {
      if (!Object.prototype.hasOwnProperty.call(value, key)) {
        errors.push(`${path}.${key}: required property is missing`);
      }
    }

    const properties = isPlainObject(schema.properties) ? schema.properties : {};
    for (const [key, childSchema] of Object.entries(properties)) {
      if (Object.prototype.hasOwnProperty.call(value, key)) {
        errors.push(...validate(childSchema, value[key], `${path}.${key}`));
      }
    }

    if (schema.additionalProperties === false) {
      for (const key of Object.keys(value)) {
        if (!Object.prototype.hasOwnProperty.call(properties, key)) {
          errors.push(`${path}.${key}: additional properties are not allowed`);
        }
      }
    }
  }

  return errors;
}

function policyHint(error) {
  return error?.llm_hint || "Ask for the missing approval or policy context before retrying.";
}

function policyMessage(error) {
  return error?.message || String(error);
}

function isPolicyError(error) {
  return error?.code === "PolicyViolation" || error?.name === "PolicyViolation" || error?.recoverable === true;
}

try {
  const action = await import(pathToFileURL(actionPath).href);
  const inputSchema = action.inputSchema;
  const outputSchema = action.outputSchema;
  const context = JSON.parse(fs.readFileSync(contextPath, "utf8"));
  const input = JSON.parse(fs.readFileSync(inputPath, "utf8"));

  const inputErrors = validate(inputSchema, input);
  if (inputErrors.length > 0) {
    writeError("ValidationError", inputErrors.join("; "), true);
    process.exit(1);
  }

  if (typeof action.preflight === "function") {
    try {
      await action.preflight(input, context);
    } catch (error) {
      writeError("PolicyViolation", policyMessage(error), true, policyHint(error));
      process.exit(1);
    }
  }

  if (typeof action.run !== "function") {
    writeError("ProtocolViolation", "action.mjs must export a run function", false);
    process.exit(1);
  }

  let result;
  try {
    result = await action.run(input, context);
  } catch (error) {
    if (isPolicyError(error)) {
      writeError("PolicyViolation", policyMessage(error), true, policyHint(error));
      process.exit(1);
    }
    console.error(error?.stack || String(error));
    writeError("RuntimeError", policyMessage(error), false);
    process.exit(1);
  }

  let outputValue = result;
  let artifacts = [];
  let displayMarkdown = undefined;
  if (
    isPlainObject(result) &&
    ("output" in result || "result" in result || "artifacts" in result || "display" in result)
  ) {
    outputValue = result.output ?? result.result;
    artifacts = result.artifacts ?? [];
    if (isPlainObject(result.display)) {
      displayMarkdown = result.display.markdown;
    } else if (typeof result.display === "string") {
      displayMarkdown = result.display;
    }
  }

  const outputErrors = validate(outputSchema, outputValue);
  if (outputValue === undefined) {
    outputErrors.unshift("action result envelope must contain output");
  }
  if (outputErrors.length > 0) {
    writeError("ProtocolViolation", outputErrors.join("; "), false);
    process.exit(1);
  }

  if (displayMarkdown === undefined) {
    displayMarkdown = outputValue.reasoning_summary || "Run completed.";
  }
  writeEnvelope({
    ok: true,
    output: outputValue,
    artifacts,
    display: { markdown: displayMarkdown }
  });
} catch (error) {
  console.error(error?.stack || String(error));
  writeError("RuntimeError", policyMessage(error), false);
  process.exit(1);
}
"#;

    let mut command = Command::new("node");
    command
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .arg(&action_path)
        .current_dir(request.capsule_dir);
    apply_process_env(&mut command);
    for (key, value) in request.env {
        command.env(key, value);
    }
    command
        .env("SKILLRUN_CONTEXT_JSON", request.context_json)
        .env("SKILLRUN_INPUT_JSON", request.input_json)
        .env("SKILLRUN_OUTPUT_JSON", request.output_json)
        .env("SKILLRUN_ARTIFACT_DIR", request.artifact_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = run_with_timeout(command, request.timeout).map_err(|error| {
        format!(
            "failed to run Node action adapter for {}: {error}",
            action_path.display()
        )
    })?;

    Ok(ActionRunOutput {
        success: output.status.success(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn metadata_timeout() -> Duration {
    std::env::var("SKILLRUN_METADATA_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(10))
}

fn apply_process_env(command: &mut Command) {
    command.env_clear();
    for key in [
        "SystemRoot",
        "WINDIR",
        "COMSPEC",
        "TEMP",
        "TMP",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "PATH",
    ] {
        if let Ok(value) = std::env::var(key) {
            command.env(key, value);
        }
    }
}

fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to spawn Node metadata extractor: {error}"))?;
    let started_at = Instant::now();

    loop {
        if child
            .try_wait()
            .map_err(|error| format!("failed to poll Node metadata extractor: {error}"))?
            .is_some()
        {
            return child
                .wait_with_output()
                .map_err(|error| format!("failed to collect Node metadata output: {error}"));
        }

        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait_with_output();
            return Err(format!(
                "metadata extraction timed out after {} ms",
                timeout.as_millis()
            ));
        }

        thread::sleep(Duration::from_millis(10));
    }
}
