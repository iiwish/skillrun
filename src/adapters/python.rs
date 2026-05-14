use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::adapters::{ActionRunOutput, ActionRunRequest, DiscoveredDependency, RuntimeDiscovery};
use crate::schemas::Schemas;

pub fn discover_runtime() -> RuntimeDiscovery {
    let executable = discover_python();
    let packages = if executable.available {
        vec![discover_pydantic()]
    } else {
        vec![DiscoveredDependency {
            name: "pydantic".to_string(),
            detected: None,
            available: false,
        }]
    };

    RuntimeDiscovery {
        executable,
        packages,
    }
}

fn discover_python() -> DiscoveredDependency {
    let output = run_python_probe(&["--version"]);
    match output {
        Ok(output) if output.status.success() => {
            let detected = command_text(&output);
            DiscoveredDependency {
                name: "python".to_string(),
                detected: non_empty(detected),
                available: true,
            }
        }
        _ => DiscoveredDependency {
            name: "python".to_string(),
            detected: None,
            available: false,
        },
    }
}

fn discover_pydantic() -> DiscoveredDependency {
    let output = run_python_probe(&["-c", "import pydantic; print(pydantic.__version__)"]);
    match output {
        Ok(output) if output.status.success() => {
            let detected = command_text(&output);
            DiscoveredDependency {
                name: "pydantic".to_string(),
                detected: non_empty(detected),
                available: true,
            }
        }
        _ => DiscoveredDependency {
            name: "pydantic".to_string(),
            detected: None,
            available: false,
        },
    }
}

fn run_python_probe(args: &[&str]) -> Result<Output, std::io::Error> {
    let mut last_error = None;
    for executable in python_probe_commands() {
        let output = Command::new(executable).args(args).output();
        match output {
            Ok(output) => return Ok(output),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "python executable not found")
    }))
}

fn python_probe_commands() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &["python", "python.cmd"]
    }
    #[cfg(not(windows))]
    {
        &["python"]
    }
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
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_process_env(&mut command);

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
from pydantic import ValidationError as PydanticValidationError

context_path = Path(os.environ["SKILLRUN_CONTEXT_JSON"])
input_path = Path(os.environ["SKILLRUN_INPUT_JSON"])
output_path = Path(os.environ["SKILLRUN_OUTPUT_JSON"])

def write_error(code, message, recoverable=False, llm_hint=None):
    error = {
        "code": code,
        "message": message,
        "recoverable": recoverable
    }
    if llm_hint:
        error["llm_hint"] = llm_hint
    output_path.write_text(json.dumps({
        "ok": False,
        "error": error,
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
    context = json.loads(context_path.read_text(encoding="utf-8"))

    try:
        input_model = Input.model_validate_json(input_path.read_text(encoding="utf-8"))
    except PydanticValidationError as exc:
        write_error("ValidationError", str(exc), True)
        sys.exit(1)

    preflight = getattr(module, "preflight", None)
    if preflight is not None:
        try:
            preflight(input_model, context)
        except ValueError as exc:
            write_error(
                "PolicyViolation",
                str(exc),
                True,
                "Ask for the missing approval or policy context before retrying."
            )
            sys.exit(1)

    try:
        result = module.run(input_model, context)
    except ValueError as exc:
        write_error(
            "PolicyViolation",
            str(exc),
            True,
            "Ask for the missing approval or policy context before retrying."
        )
        sys.exit(1)
    except Exception as exc:
        traceback.print_exc(file=sys.stderr)
        write_error("RuntimeError", str(exc), False)
        sys.exit(1)

    artifacts = []
    display_markdown = None
    try:
        if isinstance(result, Output):
            output_model = result
        elif isinstance(result, dict) and (
            "output" in result or "result" in result or "artifacts" in result or "display" in result
        ):
            output_value = result.get("output", result.get("result"))
            if output_value is None:
                raise ValueError("action result envelope must contain output")
            artifacts = result.get("artifacts", [])
            display = result.get("display")
            if isinstance(display, dict):
                display_markdown = display.get("markdown")
            elif isinstance(display, str):
                display_markdown = display
            output_model = Output.model_validate(output_value)
        else:
            output_model = Output.model_validate(result)
    except (PydanticValidationError, ValueError) as exc:
        write_error("ProtocolViolation", str(exc), False)
        sys.exit(1)

    payload = output_model.model_dump(mode="json")
    if display_markdown is None:
        display_markdown = payload.get("reasoning_summary", "Run completed.")
    output_path.write_text(json.dumps({
        "ok": True,
        "output": payload,
        "artifacts": artifacts,
        "display": {
            "markdown": display_markdown
        }
    }, ensure_ascii=False, indent=2), encoding="utf-8")
except Exception as exc:
    traceback.print_exc(file=sys.stderr)
    write_error("RuntimeError", str(exc), False)
    sys.exit(1)
"#;

    let mut command = Command::new("python");
    command
        .arg("-c")
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
            "failed to run Python action adapter for {}: {error}",
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

fn command_text(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command.spawn().map_err(|error| {
        format!(
            "failed to spawn Python metadata extractor: {}",
            spawn_error_text(&error)
        )
    })?;
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

fn spawn_error_text(error: &std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        "program not found".to_string()
    } else {
        error.to_string()
    }
}
