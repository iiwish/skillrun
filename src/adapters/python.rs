use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::schemas::Schemas;

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
