use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest;
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
    let readiness = readiness::evaluate(&cwd)?;
    let manifest_path = manifest::generated_manifest_path(&cwd);
    let manifest_value = read_manifest(&manifest_path)?;

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

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for segment in path {
        let key = Value::String((*segment).to_string());
        current = current.as_mapping()?.get(&key)?;
    }
    current.as_str()
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
