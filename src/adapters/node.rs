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
