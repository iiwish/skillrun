use chrono::Utc;
use serde_json::{json, Value};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::adapters::python::{self, ActionRunRequest};
use crate::hashing;
use crate::manifest;
use crate::run_record::{self, RunRecordInput};

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

pub fn run_test(options: &TestOptions) -> Result<String, String> {
    let input = PathBuf::from("examples").join("default.input.json");
    execute(&options.cwd, &input, "test")
}

pub fn run_with_input(options: &RunOptions) -> Result<String, String> {
    execute(&options.cwd, &options.input, "run")
}

fn execute(cwd: &Path, input: &Path, mode: &str) -> Result<String, String> {
    let capsule_dir = absolute_path(cwd)?;
    require_dir(&capsule_dir)?;
    let manifest = load_manifest(&capsule_dir)?;
    let adapter = string_at(&manifest.value, &["runtime", "adapter"]).unwrap_or("python");
    if adapter != "python" {
        return Err(format!("unsupported runtime adapter: {adapter}"));
    }

    let entrypoint = string_at(&manifest.value, &["runtime", "entrypoint"]).unwrap_or("action.py");
    let timeout = string_at(&manifest.value, &["runtime", "timeout"])
        .and_then(parse_timeout)
        .unwrap_or_else(|| Duration::from_secs(30));
    let run_id = new_run_id();
    let paths = create_run_paths(&capsule_dir, &run_id)?;
    let input_source = resolve_input_path(&capsule_dir, input);
    let input_value = read_input_json(&input_source)?;
    let permissions = json_value_at(&manifest.value, &["permissions"]).unwrap_or(Value::Null);
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

    let adapter_output = python::run_action(&ActionRunRequest {
        capsule_dir: &capsule_dir,
        entrypoint,
        context_json: &paths.context_json,
        input_json: &paths.input_json,
        output_json: &paths.output_json,
        artifact_dir: &paths.artifact_dir,
        timeout,
    })?;
    fs::write(&paths.stdout_log, adapter_output.stdout)
        .map_err(|error| format!("failed to write {}: {error}", paths.stdout_log.display()))?;
    fs::write(&paths.stderr_log, adapter_output.stderr)
        .map_err(|error| format!("failed to write {}: {error}", paths.stderr_log.display()))?;

    if !paths.output_json.is_file() {
        return Err(format!(
            "adapter did not write output envelope: {}",
            paths.output_json.display()
        ));
    }

    let mut envelope = read_json_file(&paths.output_json)?;
    if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err("adapter did not return an ok: true envelope".to_string());
    }
    let Some(object) = envelope.as_object_mut() else {
        return Err("adapter output envelope must be a JSON object".to_string());
    };
    object.insert("run_id".to_string(), Value::String(run_id.clone()));
    object.insert(
        "run_dir".to_string(),
        Value::String(paths.run_dir.display().to_string()),
    );
    object.insert(
        "record".to_string(),
        Value::String(paths.record_json.display().to_string()),
    );
    write_json(&paths.output_json, &envelope)?;

    let finished_at = Utc::now();
    let skill_sha256 = string_at(&manifest.value, &["sources", "skill", "sha256"])
        .or_else(|| string_at(&manifest.value, &["skill", "skill_hash"]))
        .unwrap_or("missing");
    let action_sha256 =
        string_at(&manifest.value, &["sources", "action", "sha256"]).unwrap_or("missing");
    run_record::write(
        &paths.record_json,
        RunRecordInput {
            run_id: &run_id,
            mode,
            status: "succeeded",
            started_at,
            finished_at,
            duration: started.elapsed(),
            capsule_dir: &capsule_dir,
            manifest_path: &manifest.path,
            manifest_sha256: &manifest.sha256,
            skill_sha256,
            action_sha256,
            permissions: json_value_at(&manifest.value, &["permissions"]).unwrap_or(Value::Null),
        },
    )?;

    serde_json::to_string_pretty(&envelope)
        .map_err(|error| format!("failed to serialize output envelope: {error}"))
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

fn load_manifest(cwd: &Path) -> Result<RuntimeManifest, String> {
    let path = manifest::generated_manifest_path(cwd);
    if !path.is_file() {
        return Err(format!("missing Manifest: {}", path.display()));
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let value = serde_yaml::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    let sha256 = hashing::sha256_file(&path)?;
    Ok(RuntimeManifest {
        value,
        path,
        sha256,
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

fn value_at<'a>(value: &'a YamlValue, path: &[&str]) -> Option<&'a YamlValue> {
    let mut current = value;
    for segment in path {
        let key = YamlValue::String((*segment).to_string());
        current = current.as_mapping()?.get(&key)?;
    }
    Some(current)
}

fn string_at<'a>(value: &'a YamlValue, path: &[&str]) -> Option<&'a str> {
    value_at(value, path)?.as_str()
}

fn json_value_at(value: &YamlValue, path: &[&str]) -> Option<Value> {
    let value = value_at(value, path)?;
    serde_json::to_value(value).ok()
}
