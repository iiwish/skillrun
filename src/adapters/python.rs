use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::schemas::Schemas;

pub struct ActionRunRequest<'a> {
    pub capsule_dir: &'a Path,
    pub entrypoint: &'a str,
    pub context_json: &'a Path,
    pub input_json: &'a Path,
    pub output_json: &'a Path,
    pub artifact_dir: &'a Path,
    pub timeout: Duration,
}

pub struct ActionRunOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn extract_schemas(capsule_dir: &Path, action_path: &Path) -> Result<Schemas, String> {
    let action_path = action_path
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", action_path.display()))?;
    let script = r#"
import importlib.util
import json
import sys
from pathlib import Path

action = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("skillrun_action", action)
if spec is None or spec.loader is None:
    raise RuntimeError(f"cannot load action module: {action}")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

def schema_for(name):
    model = getattr(module, name, None)
    if model is None:
        raise RuntimeError(f"missing {name} model")
    method = getattr(model, "model_json_schema", None)
    if method is None:
        raise RuntimeError(f"{name} must be a Pydantic v2 model")
    return method()

print(json.dumps({
    "input": schema_for("Input"),
    "output": schema_for("Output"),
}, ensure_ascii=False))
"#;

    let mut command = Command::new("python");
    command
        .arg("-c")
        .arg(script)
        .arg(&action_path)
        .current_dir(capsule_dir)
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let timeout = metadata_timeout();
    let output = run_with_timeout(command, timeout).map_err(|error| {
        format!(
            "failed to run Python metadata extractor for {}: {error}",
            action_path.display()
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "python metadata extraction failed for {}: {}",
            action_path.display(),
            stderr.trim()
        ));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("python metadata output was not valid JSON: {error}"))
}

pub fn run_action(request: &ActionRunRequest<'_>) -> Result<ActionRunOutput, String> {
    let action_path = request.capsule_dir.join(request.entrypoint);
    let action_path = action_path
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", action_path.display()))?;
    let script = r#"
import importlib.util
import json
import os
import sys
import traceback
from pathlib import Path

context_path = Path(os.environ["SKILLRUN_CONTEXT_JSON"])
input_path = Path(os.environ["SKILLRUN_INPUT_JSON"])
output_path = Path(os.environ["SKILLRUN_OUTPUT_JSON"])

def write_error(code, message):
    output_path.write_text(json.dumps({
        "ok": False,
        "error": {
            "code": code,
            "message": message,
            "recoverable": False
        },
        "display": {
            "markdown": message
        }
    }, ensure_ascii=False, indent=2), encoding="utf-8")

try:
    action = Path(sys.argv[1])
    spec = importlib.util.spec_from_file_location("skillrun_action", action)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load action module: {action}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    Input = getattr(module, "Input")
    Output = getattr(module, "Output")
    input_model = Input.model_validate_json(input_path.read_text(encoding="utf-8"))
    context = json.loads(context_path.read_text(encoding="utf-8"))

    preflight = getattr(module, "preflight", None)
    if preflight is not None:
        preflight(input_model, context)

    result = module.run(input_model, context)
    if isinstance(result, Output):
        output_model = result
    else:
        output_model = Output.model_validate(result)

    payload = output_model.model_dump(mode="json")
    output_path.write_text(json.dumps({
        "ok": True,
        "output": payload,
        "display": {
            "markdown": payload.get("reasoning_summary", "Run completed.")
        }
    }, ensure_ascii=False, indent=2), encoding="utf-8")
except Exception as exc:
    traceback.print_exc(file=sys.stderr)
    write_error("RuntimeError", str(exc))
    sys.exit(1)
"#;

    let mut command = Command::new("python");
    command
        .arg("-c")
        .arg(script)
        .arg(&action_path)
        .current_dir(request.capsule_dir)
        .env_clear()
        .env("SKILLRUN_CONTEXT_JSON", request.context_json)
        .env("SKILLRUN_INPUT_JSON", request.input_json)
        .env("SKILLRUN_OUTPUT_JSON", request.output_json)
        .env("SKILLRUN_ARTIFACT_DIR", request.artifact_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = run_with_timeout(command, request.timeout).map_err(|error| {
        format!(
            "failed to run Python action adapter for {}: {error}",
            action_path.display()
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "python action adapter failed for {}: {}",
            action_path.display(),
            stderr.trim()
        ));
    }

    Ok(ActionRunOutput {
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

fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to spawn Python metadata extractor: {error}"))?;
    let started_at = Instant::now();

    loop {
        if child
            .try_wait()
            .map_err(|error| format!("failed to poll Python metadata extractor: {error}"))?
            .is_some()
        {
            return child
                .wait_with_output()
                .map_err(|error| format!("failed to collect Python metadata output: {error}"));
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
