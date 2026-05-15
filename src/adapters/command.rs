use std::process::{Command, Stdio};

use crate::adapters::process::{self, TimeoutMessages};
use crate::adapters::{ActionRunOutput, ActionRunRequest};

pub fn run_action(request: &ActionRunRequest<'_>) -> Result<ActionRunOutput, String> {
    let command = request
        .command
        .ok_or_else(|| "runtime.command is required for command adapter".to_string())?;
    let (program, args) = command
        .split_first()
        .ok_or_else(|| "runtime.command must not be empty".to_string())?;

    let mut child = Command::new(program);
    child.args(args).current_dir(request.capsule_dir);
    apply_process_env(&mut child);
    for (key, value) in request.env {
        child.env(key, value);
    }
    child
        .env("SKILLRUN_CONTEXT_JSON", request.context_json)
        .env("SKILLRUN_INPUT_JSON", request.input_json)
        .env("SKILLRUN_OUTPUT_JSON", request.output_json)
        .env("SKILLRUN_ARTIFACT_DIR", request.artifact_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = process::run_with_timeout(
        child,
        request.timeout,
        TimeoutMessages {
            spawn: "failed to spawn command adapter",
            poll: "failed to poll command adapter",
            collect: "failed to collect command adapter output",
            timeout: "command adapter timed out",
        },
    )
    .map_err(|error| format!("failed to run command adapter {program}: {error}"))?;

    Ok(ActionRunOutput {
        success: output.status.success(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
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
