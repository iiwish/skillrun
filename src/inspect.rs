use serde::Serialize;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::consumer;
use crate::manifest;
use crate::manifest_access::{string_at, value_at};

#[derive(Debug)]
pub struct InspectOptions {
    pub cwd: PathBuf,
    pub json: bool,
}

pub fn render(options: &InspectOptions) -> Result<String, String> {
    let cwd = options.cwd.clone();
    if !cwd.exists() {
        return Err(format!("cwd does not exist: {}", cwd.display()));
    }
    if !cwd.is_dir() {
        return Err(format!("cwd is not a directory: {}", cwd.display()));
    }

    if options.json {
        render_json(&cwd)
    } else {
        render_text(&cwd)
    }
}

fn render_text(cwd: &Path) -> Result<String, String> {
    let manifest_path = manifest::generated_manifest_path(cwd);
    if manifest_path.is_file() {
        match consumer::validate(cwd, "skillrun inspect") {
            Ok(valid) => render_runnable(cwd, &valid.path, &valid.value),
            Err(error) => Ok(render_invalid_runnable(cwd, &manifest_path, &error)),
        }
    } else {
        render_instruction_only(cwd, &manifest_path)
    }
}

#[derive(Debug, Serialize)]
struct InspectJsonReport {
    command: &'static str,
    cwd: String,
    status: &'static str,
    manifest: JsonManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skill: Option<JsonSkill>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sources: Option<JsonSources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schemas: Option<JsonSchemas>,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime: Option<JsonRuntime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<JsonPermissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    examples: Option<Vec<JsonExample>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preflight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<JsonTool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing: Vec<String>,
}

#[derive(Debug, Serialize)]
struct JsonManifest {
    path: String,
    present: bool,
    freshness: &'static str,
}

#[derive(Debug, Serialize)]
struct JsonSkill {
    name: String,
    sop_hash: String,
}

#[derive(Debug, Serialize)]
struct JsonSources {
    skill: JsonSource,
    action: JsonSource,
}

#[derive(Debug, Serialize)]
struct JsonSource {
    path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct JsonSchemas {
    input: &'static str,
    output: &'static str,
}

#[derive(Debug, Serialize)]
struct JsonRuntime {
    adapter: String,
    entrypoint: String,
    timeout: String,
}

#[derive(Debug, Serialize)]
struct JsonPermissions {
    files: JsonFilePermissions,
    network: JsonNetworkPermissions,
    env: JsonEnvPermissions,
}

#[derive(Debug, Serialize)]
struct JsonFilePermissions {
    read: Vec<String>,
    write: Vec<String>,
}

#[derive(Debug, Serialize)]
struct JsonNetworkPermissions {
    outbound: Vec<String>,
}

#[derive(Debug, Serialize)]
struct JsonEnvPermissions {
    read: Vec<String>,
}

#[derive(Debug, Serialize)]
struct JsonExample {
    id: String,
    input: String,
}

#[derive(Debug, Serialize)]
struct JsonTool {
    name: String,
    description: String,
}

fn render_json(cwd: &Path) -> Result<String, String> {
    let manifest_path = manifest::generated_manifest_path(cwd);
    let report = if manifest_path.is_file() {
        match consumer::validate(cwd, "skillrun inspect --json") {
            Ok(valid) => runnable_json_report(cwd, &valid.path, &valid.value),
            Err(error) => invalid_runnable_json_report(cwd, &manifest_path, &error),
        }
    } else {
        instruction_only_json_report(cwd, &manifest_path)?
    };

    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
}

fn runnable_json_report(cwd: &Path, manifest_path: &Path, manifest: &Value) -> InspectJsonReport {
    let fallback_name = cwd
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("skill");
    let name = string_at(manifest, &["skill", "name"]).unwrap_or(fallback_name);
    let sop_hash = string_at(manifest, &["sources", "skill", "sha256"])
        .or_else(|| string_at(manifest, &["skill", "skill_hash"]))
        .unwrap_or("unknown");
    let action_hash = string_at(manifest, &["sources", "action", "sha256"]).unwrap_or("unknown");
    let skill_path = string_at(manifest, &["sources", "skill", "path"]).unwrap_or("SKILL.md");
    let action_path = string_at(manifest, &["sources", "action", "path"]).unwrap_or("action.py");
    let adapter = string_at(manifest, &["runtime", "adapter"]).unwrap_or("unknown");
    let entrypoint = string_at(manifest, &["runtime", "entrypoint"]).unwrap_or("action.py");
    let timeout = string_at(manifest, &["runtime", "timeout"]).unwrap_or("unknown");
    let tool_name = string_at(manifest, &["tool", "name"]).unwrap_or(name);
    let tool_description = string_at(manifest, &["tool", "description"]).unwrap_or("unknown");

    InspectJsonReport {
        command: "inspect",
        cwd: cwd.display().to_string(),
        status: "runnable",
        manifest: JsonManifest {
            path: display_path(cwd, manifest_path),
            present: true,
            freshness: "fresh",
        },
        reason: None,
        skill: Some(JsonSkill {
            name: name.to_string(),
            sop_hash: sop_hash.to_string(),
        }),
        sources: Some(JsonSources {
            skill: JsonSource {
                path: skill_path.to_string(),
                sha256: sop_hash.to_string(),
            },
            action: JsonSource {
                path: action_path.to_string(),
                sha256: action_hash.to_string(),
            },
        }),
        schemas: Some(JsonSchemas {
            input: presence(value_at(manifest, &["schemas", "input"])),
            output: presence(value_at(manifest, &["schemas", "output"])),
        }),
        runtime: Some(JsonRuntime {
            adapter: adapter.to_string(),
            entrypoint: entrypoint.to_string(),
            timeout: timeout.to_string(),
        }),
        permissions: Some(JsonPermissions {
            files: JsonFilePermissions {
                read: strings_at(manifest, &["permissions", "files", "read"]),
                write: strings_at(manifest, &["permissions", "files", "write"]),
            },
            network: JsonNetworkPermissions {
                outbound: strings_at(manifest, &["permissions", "network", "outbound"]),
            },
            env: JsonEnvPermissions {
                read: strings_at(manifest, &["permissions", "env", "read"]),
            },
        }),
        examples: Some(example_reports(manifest)),
        preflight: Some(preflight_status(cwd, entrypoint)),
        tool: Some(JsonTool {
            name: tool_name.to_string(),
            description: tool_description.to_string(),
        }),
        missing: Vec::new(),
    }
}

fn invalid_runnable_json_report(
    cwd: &Path,
    manifest_path: &Path,
    reason: &str,
) -> InspectJsonReport {
    InspectJsonReport {
        command: "inspect",
        cwd: cwd.display().to_string(),
        status: "invalid-runnable",
        manifest: JsonManifest {
            path: display_path(cwd, manifest_path),
            present: true,
            freshness: invalid_manifest_freshness(reason),
        },
        reason: Some(reason.to_string()),
        skill: None,
        sources: None,
        schemas: None,
        runtime: None,
        permissions: None,
        examples: None,
        preflight: None,
        tool: None,
        missing: Vec::new(),
    }
}

fn instruction_only_json_report(
    cwd: &Path,
    manifest_path: &Path,
) -> Result<InspectJsonReport, String> {
    let skill_path = cwd.join("SKILL.md");
    if !skill_path.is_file() {
        return Err(format!(
            "not a SkillRun capsule or instruction-only Skill: missing SKILL.md in {}",
            cwd.display()
        ));
    }

    Ok(InspectJsonReport {
        command: "inspect",
        cwd: cwd.display().to_string(),
        status: "instruction-only",
        manifest: JsonManifest {
            path: display_path(cwd, manifest_path),
            present: false,
            freshness: "missing",
        },
        reason: Some("not a runnable capsule".to_string()),
        skill: None,
        sources: None,
        schemas: None,
        runtime: None,
        permissions: None,
        examples: None,
        preflight: None,
        tool: None,
        missing: instruction_only_missing(cwd, manifest_path),
    })
}

fn render_runnable(cwd: &Path, manifest_path: &Path, manifest: &Value) -> Result<String, String> {
    let fallback_name = cwd
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("skill");
    let name = string_at(manifest, &["skill", "name"]).unwrap_or(fallback_name);
    let sop_hash = string_at(manifest, &["sources", "skill", "sha256"])
        .or_else(|| string_at(manifest, &["skill", "skill_hash"]))
        .unwrap_or("unknown");
    let action_hash = string_at(manifest, &["sources", "action", "sha256"]).unwrap_or("unknown");
    let input_schema = presence(value_at(manifest, &["schemas", "input"]));
    let output_schema = presence(value_at(manifest, &["schemas", "output"]));
    let adapter = string_at(manifest, &["runtime", "adapter"]).unwrap_or("unknown");
    let entrypoint = string_at(manifest, &["runtime", "entrypoint"]).unwrap_or("action.py");
    let timeout = string_at(manifest, &["runtime", "timeout"]).unwrap_or("unknown");
    let file_reads = strings_at(manifest, &["permissions", "files", "read"]);
    let file_writes = strings_at(manifest, &["permissions", "files", "write"]);
    let network = strings_at(manifest, &["permissions", "network", "outbound"]);
    let env_reads = strings_at(manifest, &["permissions", "env", "read"]);
    let examples = examples(manifest);
    let tool_name = string_at(manifest, &["tool", "name"]).unwrap_or(name);
    let tool_description = string_at(manifest, &["tool", "description"]).unwrap_or("unknown");
    let preflight = preflight_status(cwd, entrypoint);

    Ok(format!(
        "\
SkillRun Inspect
cwd: {cwd}
status: runnable
name: {name}
manifest: {manifest_path}
source hashes:
  SOP hash: {sop_hash}
  action hash: {action_hash}
schemas:
  input schema: {input_schema}
  output schema: {output_schema}
runtime:
  runtime contract: Manifest adapter and entrypoint
  adapter: {adapter}
  entrypoint: {entrypoint}
  timeout: {timeout}
permissions:
  file read: {file_reads}
  file write: {file_writes}
  network outbound: {network}
  env read: {env_reads}
examples:
{examples}
preflight: {preflight}
MCP tool: {tool_name}
MCP description: {tool_description}",
        cwd = cwd.display(),
        manifest_path = manifest_path.display(),
        file_reads = list_or_none(&file_reads),
        file_writes = list_or_none(&file_writes),
        network = list_or_none(&network),
        env_reads = list_or_none(&env_reads),
    ))
}

fn render_invalid_runnable(cwd: &Path, manifest_path: &Path, reason: &str) -> String {
    format!(
        "\
SkillRun Inspect
cwd: {cwd}
status: invalid-runnable
manifest: {manifest_path}
reason: {reason}
note: Consumer Mode fails closed until sources and Manifest hashes match.",
        cwd = cwd.display(),
        manifest_path = manifest_path.display()
    )
}

fn render_instruction_only(cwd: &Path, manifest_path: &Path) -> Result<String, String> {
    let skill_path = cwd.join("SKILL.md");
    if !skill_path.is_file() {
        return Err(format!(
            "not a SkillRun capsule or instruction-only Skill: missing SKILL.md in {}",
            cwd.display()
        ));
    }

    let action_path = cwd.join("action.py");
    let mut missing = Vec::new();
    if !action_path.is_file() {
        missing.push("missing action.py".to_string());
    }
    if !manifest_path.is_file() {
        missing.push("missing .skillrun/manifest.generated.yaml".to_string());
    }

    let missing_lines = missing
        .iter()
        .map(|item| format!("  - {item}"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "\
SkillRun Inspect
cwd: {cwd}
status: instruction-only
reason: not a runnable capsule
instruction file: SKILL.md
missing:
{missing_lines}
note: instruction-only Skill directories remain documentation until an action and Manifest are present.",
        cwd = cwd.display()
    ))
}

fn strings_at(value: &Value, path: &[&str]) -> Vec<String> {
    match value_at(value, path) {
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn presence(value: Option<&Value>) -> &'static str {
    match value {
        Some(Value::Null) | None => "absent",
        Some(_) => "present",
    }
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

fn examples(manifest: &Value) -> String {
    let Some(Value::Sequence(items)) = value_at(manifest, &["examples"]) else {
        return "  none".to_string();
    };

    let rendered = items
        .iter()
        .map(|item| {
            let id = string_at(item, &["id"]).unwrap_or("example");
            let input = string_at(item, &["input"]).unwrap_or("unknown");
            format!("  - {id}: {input}")
        })
        .collect::<Vec<_>>();

    if rendered.is_empty() {
        "  none".to_string()
    } else {
        rendered.join("\n")
    }
}

fn example_reports(manifest: &Value) -> Vec<JsonExample> {
    let Some(Value::Sequence(items)) = value_at(manifest, &["examples"]) else {
        return Vec::new();
    };

    items
        .iter()
        .map(|item| JsonExample {
            id: string_at(item, &["id"]).unwrap_or("example").to_string(),
            input: string_at(item, &["input"]).unwrap_or("unknown").to_string(),
        })
        .collect()
}

fn instruction_only_missing(cwd: &Path, manifest_path: &Path) -> Vec<String> {
    let action_path = cwd.join("action.py");
    let mut missing = Vec::new();
    if !action_path.is_file() {
        missing.push("missing action.py".to_string());
    }
    if !manifest_path.is_file() {
        missing.push("missing .skillrun/manifest.generated.yaml".to_string());
    }
    missing
}

fn invalid_manifest_freshness(reason: &str) -> &'static str {
    if reason.contains("stale Manifest") {
        "stale"
    } else {
        "invalid"
    }
}

fn display_path(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn preflight_status(cwd: &Path, entrypoint: &str) -> String {
    let action_path = cwd.join(entrypoint);
    match fs::read_to_string(&action_path) {
        Ok(text) if has_preflight(&text, entrypoint) => "present".to_string(),
        Ok(_) => "absent".to_string(),
        Err(_) => format!("source missing ({})", action_path.display()),
    }
}

fn has_preflight(source: &str, entrypoint: &str) -> bool {
    if entrypoint.ends_with(".py") {
        return source.contains("def preflight");
    }
    if entrypoint.ends_with(".mjs") || entrypoint.ends_with(".js") {
        return source.contains("export function preflight")
            || source.contains("export async function preflight")
            || source.contains("export const preflight");
    }
    source.contains("preflight")
}
