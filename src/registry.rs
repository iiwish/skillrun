use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    Add {
        cwd: PathBuf,
        id: Option<String>,
    },
    List {
        json: bool,
    },
    Inspect {
        id: String,
        json: bool,
    },
    Remove {
        id: String,
        delete_files: bool,
        json: bool,
    },
}

pub struct RegistryOutput {
    pub output: String,
}

pub struct ConsumerRunsListOptions<'a> {
    pub json: bool,
    pub capsule_id: Option<&'a str>,
    pub source: RunsListSource,
    pub limit: Option<usize>,
    pub status_filter: Option<&'a str>,
    pub mode_filter: Option<&'a str>,
    pub ok_filter: Option<bool>,
    pub error_code_filter: Option<&'a str>,
    pub since_filter: Option<&'a str>,
    pub until_filter: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunsListSource {
    Scan,
    Index,
}

impl RunsListSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Index => "index",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouterCandidate {
    pub id: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub readiness_ok: bool,
    pub readiness_status: String,
    pub readiness_reason: Option<String>,
    pub readiness_next_step: String,
    pub tool_name: Option<String>,
    pub manifest_hash: Option<String>,
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

pub const IMPORTED_SKR_SOURCE_TYPE: &str = "imported_skr";

pub struct RegisteredCapsule {
    pub id: String,
}

pub struct ImportedCapsuleForReplace {
    pub path: PathBuf,
    pub enabled: bool,
}

pub struct RegistryEntryStatus {
    pub source_type: String,
    pub enabled: bool,
    pub path: String,
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
struct RegistryRemoveView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    registry_path: String,
    capsule: RemovedCapsuleView,
    removed: RegistryRemoveResultView,
    warnings: Vec<RegistryRemoveWarningView>,
}

#[derive(Debug, Serialize)]
struct RemovedCapsuleView {
    id: String,
    path: String,
    source_type: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct RegistryRemoveResultView {
    registry_entry: bool,
    files_deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    files_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegistryRemoveWarningView {
    code: &'static str,
    message: String,
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
    source: RunsListSourceView,
    scope: RunsScopeView,
    runs: Vec<RunSummaryView>,
}

#[derive(Debug, Serialize)]
struct RunsListSourceView {
    kind: &'static str,
    index_path: Option<String>,
    generated_at: Option<String>,
    stale: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConsumerRunsIndexFile {
    schema_version: String,
    generated_at: String,
    registry_path: String,
    runs: Vec<RunSummaryView>,
}

#[derive(Debug, Serialize)]
struct ConsumerRunsIndexRebuildView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    registry_path: String,
    index_path: String,
    generated_at: String,
    capsules_scanned: usize,
    runs_indexed: usize,
}

#[derive(Debug, Serialize)]
struct ConsumerRunsIndexStatusView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    registry_path: String,
    index_path: String,
    index: RunsIndexStatusDetailView,
    warnings: Vec<WarningView>,
}

#[derive(Debug, Serialize)]
struct RunsIndexStatusDetailView {
    exists: bool,
    readable: bool,
    supported_schema: bool,
    schema_version: Option<String>,
    generated_at: Option<String>,
    runs_indexed: Option<usize>,
    stale: Option<bool>,
    registry_modified_after_generated_at: Option<bool>,
    evidence_modified_after_generated_at: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ConsumerRunsInspectView {
    command: &'static str,
    schema_version: &'static str,
    registry_path: String,
    ok: bool,
    run_ref: Option<RunRefView>,
    capsule: Option<RunCapsuleView>,
    record: Option<RunRecordView>,
    input: InspectInputView,
    envelope: InspectEnvelopeView,
    artifacts: Vec<InspectArtifactView>,
    logs: InspectLogsView,
    warnings: Vec<WarningView>,
}

#[derive(Debug, Serialize)]
struct ConsumerRunsInspectErrorView {
    command: &'static str,
    schema_version: &'static str,
    registry_path: String,
    ok: bool,
    error: ErrorView,
    matches: Vec<RunRefView>,
}

#[derive(Debug, Serialize)]
struct RunsScopeView {
    kind: &'static str,
    capsule_id: Option<String>,
    source: &'static str,
    status: Option<String>,
    mode: Option<String>,
    ok: Option<bool>,
    error_code: Option<String>,
    since: Option<String>,
    until: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct RunRefView {
    kind: String,
    capsule_id: String,
    run_id: String,
}

#[derive(Debug, Serialize)]
struct RunCapsuleView {
    id: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct RunRecordView {
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
struct InspectInputView {
    included: bool,
    available: bool,
}

#[derive(Debug, Serialize)]
struct InspectEnvelopeView {
    included: bool,
    status: String,
    value: Option<JsonValue>,
}

#[derive(Debug, Serialize)]
struct InspectArtifactView {
    name: Option<String>,
    kind: Option<String>,
    path: Option<String>,
    available: bool,
}

#[derive(Debug, Serialize)]
struct InspectLogsView {
    stdout_available: bool,
    stderr_available: bool,
    stdout_included: bool,
    stderr_included: bool,
}

#[derive(Debug, Serialize)]
struct WarningView {
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorView {
    code: &'static str,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(default)]
    input: Option<String>,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    stdout: Option<String>,
    #[serde(default)]
    stderr: Option<String>,
    #[serde(default)]
    artifacts: Option<String>,
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
        RegistryCommand::Remove {
            id,
            delete_files,
            json,
        } => remove(id, *delete_files, *json),
    }
}

fn add(cwd: &Path, id: Option<&str>) -> Result<RegistryOutput, String> {
    let registered = register_capsule_with_source(cwd, id, "local_path")?;

    Ok(RegistryOutput {
        output: format!("registered {}\nenabled: false", registered.id),
    })
}

pub fn register_capsule_with_source(
    cwd: &Path,
    id: Option<&str>,
    source_type: &str,
) -> Result<RegisteredCapsule, String> {
    let capsule_path = absolute_existing_dir(cwd)?;
    let mut registry = load_registry()?;
    let registry_id = match id {
        Some(id) => {
            validate_registry_id(id)?;
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
        source_type: source_type.to_string(),
        enabled: false,
        registered_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    });
    save_registry(&registry)?;

    Ok(RegisteredCapsule { id: registry_id })
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

fn remove(id: &str, delete_files: bool, json: bool) -> Result<RegistryOutput, String> {
    let mut registry = load_registry()?;
    let registry_path = registry_path()?;
    let Some(index) = registry.capsules.iter().position(|entry| entry.id == id) else {
        return Err(format!("registry id not found: {id}"));
    };
    let entry = registry.capsules[index].clone();
    if delete_files && entry.source_type != IMPORTED_SKR_SOURCE_TYPE {
        return Err(format!(
            "registry remove --delete-files requires imported_skr source_type for {id}; found {}",
            entry.source_type
        ));
    }

    let mut staged_files = None;
    let mut warnings = Vec::new();
    if delete_files {
        staged_files = stage_imported_files_for_delete(&entry)?;
        if staged_files.is_none() {
            warnings.push(RegistryRemoveWarningView {
                code: "files-missing",
                message: format!(
                    "imported capsule files were already missing at {}",
                    entry.path
                ),
            });
        }
    }

    registry.capsules.remove(index);
    if let Err(error) = save_registry(&registry) {
        if let Some((original, backup)) = &staged_files {
            restore_staged_delete(original, backup);
        }
        return Err(error);
    }

    let mut files_deleted = false;
    let mut files_path = None;
    if let Some((original, backup)) = staged_files {
        files_path = Some(display_path(&original));
        match fs::remove_dir_all(&backup) {
            Ok(()) => files_deleted = true,
            Err(error) => warnings.push(RegistryRemoveWarningView {
                code: "files-delete-failed",
                message: format!(
                    "registry entry was removed, but staged files remain at {}: {error}",
                    backup.display()
                ),
            }),
        }
    }

    let view = RegistryRemoveView {
        command: "registry remove",
        schema_version: "registry.remove.v1",
        ok: true,
        registry_path: display_path(&registry_path),
        capsule: RemovedCapsuleView {
            id: entry.id.clone(),
            path: entry.path.clone(),
            source_type: entry.source_type.clone(),
            enabled: entry.enabled,
        },
        removed: RegistryRemoveResultView {
            registry_entry: true,
            files_deleted,
            files_path,
        },
        warnings,
    };

    if json {
        return json_output(&view);
    }

    let mut output = format!(
        "removed {id}\nregistry_entry: true\nfiles_deleted: {}",
        view.removed.files_deleted
    );
    for warning in view.warnings {
        output.push_str(&format!("\nwarning: {}", warning.message));
    }
    Ok(RegistryOutput { output })
}

fn stage_imported_files_for_delete(
    entry: &RegistryEntry,
) -> Result<Option<(PathBuf, PathBuf)>, String> {
    let path = PathBuf::from(&entry.path);
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_dir() {
        return Err(format!(
            "registry remove --delete-files expected a directory at {}",
            path.display()
        ));
    }
    let backup = path.with_file_name(format!(
        ".remove-{}-{}-{}.bak",
        entry.id,
        std::process::id(),
        nonce()?
    ));
    fs::rename(&path, &backup).map_err(|error| {
        format!(
            "failed to stage imported capsule files {} for deletion: {error}",
            path.display()
        )
    })?;
    Ok(Some((path, backup)))
}

fn restore_staged_delete(original: &Path, backup: &Path) {
    if backup.exists() && !original.exists() {
        fs::rename(backup, original).ok();
    }
}

fn nonce() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| format!("system clock is before unix epoch: {error}"))
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

pub fn router_candidates() -> Result<Vec<RouterCandidate>, String> {
    let registry = load_registry()?;
    let mut candidates = Vec::new();

    for entry in &registry.capsules {
        let capsule = capsule_view(entry)?;
        if !capsule.enabled {
            continue;
        }

        let capsule_path = PathBuf::from(&entry.path);
        let manifest_hash = if capsule.readiness.ok && capsule.tool.is_some() {
            let manifest_path = manifest::generated_manifest_path(&capsule_path);
            Some(hashing::sha256_file(&manifest_path).map_err(|error| {
                format!(
                    "failed to hash exposed Manifest for {} at {}: {error}",
                    entry.id,
                    manifest_path.display()
                )
            })?)
        } else {
            None
        };

        candidates.push(RouterCandidate {
            id: entry.id.clone(),
            path: capsule_path,
            enabled: capsule.enabled,
            readiness_ok: capsule.readiness.ok,
            readiness_status: capsule.readiness.status,
            readiness_reason: capsule.readiness.reason,
            readiness_next_step: capsule.readiness.next_step,
            tool_name: capsule.tool.map(|tool| tool.name),
            manifest_hash,
        });
    }

    Ok(candidates)
}

pub fn consumer_runs_list(options: ConsumerRunsListOptions<'_>) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let entries = registry_entries_for_scope(&registry, options.capsule_id)?;
    let since_filter = parse_run_time_filter("--since", options.since_filter)?;
    let until_filter = parse_run_time_filter("--until", options.until_filter)?;
    if let (Some(since), Some(until)) = (since_filter, until_filter) {
        if since > until {
            return Err("--since must be earlier than or equal to --until".to_string());
        }
    }
    let index_path = runs_index_path()?;
    let (source_view, mut runs) = match options.source {
        RunsListSource::Scan => (
            RunsListSourceView {
                kind: "scan",
                index_path: None,
                generated_at: None,
                stale: None,
            },
            collect_run_summaries(entries),
        ),
        RunsListSource::Index => {
            let all_entries = registry_entries_for_scope(&registry, None)?;
            let index = read_ready_runs_index(&index_path, &registry_path, &all_entries)?;
            (
                RunsListSourceView {
                    kind: "index",
                    index_path: Some(display_path(&index_path)),
                    generated_at: Some(index.generated_at.clone()),
                    stale: Some(false),
                },
                index.runs,
            )
        }
    };
    runs.sort_by(|left, right| {
        right
            .started_at
            .cmp(&left.started_at)
            .then_with(|| right.run_id.cmp(&left.run_id))
    });
    runs.retain(|summary| {
        if let Some(capsule_id) = options.capsule_id {
            if summary.capsule_id != capsule_id {
                return false;
            }
        }
        run_summary_matches(
            summary,
            options.status_filter,
            options.mode_filter,
            options.ok_filter,
            options.error_code_filter,
            since_filter,
            until_filter,
        )
    });
    if let Some(limit) = options.limit {
        runs.truncate(limit);
    }

    if options.json {
        let view = ConsumerRunsListView {
            command: "consumer runs list",
            schema_version: "consumer.runs.list.v1",
            registry_path: display_path(&registry_path),
            source: source_view,
            scope: RunsScopeView {
                kind: "registry",
                capsule_id: options.capsule_id.map(str::to_string),
                source: options.source.as_str(),
                status: options.status_filter.map(str::to_string),
                mode: options.mode_filter.map(str::to_string),
                ok: options.ok_filter,
                error_code: options.error_code_filter.map(str::to_string),
                since: since_filter.map(|timestamp| timestamp.to_rfc3339()),
                until: until_filter.map(|timestamp| timestamp.to_rfc3339()),
            },
            runs,
        };
        return json_output(&view);
    }

    let output = if runs.is_empty() {
        format!(
            "SkillRun Consumer Runs\nsource: {}\nevidence: none",
            options.source.as_str()
        )
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
        format!(
            "SkillRun Consumer Runs\nsource: {}\nevidence:\n{items}",
            options.source.as_str()
        )
    };
    Ok(RegistryOutput { output })
}

pub fn consumer_runs_index_rebuild(json: bool) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let index_path = runs_index_path()?;
    let entries = registry_entries_for_scope(&registry, None)?;
    let capsules_scanned = entries.len();
    let mut runs = collect_run_summaries(entries);
    runs.sort_by(|left, right| {
        right
            .started_at
            .cmp(&left.started_at)
            .then_with(|| right.run_id.cmp(&left.run_id))
    });
    let runs_indexed = runs.len();
    let generated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let index = ConsumerRunsIndexFile {
        schema_version: "consumer.runs.index.v1".to_string(),
        generated_at: generated_at.clone(),
        registry_path: display_path(&registry_path),
        runs,
    };
    write_json_file(&index_path, &index)?;

    let view = ConsumerRunsIndexRebuildView {
        command: "consumer runs index rebuild",
        schema_version: "consumer.runs.index.v1",
        ok: true,
        registry_path: display_path(&registry_path),
        index_path: display_path(&index_path),
        generated_at,
        capsules_scanned,
        runs_indexed,
    };

    if json {
        json_output(&view)
    } else {
        Ok(RegistryOutput {
            output: format!(
                "SkillRun Consumer Runs Index\nstatus: rebuilt\nruns indexed: {}\npath: {}",
                view.runs_indexed, view.index_path
            ),
        })
    }
}

pub fn consumer_runs_index_status(json: bool) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let index_path = runs_index_path()?;
    let entries = registry_entries_for_scope(&registry, None)?;
    let mut warnings = Vec::new();
    let mut detail = RunsIndexStatusDetailView {
        exists: index_path.is_file(),
        readable: false,
        supported_schema: false,
        schema_version: None,
        generated_at: None,
        runs_indexed: None,
        stale: None,
        registry_modified_after_generated_at: None,
        evidence_modified_after_generated_at: None,
    };

    if !detail.exists {
        warnings.push(WarningView {
            code: "missing-index",
            message: "runs-index.json is missing; run `skillrun consumer runs index rebuild`"
                .to_string(),
        });
    } else {
        match fs::read_to_string(&index_path) {
            Ok(text) => {
                detail.readable = true;
                match serde_json::from_str::<JsonValue>(&text) {
                    Ok(index) => {
                        detail.schema_version = index
                            .get("schema_version")
                            .and_then(JsonValue::as_str)
                            .map(str::to_string);
                        detail.supported_schema =
                            detail.schema_version.as_deref() == Some("consumer.runs.index.v1");
                        detail.generated_at = index
                            .get("generated_at")
                            .and_then(JsonValue::as_str)
                            .map(str::to_string);
                        detail.runs_indexed = index
                            .get("runs")
                            .and_then(JsonValue::as_array)
                            .map(Vec::len);

                        if !detail.supported_schema {
                            warnings.push(WarningView {
                                code: "unsupported-index-schema",
                                message: format!(
                                    "runs index schema is unsupported: {}",
                                    detail
                                        .schema_version
                                        .as_deref()
                                        .unwrap_or("<missing schema_version>")
                                ),
                            });
                        }

                        if let Some(generated_at) = &detail.generated_at {
                            match DateTime::parse_from_rfc3339(generated_at) {
                                Ok(timestamp) => {
                                    let generated_at = timestamp.with_timezone(&Utc);
                                    let registry_stale =
                                        path_modified_after(&registry_path, &generated_at);
                                    let evidence_stale =
                                        evidence_modified_after(&entries, &generated_at);
                                    detail.registry_modified_after_generated_at =
                                        Some(registry_stale);
                                    detail.evidence_modified_after_generated_at =
                                        Some(evidence_stale);
                                    detail.stale = Some(registry_stale || evidence_stale);
                                    if detail.stale == Some(true) {
                                        warnings.push(WarningView {
                                            code: "stale-index",
                                            message: "runs index may be stale; rebuild it before using it as a query cache".to_string(),
                                        });
                                    }
                                }
                                Err(error) => warnings.push(WarningView {
                                    code: "invalid-generated-at",
                                    message: format!("runs index generated_at is invalid: {error}"),
                                }),
                            }
                        } else {
                            warnings.push(WarningView {
                                code: "missing-generated-at",
                                message: "runs index is missing generated_at".to_string(),
                            });
                        }
                    }
                    Err(error) => warnings.push(WarningView {
                        code: "invalid-index",
                        message: format!("runs-index.json is not valid JSON: {error}"),
                    }),
                }
            }
            Err(error) => warnings.push(WarningView {
                code: "unreadable-index",
                message: format!("failed to read {}: {error}", index_path.display()),
            }),
        }
    }

    let ok = detail.exists
        && detail.readable
        && detail.supported_schema
        && detail.generated_at.is_some()
        && detail.runs_indexed.is_some()
        && detail.stale == Some(false);
    let view = ConsumerRunsIndexStatusView {
        command: "consumer runs index status",
        schema_version: "consumer.runs.index.status.v1",
        ok,
        registry_path: display_path(&registry_path),
        index_path: display_path(&index_path),
        index: detail,
        warnings,
    };

    if json {
        json_output(&view)
    } else {
        let status = if view.ok {
            "ready"
        } else if view.index.stale == Some(true) {
            "stale"
        } else if !view.index.exists {
            "missing"
        } else {
            "invalid"
        };
        let runs_indexed = view
            .index
            .runs_indexed
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Ok(RegistryOutput {
            output: format!(
                "SkillRun Consumer Runs Index\nstatus: {status}\nruns indexed: {runs_indexed}\npath: {}",
                view.index_path
            ),
        })
    }
}

fn read_ready_runs_index(
    index_path: &Path,
    registry_path: &Path,
    entries: &[&RegistryEntry],
) -> Result<ConsumerRunsIndexFile, String> {
    if !index_path.is_file() {
        return Err(
            "runs list --source index requires runs-index.json; run `skillrun consumer runs index rebuild`"
                .to_string(),
        );
    }

    let text = fs::read_to_string(index_path)
        .map_err(|error| format!("failed to read {}: {error}", index_path.display()))?;
    let index: ConsumerRunsIndexFile = serde_json::from_str(&text)
        .map_err(|error| format!("runs-index.json is not valid JSON: {error}"))?;

    if index.schema_version != "consumer.runs.index.v1" {
        return Err(format!(
            "runs index schema is unsupported: {}",
            index.schema_version
        ));
    }

    let generated_at = DateTime::parse_from_rfc3339(&index.generated_at)
        .map_err(|error| format!("runs index generated_at is invalid: {error}"))?
        .with_timezone(&Utc);
    let registry_stale = path_modified_after(registry_path, &generated_at);
    let evidence_stale = evidence_modified_after(entries, &generated_at);
    if registry_stale || evidence_stale {
        return Err(
            "runs index is stale; run `skillrun consumer runs index rebuild` before using --source index"
                .to_string(),
        );
    }

    Ok(index)
}

fn collect_run_summaries(entries: Vec<&RegistryEntry>) -> Vec<RunSummaryView> {
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

    runs
}

fn path_modified_after(path: &Path, generated_at: &DateTime<Utc>) -> bool {
    path_modified_at(path)
        .map(|modified_at| modified_at.timestamp() > generated_at.timestamp())
        .unwrap_or(false)
}

fn evidence_modified_after(entries: &[&RegistryEntry], generated_at: &DateTime<Utc>) -> bool {
    entries
        .iter()
        .filter_map(|entry| {
            let run_root = Path::new(&entry.path).join(".skillrun").join("runs");
            latest_tree_modified_at(&run_root)
        })
        .any(|modified_at| modified_at.timestamp() > generated_at.timestamp())
}

fn latest_tree_modified_at(root: &Path) -> Option<DateTime<Utc>> {
    let mut latest = path_modified_at(root);
    let Ok(children) = fs::read_dir(root) else {
        return latest;
    };

    for child in children.flatten() {
        let child_path = child.path();
        if child
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false)
        {
            latest = latest_datetime(latest, latest_tree_modified_at(&child_path));
        } else {
            latest = latest_datetime(latest, path_modified_at(&child_path));
        }
    }

    latest
}

fn path_modified_at(path: &Path) -> Option<DateTime<Utc>> {
    fs::symlink_metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .map(DateTime::<Utc>::from)
}

fn latest_datetime(
    left: Option<DateTime<Utc>>,
    right: Option<DateTime<Utc>>,
) -> Option<DateTime<Utc>> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn run_summary_matches(
    summary: &RunSummaryView,
    status_filter: Option<&str>,
    mode_filter: Option<&str>,
    ok_filter: Option<bool>,
    error_code_filter: Option<&str>,
    since_filter: Option<DateTime<Utc>>,
    until_filter: Option<DateTime<Utc>>,
) -> bool {
    if let Some(status) = status_filter {
        if summary.status != status {
            return false;
        }
    }
    if let Some(mode) = mode_filter {
        if summary.mode.as_deref() != Some(mode) {
            return false;
        }
    }
    if let Some(ok) = ok_filter {
        if summary.ok != Some(ok) {
            return false;
        }
    }
    if let Some(error_code) = error_code_filter {
        if summary.error_code.as_deref() != Some(error_code) {
            return false;
        }
    }
    if since_filter.is_some() || until_filter.is_some() {
        let Some(started_at) = summary.started_at.as_deref().and_then(parse_run_timestamp) else {
            return false;
        };
        if let Some(since) = since_filter {
            if started_at < since {
                return false;
            }
        }
        if let Some(until) = until_filter {
            if started_at > until {
                return false;
            }
        }
    }
    true
}

fn parse_run_time_filter(flag: &str, value: Option<&str>) -> Result<Option<DateTime<Utc>>, String> {
    value
        .map(|timestamp| {
            parse_run_timestamp(timestamp)
                .ok_or_else(|| format!("{flag} must be an RFC3339 timestamp: {timestamp}"))
        })
        .transpose()
}

fn parse_run_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .ok()
}

pub fn consumer_runs_inspect(
    run_id: &str,
    json: bool,
    capsule_id: Option<&str>,
) -> Result<RegistryOutput, String> {
    let registry = load_registry()?;
    let registry_path = registry_path()?;
    let registry_path_display = display_path(&registry_path);
    let entries = registry_entries_for_scope(&registry, capsule_id)?;
    let matches = matching_runs(entries, run_id);

    if matches.is_empty() {
        let view = ConsumerRunsInspectErrorView {
            command: "consumer runs inspect",
            schema_version: "consumer.runs.inspect.v1",
            registry_path: registry_path_display,
            ok: false,
            error: ErrorView {
                code: "RunNotFound",
                message: "run_id was not found in registered capsules".to_string(),
            },
            matches: Vec::new(),
        };
        if json {
            return json_output(&view);
        }
        return Ok(RegistryOutput {
            output: format!("SkillRun Consumer Run Inspect\nrun: {run_id}\nstatus: not found"),
        });
    }

    if matches.len() > 1 {
        let refs = matches
            .iter()
            .map(|(entry, _)| RunRefView {
                kind: "local_run".to_string(),
                capsule_id: entry.id.clone(),
                run_id: run_id.to_string(),
            })
            .collect::<Vec<_>>();
        let view = ConsumerRunsInspectErrorView {
            command: "consumer runs inspect",
            schema_version: "consumer.runs.inspect.v1",
            registry_path: registry_path_display,
            ok: false,
            error: ErrorView {
                code: "AmbiguousRunId",
                message: "run_id matched multiple registered capsules; pass --capsule <id>"
                    .to_string(),
            },
            matches: refs,
        };
        if json {
            return json_output(&view);
        }
        return Ok(RegistryOutput {
            output: format!("SkillRun Consumer Run Inspect\nrun: {run_id}\nstatus: ambiguous"),
        });
    }

    let (entry, run_dir) = &matches[0];
    let view = run_inspect_view(&registry_path_display, entry, run_dir, run_id);
    if json {
        return json_output(&view);
    }

    let output = format!(
        "SkillRun Consumer Run Inspect\nrun: {}\ncapsule: {}\nstatus: {}",
        run_id,
        entry.id,
        view.record
            .as_ref()
            .map(|record| record.status.as_str())
            .unwrap_or("degraded")
    );
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

fn matching_runs<'a>(
    entries: Vec<&'a RegistryEntry>,
    run_id: &str,
) -> Vec<(&'a RegistryEntry, PathBuf)> {
    entries
        .into_iter()
        .filter_map(|entry| {
            let run_dir = Path::new(&entry.path)
                .join(".skillrun")
                .join("runs")
                .join(run_id);
            run_dir.is_dir().then_some((entry, run_dir))
        })
        .collect()
}

fn run_inspect_view(
    registry_path: &str,
    entry: &RegistryEntry,
    run_dir: &Path,
    fallback_run_id: &str,
) -> ConsumerRunsInspectView {
    let run_ref = RunRefView {
        kind: "local_run".to_string(),
        capsule_id: entry.id.clone(),
        run_id: fallback_run_id.to_string(),
    };
    let mut warnings = Vec::new();
    let mut record_view = None;
    let mut record_for_files = None;

    let record_path = run_dir.join("record.json");
    if !record_path.is_file() {
        warnings.push(WarningView {
            code: "missing-record",
            message: "record.json is missing; run evidence is degraded".to_string(),
        });
    } else {
        match read_stored_run_record(&record_path) {
            Ok(record) => {
                record_for_files = Some(record.clone());
                record_view = Some(RunRecordView {
                    run_id: record.run_id.clone(),
                    mode: record.mode.clone(),
                    status: record.status.clone(),
                    started_at: record.started_at.clone(),
                    finished_at: record.finished_at.clone(),
                    duration_ms: record.duration_ms,
                    manifest_sha256: record.manifest_sha256.clone(),
                    skill_sha256: record.skill_sha256.clone(),
                    action_sha256: record.action_sha256.clone(),
                });
            }
            Err(message) => warnings.push(WarningView {
                code: "invalid-record",
                message,
            }),
        }
    }

    let input_name = evidence_file_name(
        record_for_files.as_ref(),
        |record| record.input.as_deref(),
        "input.json",
    );
    let output_name = evidence_file_name(
        record_for_files.as_ref(),
        |record| record.output.as_deref(),
        "output.json",
    );
    let stdout_name = evidence_file_name(
        record_for_files.as_ref(),
        |record| record.stdout.as_deref(),
        "stdout.log",
    );
    let stderr_name = evidence_file_name(
        record_for_files.as_ref(),
        |record| record.stderr.as_deref(),
        "stderr.log",
    );
    let artifacts_name = evidence_file_name(
        record_for_files.as_ref(),
        |record| record.artifacts.as_deref(),
        "artifacts",
    );

    let input = InspectInputView {
        included: false,
        available: run_dir.join(&input_name).is_file(),
    };

    let (envelope, artifacts) =
        inspect_envelope(run_dir, &output_name, &artifacts_name, &mut warnings);
    let logs = InspectLogsView {
        stdout_available: run_dir.join(&stdout_name).is_file(),
        stderr_available: run_dir.join(&stderr_name).is_file(),
        stdout_included: false,
        stderr_included: false,
    };

    ConsumerRunsInspectView {
        command: "consumer runs inspect",
        schema_version: "consumer.runs.inspect.v1",
        registry_path: registry_path.to_string(),
        ok: warnings.is_empty(),
        run_ref: Some(run_ref),
        capsule: Some(RunCapsuleView {
            id: entry.id.clone(),
            path: entry.path.clone(),
        }),
        record: record_view,
        input,
        envelope,
        artifacts,
        logs,
        warnings,
    }
}

fn read_stored_run_record(path: &Path) -> Result<StoredRunRecord, String> {
    read_json(path).and_then(|value| {
        serde_json::from_value::<StoredRunRecord>(value)
            .map_err(|error| format!("record.json is not a valid run record: {error}"))
    })
}

fn evidence_file_name(
    record: Option<&StoredRunRecord>,
    get: impl Fn(&StoredRunRecord) -> Option<&str>,
    fallback: &str,
) -> String {
    record
        .and_then(get)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn inspect_envelope(
    run_dir: &Path,
    output_name: &str,
    artifacts_name: &str,
    warnings: &mut Vec<WarningView>,
) -> (InspectEnvelopeView, Vec<InspectArtifactView>) {
    let envelope_path = run_dir.join(output_name);
    if !envelope_path.is_file() {
        warnings.push(WarningView {
            code: "missing-envelope",
            message: "output envelope is missing; run result is degraded".to_string(),
        });
        return (
            InspectEnvelopeView {
                included: false,
                status: "missing-envelope".to_string(),
                value: None,
            },
            Vec::new(),
        );
    }

    let envelope = match read_json(&envelope_path) {
        Ok(envelope) => envelope,
        Err(message) => {
            warnings.push(WarningView {
                code: "invalid-envelope",
                message,
            });
            return (
                InspectEnvelopeView {
                    included: false,
                    status: "invalid-envelope".to_string(),
                    value: None,
                },
                Vec::new(),
            );
        }
    };

    let artifacts = inspect_artifacts(run_dir, artifacts_name, &envelope);
    (
        InspectEnvelopeView {
            included: true,
            status: "ok".to_string(),
            value: Some(envelope),
        },
        artifacts,
    )
}

fn inspect_artifacts(
    run_dir: &Path,
    artifacts_name: &str,
    envelope: &JsonValue,
) -> Vec<InspectArtifactView> {
    envelope
        .get("artifacts")
        .and_then(JsonValue::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let path = item.get("path").and_then(JsonValue::as_str);
                    let name = item
                        .get("name")
                        .and_then(JsonValue::as_str)
                        .map(str::to_string);
                    let kind = item
                        .get("kind")
                        .and_then(JsonValue::as_str)
                        .map(str::to_string);
                    let available = path
                        .filter(|relative| is_safe_relative_artifact_path(relative))
                        .map(|relative| run_dir.join(artifacts_name).join(relative).is_file())
                        .unwrap_or(false);
                    InspectArtifactView {
                        name,
                        kind,
                        path: path.map(str::to_string),
                        available,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn is_safe_relative_artifact_path(path: &str) -> bool {
    let path = Path::new(path);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn run_summary_view(
    entry: &RegistryEntry,
    run_dir: &Path,
    fallback_run_id: &str,
) -> RunSummaryView {
    let record_path = run_dir.join("record.json");
    let run_ref = RunRefView {
        kind: "local_run".to_string(),
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
    validate_registry_id(&id)?;
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
    write_json_file(&path, registry)
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(&tmp, text).map_err(|error| format!("failed to write {}: {error}", tmp.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("failed to replace {}: {error}", path.display()))?;
    }
    fs::rename(&tmp, path).map_err(|error| format!("failed to replace {}: {error}", path.display()))
}

fn runs_index_path() -> Result<PathBuf, String> {
    registry_path().map(|path| path.with_file_name("runs-index.json"))
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

pub fn registry_path_display() -> Result<String, String> {
    registry_path().map(|path| display_path(&path))
}

pub fn ensure_registry_id_available(id: &str) -> Result<(), String> {
    validate_registry_id(id)?;
    let registry = load_registry()?;
    if registry.capsules.iter().any(|entry| entry.id == id) {
        return Err(format!("registry id already exists: {id}"));
    }
    Ok(())
}

pub fn imported_capsule_for_replace(id: &str) -> Result<ImportedCapsuleForReplace, String> {
    validate_registry_id(id)?;
    let registry = load_registry()?;
    let entry = registry
        .capsules
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("registry id not found: {id}"))?;
    if entry.source_type != IMPORTED_SKR_SOURCE_TYPE {
        return Err(format!(
            "import replace requires imported_skr source_type for {id}; found {}",
            entry.source_type
        ));
    }
    Ok(ImportedCapsuleForReplace {
        path: PathBuf::from(&entry.path),
        enabled: entry.enabled,
    })
}

pub fn registry_entry_status(id: &str) -> Result<Option<RegistryEntryStatus>, String> {
    validate_registry_id(id)?;
    let registry = load_registry()?;
    Ok(registry
        .capsules
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| RegistryEntryStatus {
            source_type: entry.source_type.clone(),
            enabled: entry.enabled,
            path: entry.path.clone(),
        }))
}

pub fn replace_imported_capsule(id: &str, path: &Path) -> Result<(), String> {
    validate_registry_id(id)?;
    let capsule_path = absolute_existing_dir(path)?;
    let mut registry = load_registry()?;
    let entry = registry
        .capsules
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("registry id not found: {id}"))?;
    if entry.source_type != IMPORTED_SKR_SOURCE_TYPE {
        return Err(format!(
            "import replace requires imported_skr source_type for {id}; found {}",
            entry.source_type
        ));
    }
    entry.path = display_path(&capsule_path);
    save_registry(&registry)
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

pub fn validate_registry_id(id: &str) -> Result<(), String> {
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
