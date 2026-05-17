use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::hashing;
use crate::manifest;
use crate::manifest_access::string_at;
use crate::readiness;

#[derive(Debug)]
pub struct RegistryOptions {
    pub command: RegistryCommand,
}

#[derive(Debug)]
pub enum RegistryCommand {
    Add { cwd: PathBuf, id: Option<String> },
    List { json: bool },
    Inspect { id: String, json: bool },
    Remove { id: String },
}

pub struct RegistryOutput {
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistryFile {
    version: u32,
    capsules: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryEntry {
    id: String,
    path: String,
    source_type: String,
    enabled: bool,
    registered_at: String,
}

#[derive(Debug, Serialize)]
struct RegistryListView {
    command: &'static str,
    version: u32,
    registry_path: String,
    capsules: Vec<CapsuleView>,
}

#[derive(Debug, Serialize)]
struct RegistryInspectView {
    command: &'static str,
    registry_path: String,
    capsule: CapsuleView,
}

#[derive(Debug, Serialize)]
struct SwitchboardListView {
    command: &'static str,
    registry_path: String,
    capsules: Vec<CapsuleView>,
}

#[derive(Debug, Serialize)]
struct ConsumerInventoryView {
    command: &'static str,
    schema_version: &'static str,
    version: u32,
    registry_path: String,
    capsules: Vec<CapsuleView>,
}

#[derive(Debug, Serialize)]
struct ConsumerExposureView {
    command: &'static str,
    schema_version: &'static str,
    registry_path: String,
    tools: Vec<ExposureToolView>,
}

#[derive(Debug, Serialize)]
struct ExposureToolView {
    capsule_id: String,
    tool_name: String,
    enabled: bool,
    exposed: bool,
    readiness_status: String,
    manifest_hash: String,
}

#[derive(Debug, Serialize)]
struct ConsumerRunsListView {
    command: &'static str,
    schema_version: &'static str,
    registry_path: String,
    scope: RunsScopeView,
    runs: Vec<RunSummaryView>,
}

#[derive(Debug, Serialize)]
struct RunsScopeView {
    kind: &'static str,
    capsule_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct RunSummaryView {
    run_id: String,
    run_ref: RunRefView,
    capsule_id: String,
    capsule_path: String,
    mode: Option<String>,
    status: String,
    ok: Option<bool>,
    error_code: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    duration_ms: Option<u128>,
    manifest_sha256: Option<String>,
    skill_sha256: Option<String>,
    action_sha256: Option<String>,
    artifact_count: usize,
    input_included: bool,
}

#[derive(Debug, Serialize)]
struct RunRefView {
    kind: &'static str,
    capsule_id: String,
    run_id: String,
}

#[derive(Debug, Deserialize)]
struct StoredRunRecord {
    run_id: String,
    mode: String,
    status: String,
    started_at: String,
    finished_at: String,
    duration_ms: u128,
    manifest_sha256: String,
    skill_sha256: String,
    action_sha256: String,
}

#[derive(Debug, Serialize)]
struct CapsuleView {
    id: String,
    path: String,
    source_type: String,
    enabled: bool,
    registered_at: String,
    manifest: ManifestView,
    #[serde(skip_serializing_if = "Option::is_none")]
    skill: Option<SkillView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime: Option<RuntimeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<ToolView>,
    readiness: ReadinessView,
}

#[derive(Debug, Serialize)]
struct ManifestView {
    path: String,
    present: bool,
    freshness: String,
}

#[derive(Debug, Serialize)]
struct SkillView {
    name: String,
}

#[derive(Debug, Serialize)]
struct RuntimeView {
    adapter: String,
    entrypoint: String,
}

#[derive(Debug, Serialize)]
struct ToolView {
    name: String,
}

#[derive(Debug, Serialize)]
struct ReadinessView {
    ok: bool,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    next_step: String,
}

pub fn run(options: &RegistryOptions) -> Result<RegistryOutput, String> {
    match &options.command {
        RegistryCommand::Add { cwd, id } => add(cwd, id.as_deref()),
        RegistryCommand::List { json } => list(*json),
        RegistryCommand::Inspect { id, json } => inspect(id, *json),
        RegistryCommand::Remove { id } => remove(id),
    }
}

fn add(cwd: &Path, id: Option<&str>) -> Result<RegistryOutput, String> {
    let capsule_path = absolute_existing_dir(cwd)?;
    let mut registry = load_registry()?;
    let registry_id = match id {
        Some(id) => {
            validate_id(id)?;
            id.to_string()
        }
        None => default_id(&capsule_path)?,
    };

    if registry
        .capsules
        .iter()
        .any(|entry| entry.id == registry_id)
    {
        return Err(format!("registry id already exists: {registry_id}"));
    }

    registry.capsules.push(RegistryEntry {
        id: registry_id.clone(),
        path: display_path(&capsule_path),
        source_type: "local_path".to_string(),
        enabled: false,
        registered_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    });
    save_registry(&registry)?;

    Ok(RegistryOutput {
        output: format!("registered {registry_id}\nenabled: false"),
    })
}

fn list(json: bool) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let capsules = registry
        .capsules
        .iter()
        .map(capsule_view)
        .collect::<Result<Vec<_>, _>>()?;

    if json {
        let view = RegistryListView {
            command: "registry list",
            version: registry.version,
            registry_path: display_path(&registry_path),
            capsules,
        };
        return json_output(&view);
    }

    let output = if capsules.is_empty() {
        "SkillRun Registry\ncapsules: none".to_string()
    } else {
        let items = capsules
            .iter()
            .map(|item| {
                format!(
                    "- {} enabled: {} status: {}",
                    item.id, item.enabled, item.readiness.status
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("SkillRun Registry\ncapsules:\n{items}")
    };
    Ok(RegistryOutput { output })
}

fn inspect(id: &str, json: bool) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let entry = registry
        .capsules
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("registry id not found: {id}"))?;
    let capsule = capsule_view(entry)?;

    if json {
        let view = RegistryInspectView {
            command: "registry inspect",
            registry_path: display_path(&registry_path),
            capsule,
        };
        return json_output(&view);
    }

    Ok(RegistryOutput {
        output: format!(
            "SkillRun Registry Inspect\nid: {}\npath: {}\nenabled: {}\nstatus: {}",
            capsule.id, capsule.path, capsule.enabled, capsule.readiness.status
        ),
    })
}

fn remove(id: &str) -> Result<RegistryOutput, String> {
    let mut registry = load_registry()?;
    let before = registry.capsules.len();
    registry.capsules.retain(|entry| entry.id != id);
    if registry.capsules.len() == before {
        return Err(format!("registry id not found: {id}"));
    }
    save_registry(&registry)?;
    Ok(RegistryOutput {
        output: format!("removed {id}"),
    })
}

pub fn switchboard_list(json: bool) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let capsules = registry
        .capsules
        .iter()
        .map(capsule_view)
        .collect::<Result<Vec<_>, _>>()?;

    if json {
        let view = SwitchboardListView {
            command: "switchboard list",
            registry_path: display_path(&registry_path),
            capsules,
        };
        return json_output(&view);
    }

    let output = if capsules.is_empty() {
        "SkillRun Switchboard\ncapsules: none".to_string()
    } else {
        let items = capsules
            .iter()
            .map(|item| {
                format!(
                    "- {} enabled: {} ready: {}",
                    item.id, item.enabled, item.readiness.ok
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("SkillRun Switchboard\ncapsules:\n{items}")
    };
    Ok(RegistryOutput { output })
}

pub fn consumer_inventory(json: bool) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let capsules = registry
        .capsules
        .iter()
        .map(capsule_view)
        .collect::<Result<Vec<_>, _>>()?;

    if json {
        let view = ConsumerInventoryView {
            command: "consumer inventory",
            schema_version: "consumer.inventory.v1",
            version: registry.version,
            registry_path: display_path(&registry_path),
            capsules,
        };
        return json_output(&view);
    }

    let output = if capsules.is_empty() {
        "SkillRun Consumer Inventory\ncapsules: none".to_string()
    } else {
        let items = capsules
            .iter()
            .map(|item| {
                format!(
                    "- {} enabled: {} status: {}",
                    item.id, item.enabled, item.readiness.status
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("SkillRun Consumer Inventory\ncapsules:\n{items}")
    };
    Ok(RegistryOutput { output })
}

pub fn consumer_exposure(json: bool) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let mut tools = Vec::new();

    for entry in &registry.capsules {
        let capsule = capsule_view(entry)?;
        if !capsule.enabled || !capsule.readiness.ok {
            continue;
        }

        let Some(tool) = &capsule.tool else {
            continue;
        };

        let manifest_path = manifest::generated_manifest_path(Path::new(&entry.path));
        let manifest_hash = hashing::sha256_file(&manifest_path).map_err(|error| {
            format!(
                "failed to hash exposed Manifest for {} at {}: {error}",
                entry.id,
                manifest_path.display()
            )
        })?;

        tools.push(ExposureToolView {
            capsule_id: capsule.id,
            tool_name: tool.name.clone(),
            enabled: true,
            exposed: true,
            readiness_status: capsule.readiness.status,
            manifest_hash,
        });
    }

    if json {
        let view = ConsumerExposureView {
            command: "consumer exposure",
            schema_version: "consumer.exposure.v1",
            registry_path: display_path(&registry_path),
            tools,
        };
        return json_output(&view);
    }

    let output = if tools.is_empty() {
        "SkillRun Consumer Exposure\ntools: none".to_string()
    } else {
        let items = tools
            .iter()
            .map(|item| format!("- {} capsule: {}", item.tool_name, item.capsule_id))
            .collect::<Vec<_>>()
            .join("\n");
        format!("SkillRun Consumer Exposure\ntools:\n{items}")
    };
    Ok(RegistryOutput { output })
}

pub fn consumer_runs_list(
    json: bool,
    capsule_id: Option<&str>,
    limit: Option<usize>,
) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let entries = registry_entries_for_scope(&registry, capsule_id)?;
    let mut runs = Vec::new();

    for entry in entries {
        let run_root = Path::new(&entry.path).join(".skillrun").join("runs");
        let Ok(children) = fs::read_dir(&run_root) else {
            continue;
        };

        for child in children {
            let Ok(child) = child else {
                continue;
            };
            let run_dir = child.path();
            if !run_dir.is_dir() {
                continue;
            }
            let Some(run_id) = run_dir
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            runs.push(run_summary_view(entry, &run_dir, &run_id));
        }
    }

    runs.sort_by(|left, right| {
        right
            .started_at
            .cmp(&left.started_at)
            .then_with(|| right.run_id.cmp(&left.run_id))
    });
    if let Some(limit) = limit {
        runs.truncate(limit);
    }

    if json {
        let view = ConsumerRunsListView {
            command: "consumer runs list",
            schema_version: "consumer.runs.list.v1",
            registry_path: display_path(&registry_path),
            scope: RunsScopeView {
                kind: "registry",
                capsule_id: capsule_id.map(str::to_string),
            },
            runs,
        };
        return json_output(&view);
    }

    let output = if runs.is_empty() {
        "SkillRun Consumer Runs\nevidence: none".to_string()
    } else {
        let items = runs
            .iter()
            .map(|item| {
                format!(
                    "- {} capsule: {} status: {} ok: {}",
                    item.run_id,
                    item.capsule_id,
                    item.status,
                    item.ok
                        .map(|ok| ok.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("SkillRun Consumer Runs\nevidence:\n{items}")
    };
    Ok(RegistryOutput { output })
}

pub fn enable(id: &str) -> Result<RegistryOutput, String> {
    let mut registry = load_registry()?;
    let index = registry
        .capsules
        .iter()
        .position(|entry| entry.id == id)
        .ok_or_else(|| format!("registry id not found: {id}"))?;
    let view = capsule_view(&registry.capsules[index])?;

    if !view.readiness.ok {
        return Err(format!(
            "cannot enable {id}: readiness status {}. next step: {}",
            view.readiness.status, view.readiness.next_step
        ));
    }

    registry.capsules[index].enabled = true;
    save_registry(&registry)?;
    Ok(RegistryOutput {
        output: format!("enabled {id}"),
    })
}

pub fn disable(id: &str) -> Result<RegistryOutput, String> {
    let mut registry = load_registry()?;
    let entry = registry
        .capsules
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("registry id not found: {id}"))?;
    entry.enabled = false;
    save_registry(&registry)?;
    Ok(RegistryOutput {
        output: format!("disabled {id}"),
    })
}

fn capsule_view(entry: &RegistryEntry) -> Result<CapsuleView, String> {
    let cwd = PathBuf::from(&entry.path);
    if !cwd.exists() {
        return Ok(unavailable_capsule_view(
            entry,
            "missing-path",
            format!("cwd does not exist: {}", cwd.display()),
            format!(
                "Restore the capsule path or run `skillrun registry remove {}`.",
                entry.id
            ),
        ));
    }
    if !cwd.is_dir() {
        return Ok(unavailable_capsule_view(
            entry,
            "invalid-path",
            format!("cwd is not a directory: {}", cwd.display()),
            format!(
                "Restore the capsule directory or run `skillrun registry remove {}`.",
                entry.id
            ),
        ));
    }

    let manifest_path = manifest::generated_manifest_path(&cwd);
    let readiness = match readiness::evaluate(&cwd) {
        Ok(readiness) => readiness,
        Err(error) => {
            return Ok(manifest_error_capsule_view(
                entry,
                &cwd,
                &manifest_path,
                error,
            ));
        }
    };
    let manifest_value = match read_manifest(&manifest_path) {
        Ok(value) => value,
        Err(error) => {
            return Ok(manifest_error_capsule_view(
                entry,
                &cwd,
                &manifest_path,
                error,
            ));
        }
    };

    Ok(CapsuleView {
        id: entry.id.clone(),
        path: entry.path.clone(),
        source_type: entry.source_type.clone(),
        enabled: entry.enabled,
        registered_at: entry.registered_at.clone(),
        manifest: ManifestView {
            path: relative_path(&cwd, &readiness.manifest_path),
            present: readiness.manifest_present,
            freshness: readiness.freshness.clone(),
        },
        skill: manifest_value
            .as_ref()
            .and_then(|value| string_at(value, &["skill", "name"]))
            .map(|name| SkillView {
                name: name.to_string(),
            }),
        runtime: match (
            readiness.adapter.as_deref(),
            readiness.entrypoint.as_deref(),
        ) {
            (Some(adapter), Some(entrypoint)) => Some(RuntimeView {
                adapter: adapter.to_string(),
                entrypoint: entrypoint.to_string(),
            }),
            _ => None,
        },
        tool: manifest_value
            .as_ref()
            .and_then(|value| string_at(value, &["tool", "name"]))
            .map(|name| ToolView {
                name: name.to_string(),
            }),
        readiness: ReadinessView {
            ok: readiness.ok,
            status: readiness.status,
            reason: readiness.reason,
            next_step: readiness.next_step,
        },
    })
}

fn unavailable_capsule_view(
    entry: &RegistryEntry,
    status: &str,
    reason: String,
    next_step: String,
) -> CapsuleView {
    CapsuleView {
        id: entry.id.clone(),
        path: entry.path.clone(),
        source_type: entry.source_type.clone(),
        enabled: entry.enabled,
        registered_at: entry.registered_at.clone(),
        manifest: ManifestView {
            path: ".skillrun/manifest.generated.yaml".to_string(),
            present: false,
            freshness: "missing".to_string(),
        },
        skill: None,
        runtime: None,
        tool: None,
        readiness: ReadinessView {
            ok: false,
            status: status.to_string(),
            reason: Some(reason),
            next_step,
        },
    }
}

fn manifest_error_capsule_view(
    entry: &RegistryEntry,
    cwd: &Path,
    manifest_path: &Path,
    reason: String,
) -> CapsuleView {
    let (status, freshness) = if reason.contains("failed to parse") {
        ("invalid-manifest", "invalid")
    } else {
        ("unreadable-manifest", "unreadable")
    };
    unavailable_capsule_view_with_manifest(
        entry,
        ManifestView {
            path: relative_path(cwd, manifest_path),
            present: manifest_path.is_file(),
            freshness: freshness.to_string(),
        },
        status,
        reason,
        format!(
            "Regenerate the Manifest with `skillrun manifest --cwd {}` or run `skillrun registry remove {}`.",
            entry.path, entry.id
        ),
    )
}

fn unavailable_capsule_view_with_manifest(
    entry: &RegistryEntry,
    manifest: ManifestView,
    status: &str,
    reason: String,
    next_step: String,
) -> CapsuleView {
    CapsuleView {
        id: entry.id.clone(),
        path: entry.path.clone(),
        source_type: entry.source_type.clone(),
        enabled: entry.enabled,
        registered_at: entry.registered_at.clone(),
        manifest,
        skill: None,
        runtime: None,
        tool: None,
        readiness: ReadinessView {
            ok: false,
            status: status.to_string(),
            reason: Some(reason),
            next_step,
        },
    }
}

fn registry_entries_for_scope<'a>(
    registry: &'a RegistryFile,
    capsule_id: Option<&str>,
) -> Result<Vec<&'a RegistryEntry>, String> {
    match capsule_id {
        Some(id) => registry
            .capsules
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| vec![entry])
            .ok_or_else(|| format!("registry id not found: {id}")),
        None => Ok(registry.capsules.iter().collect()),
    }
}

fn run_summary_view(
    entry: &RegistryEntry,
    run_dir: &Path,
    fallback_run_id: &str,
) -> RunSummaryView {
    let record_path = run_dir.join("record.json");
    let run_ref = RunRefView {
        kind: "local_run",
        capsule_id: entry.id.clone(),
        run_id: fallback_run_id.to_string(),
    };

    if !record_path.is_file() {
        return degraded_run_summary(entry, run_ref, "missing-record");
    }

    let record = match read_json(&record_path).and_then(|value| {
        serde_json::from_value::<StoredRunRecord>(value).map_err(|error| error.to_string())
    }) {
        Ok(record) => record,
        Err(_) => return degraded_run_summary(entry, run_ref, "invalid-record"),
    };

    let envelope_path = run_dir.join("output.json");
    let (status, ok, error_code, artifact_count) = match read_json(&envelope_path) {
        Ok(envelope) => envelope_summary(&record.status, &envelope),
        Err(error) if error.contains("failed to read") => {
            ("missing-envelope".to_string(), None, None, 0)
        }
        Err(_) => ("invalid-envelope".to_string(), None, None, 0),
    };

    RunSummaryView {
        run_id: record.run_id.clone(),
        run_ref: RunRefView {
            run_id: record.run_id.clone(),
            ..run_ref
        },
        capsule_id: entry.id.clone(),
        capsule_path: entry.path.clone(),
        mode: Some(record.mode),
        status,
        ok,
        error_code,
        started_at: Some(record.started_at),
        finished_at: Some(record.finished_at),
        duration_ms: Some(record.duration_ms),
        manifest_sha256: Some(record.manifest_sha256),
        skill_sha256: Some(record.skill_sha256),
        action_sha256: Some(record.action_sha256),
        artifact_count,
        input_included: false,
    }
}

fn degraded_run_summary(
    entry: &RegistryEntry,
    run_ref: RunRefView,
    status: &str,
) -> RunSummaryView {
    RunSummaryView {
        run_id: run_ref.run_id.clone(),
        run_ref,
        capsule_id: entry.id.clone(),
        capsule_path: entry.path.clone(),
        mode: None,
        status: status.to_string(),
        ok: None,
        error_code: None,
        started_at: None,
        finished_at: None,
        duration_ms: None,
        manifest_sha256: None,
        skill_sha256: None,
        action_sha256: None,
        artifact_count: 0,
        input_included: false,
    }
}

fn envelope_summary(
    record_status: &str,
    envelope: &JsonValue,
) -> (String, Option<bool>, Option<String>, usize) {
    let Some(ok) = envelope.get("ok").and_then(JsonValue::as_bool) else {
        return ("invalid-envelope".to_string(), None, None, 0);
    };
    let error_code = envelope
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(JsonValue::as_str)
        .map(str::to_string);
    let artifact_count = envelope
        .get("artifacts")
        .and_then(JsonValue::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    (
        record_status.to_string(),
        Some(ok),
        error_code,
        artifact_count,
    )
}

fn default_id(capsule_path: &Path) -> Result<String, String> {
    let manifest_path = manifest::generated_manifest_path(capsule_path);
    let id = read_manifest(&manifest_path)?
        .as_ref()
        .and_then(|value| string_at(value, &["skill", "name"]))
        .or_else(|| capsule_path.file_name().and_then(|name| name.to_str()))
        .ok_or_else(|| {
            format!(
                "could not derive registry id from {}",
                capsule_path.display()
            )
        })?
        .to_string();
    validate_id(&id)?;
    Ok(id)
}

fn read_manifest(path: &Path) -> Result<Option<Value>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let value = serde_yaml::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    Ok(Some(value))
}

fn read_json(path: &Path) -> Result<JsonValue, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn load_registry() -> Result<RegistryFile, String> {
    let path = registry_path()?;
    if !path.is_file() {
        return Ok(RegistryFile {
            version: 1,
            capsules: Vec::new(),
        });
    }

    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn save_registry(registry: &RegistryFile) -> Result<(), String> {
    let path = registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(registry).map_err(|error| error.to_string())?;
    fs::write(&tmp, text).map_err(|error| format!("failed to write {}: {error}", tmp.display()))?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("failed to replace {}: {error}", path.display()))?;
    }
    fs::rename(&tmp, &path)
        .map_err(|error| format!("failed to replace {}: {error}", path.display()))
}

fn registry_path() -> Result<PathBuf, String> {
    if let Some(home) = std::env::var_os("SKILLRUN_HOME") {
        return Ok(PathBuf::from(home).join("registry.json"));
    }

    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| "SKILLRUN_HOME, USERPROFILE, or HOME must be set".to_string())?;
    Ok(PathBuf::from(home).join(".skillrun").join("registry.json"))
}

fn absolute_existing_dir(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("cwd does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("cwd is not a directory: {}", path.display()));
    }
    fs::canonicalize(path).map_err(|error| format!("failed to resolve {}: {error}", path.display()))
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("registry id cannot be empty".to_string());
    }
    if id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
    {
        Ok(())
    } else {
        Err(format!(
            "registry id may only contain ASCII letters, digits, '.', '_' or '-': {id}"
        ))
    }
}

fn json_output<T: Serialize>(value: &T) -> Result<RegistryOutput, String> {
    Ok(RegistryOutput {
        output: serde_json::to_string_pretty(value).map_err(|error| error.to_string())?,
    })
}

fn relative_path(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
