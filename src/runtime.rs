use chrono::Utc;
use serde_json::{json, Value};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::adapters::{self, ActionRunRequest};
use crate::consumer;
use crate::errors;
use crate::manifest_access::ManifestView;
use crate::permissions;
use crate::readiness;
use crate::run_record::{self, RunRecordInput};
use crate::schemas;

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TestOptions {
    pub cwd: PathBuf,
}

#[derive(Debug)]
pub struct RunOptions {
    pub cwd: PathBuf,
    pub input: PathBuf,
}

struct RuntimeManifest {
    value: YamlValue,
    path: PathBuf,
    sha256: String,
}

struct RunPaths {
    run_dir: PathBuf,
    input_json: PathBuf,
    context_json: PathBuf,
    output_json: PathBuf,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
    artifact_dir: PathBuf,
    record_json: PathBuf,
}

pub struct RunOutcome {
    pub envelope: String,
    pub success: bool,
}

pub fn run_test(options: &TestOptions) -> Result<RunOutcome, String> {
    let input = PathBuf::from("examples").join("default.input.json");
    execute(&options.cwd, &input, "test")
}

pub fn run_with_input(options: &RunOptions) -> Result<RunOutcome, String> {
    execute(&options.cwd, &options.input, "run")
}

pub fn run_with_json_input(
    cwd: &Path,
    input_value: Value,
    mode: &str,
) -> Result<RunOutcome, String> {
    let capsule_dir = absolute_path(cwd)?;
    require_dir(&capsule_dir)?;
    execute_value(&capsule_dir, input_value, mode)
}

fn execute(cwd: &Path, input: &Path, mode: &str) -> Result<RunOutcome, String> {
    let capsule_dir = absolute_path(cwd)?;
    require_dir(&capsule_dir)?;
    let input_source = resolve_input_path(&capsule_dir, input);
    let input_value = read_input_json(&input_source)?;
    execute_value(&capsule_dir, input_value, mode)
}

fn execute_value(capsule_dir: &Path, input_value: Value, mode: &str) -> Result<RunOutcome, String> {
    let manifest = load_manifest(capsule_dir, mode)?;
    let manifest_view = ManifestView::new(&manifest.value);
    let adapter = manifest_view
        .runtime_adapter()
        .ok_or_else(|| "invalid Manifest: missing runtime.adapter".to_string())?;
    adapters::ensure_runtime_adapter(adapter)?;

    let entrypoint = manifest_view
        .runtime_entrypoint()
        .ok_or_else(|| "invalid Manifest: missing runtime.entrypoint".to_string())?;
    let command = manifest_view.runtime_command()?;
    let timeout = manifest_view
        .runtime_timeout()
        .and_then(parse_timeout)
        .unwrap_or_else(|| Duration::from_secs(30));
    let run_id = new_run_id();
    let paths = create_run_paths(capsule_dir, &run_id)?;
    let permissions = manifest_view.permissions_json().unwrap_or(Value::Null);
    let declared_env = permissions::declared_env_values(&manifest.value);
    let started_at = Utc::now();
    let started = Instant::now();

    write_json(&paths.input_json, &input_value)?;
    let context = json!({
        "run_id": run_id,
        "mode": mode,
        "capsule_dir": capsule_dir.display().to_string(),
        "manifest_path": manifest.path.display().to_string(),
        "artifact_dir": paths.artifact_dir.display().to_string(),
        "permissions": permissions,
    });
    write_json(&paths.context_json, &context)?;

    if let Some(input_schema) = manifest_view.input_schema_json() {
        if let Err(error) = schemas::validate_value(&input_schema, &input_value) {
            write_empty_logs(&paths)?;
            return finish_run(
                FinishRunInput {
                    run_id: &run_id,
                    mode,
                    success: false,
                    started_at,
                    duration_started_at: started,
                    capsule_dir,
                    manifest: &manifest,
                    paths: &paths,
                },
                errors::validation_error(format!("input schema validation failed: {error}")),
            );
        }
    }

    if let Some(message) = dependency_failure_message(capsule_dir)? {
        fs::write(&paths.stdout_log, []).map_err(|write_error| {
            format!(
                "failed to write {}: {write_error}",
                paths.stdout_log.display()
            )
        })?;
        fs::write(&paths.stderr_log, message.as_bytes()).map_err(|write_error| {
            format!(
                "failed to write {}: {write_error}",
                paths.stderr_log.display()
            )
        })?;
        return finish_run(
            FinishRunInput {
                run_id: &run_id,
                mode,
                success: false,
                started_at,
                duration_started_at: started,
                capsule_dir,
                manifest: &manifest,
                paths: &paths,
            },
            errors::dependency_error(message),
        );
    }

    let adapter_output = match adapters::run_action(
        adapter,
        &ActionRunRequest {
            capsule_dir,
            entrypoint,
            command: command.as_deref(),
            context_json: &paths.context_json,
            input_json: &paths.input_json,
            output_json: &paths.output_json,
            artifact_dir: &paths.artifact_dir,
            env: &declared_env,
            timeout,
        },
    ) {
        Ok(output) => output,
        Err(error) => {
            fs::write(&paths.stdout_log, []).map_err(|write_error| {
                format!(
                    "failed to write {}: {write_error}",
                    paths.stdout_log.display()
                )
            })?;
            fs::write(&paths.stderr_log, error.as_bytes()).map_err(|write_error| {
                format!(
                    "failed to write {}: {write_error}",
                    paths.stderr_log.display()
                )
            })?;
            return finish_run(
                FinishRunInput {
                    run_id: &run_id,
                    mode,
                    success: false,
                    started_at,
                    duration_started_at: started,
                    capsule_dir,
                    manifest: &manifest,
                    paths: &paths,
                },
                errors::runtime_error(error),
            );
        }
    };
    fs::write(&paths.stdout_log, adapter_output.stdout)
        .map_err(|error| format!("failed to write {}: {error}", paths.stdout_log.display()))?;
    fs::write(&paths.stderr_log, adapter_output.stderr)
        .map_err(|error| format!("failed to write {}: {error}", paths.stderr_log.display()))?;

    let (envelope, success) = adapter_envelope(&paths, adapter_output.success, &manifest.value);
    finish_run(
        FinishRunInput {
            run_id: &run_id,
            mode,
            success,
            started_at,
            duration_started_at: started,
            capsule_dir,
            manifest: &manifest,
            paths: &paths,
        },
        envelope,
    )
}

fn write_empty_logs(paths: &RunPaths) -> Result<(), String> {
    fs::write(&paths.stdout_log, [])
        .map_err(|error| format!("failed to write {}: {error}", paths.stdout_log.display()))?;
    fs::write(&paths.stderr_log, [])
        .map_err(|error| format!("failed to write {}: {error}", paths.stderr_log.display()))
}

fn dependency_failure_message(capsule_dir: &Path) -> Result<Option<String>, String> {
    let report = readiness::evaluate(capsule_dir)?;
    let failures = report
        .dependency_checks
        .iter()
        .filter(|check| check.status != "satisfied")
        .map(|check| {
            format!(
                "{} {} required {} detected {} status {}",
                check.kind,
                check.name,
                check.required,
                check.detected.as_deref().unwrap_or("missing"),
                check.status
            )
        })
        .collect::<Vec<_>>();

    if failures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "Capsule runtime does not satisfy declared runtime requirements: {}.",
            failures.join("; ")
        )))
    }
}

struct FinishRunInput<'a> {
    run_id: &'a str,
    mode: &'a str,
    success: bool,
    started_at: chrono::DateTime<Utc>,
    duration_started_at: Instant,
    capsule_dir: &'a Path,
    manifest: &'a RuntimeManifest,
    paths: &'a RunPaths,
}

fn finish_run(input: FinishRunInput<'_>, mut envelope: Value) -> Result<RunOutcome, String> {
    let Some(object) = envelope.as_object_mut() else {
        envelope = errors::protocol_violation("adapter envelope must be a JSON object");
        return finish_run(
            FinishRunInput {
                success: false,
                ..input
            },
            envelope,
        );
    };
    object.insert(
        "run_id".to_string(),
        Value::String(input.run_id.to_string()),
    );
    object.insert(
        "run_dir".to_string(),
        Value::String(input.paths.run_dir.display().to_string()),
    );
    object.insert(
        "record".to_string(),
        Value::String(input.paths.record_json.display().to_string()),
    );
    write_json(&input.paths.output_json, &envelope)?;

    let finished_at = Utc::now();
    let manifest_view = ManifestView::new(&input.manifest.value);
    let skill_sha256 = manifest_view.skill_sha256().unwrap_or("missing");
    let action_sha256 = manifest_view.action_sha256().unwrap_or("missing");
    run_record::write(
        &input.paths.record_json,
        RunRecordInput {
            run_id: input.run_id,
            mode: input.mode,
            status: if input.success { "succeeded" } else { "failed" },
            started_at: input.started_at,
            finished_at,
            duration: input.duration_started_at.elapsed(),
            capsule_dir: input.capsule_dir,
            manifest_path: &input.manifest.path,
            manifest_sha256: &input.manifest.sha256,
            skill_sha256,
            action_sha256,
            permissions: manifest_view.permissions_json().unwrap_or(Value::Null),
        },
    )?;

    let envelope = serde_json::to_string_pretty(&envelope)
        .map_err(|error| format!("failed to serialize output envelope: {error}"))?;
    Ok(RunOutcome {
        envelope,
        success: input.success,
    })
}

fn adapter_envelope(
    paths: &RunPaths,
    adapter_success: bool,
    manifest: &YamlValue,
) -> (Value, bool) {
    if !paths.output_json.is_file() {
        return (
            errors::protocol_violation(format!(
                "adapter did not write output envelope: {}",
                paths.output_json.display()
            )),
            false,
        );
    }

    let envelope = match read_json_file(&paths.output_json) {
        Ok(envelope) => envelope,
        Err(error) => return (errors::protocol_violation(error), false),
    };

    match envelope.get("ok").and_then(Value::as_bool) {
        Some(true) if adapter_success => {
            let manifest_view = ManifestView::new(manifest);
            if let Some(output_schema) = manifest_view.output_schema_json() {
                let output = envelope.get("output").unwrap_or(&Value::Null);
                if let Err(error) = schemas::validate_value(&output_schema, output) {
                    return (
                        errors::protocol_violation(format!(
                            "output schema validation failed: {error}"
                        )),
                        false,
                    );
                }
            }
            match permissions::validate_artifacts(&envelope, &paths.artifact_dir) {
                Ok(()) => (envelope, true),
                Err(error) => (errors::permission_denied(error), false),
            }
        }
        Some(true) => (
            errors::protocol_violation("adapter exited with failure after writing ok: true"),
            false,
        ),
        Some(false) => match errors::validate_error_envelope(&envelope) {
            Ok(()) => (envelope, false),
            Err(error) => (errors::protocol_violation(error), false),
        },
        None => (
            errors::protocol_violation("adapter output envelope is missing ok"),
            false,
        ),
    }
}

fn require_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("cwd does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("cwd is not a directory: {}", path.display()));
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| format!("failed to resolve current directory: {error}"))
    }
}

fn load_manifest(cwd: &Path, mode: &str) -> Result<RuntimeManifest, String> {
    let command = match mode {
        "test" => "skillrun test",
        "mcp" => "skillrun serve --mcp",
        _ => "skillrun run",
    };
    let valid = consumer::validate(cwd, command)?;
    Ok(RuntimeManifest {
        value: valid.value,
        path: valid.path,
        sha256: valid.sha256,
    })
}

fn create_run_paths(cwd: &Path, run_id: &str) -> Result<RunPaths, String> {
    let run_dir = cwd.join(".skillrun").join("runs").join(run_id);
    let artifact_dir = run_dir.join("artifacts");
    fs::create_dir_all(&artifact_dir).map_err(|error| {
        format!(
            "failed to create run directories at {}: {error}",
            run_dir.display()
        )
    })?;
    Ok(RunPaths {
        input_json: run_dir.join("input.json"),
        context_json: run_dir.join("context.json"),
        output_json: run_dir.join("output.json"),
        stdout_log: run_dir.join("stdout.log"),
        stderr_log: run_dir.join("stderr.log"),
        record_json: run_dir.join("record.json"),
        artifact_dir,
        run_dir,
    })
}

fn resolve_input_path(cwd: &Path, input: &Path) -> PathBuf {
    if input.is_absolute() {
        input.to_path_buf()
    } else {
        cwd.join(input)
    }
}

fn read_input_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read input {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("input was not valid JSON: {error}"))
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("{} was not valid JSON: {error}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize JSON for {}: {error}", path.display()))?;
    fs::write(path, text).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn new_run_id() -> String {
    let counter = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "run-{}-{}-{counter}",
        Utc::now().format("%Y%m%dT%H%M%S%.fZ"),
        std::process::id()
    )
}

fn parse_timeout(value: &str) -> Option<Duration> {
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.parse::<u64>().ok().map(Duration::from_millis);
    }
    if let Some(seconds) = value.strip_suffix('s') {
        return seconds.parse::<u64>().ok().map(Duration::from_secs);
    }
    value.parse::<u64>().ok().map(Duration::from_secs)
}
