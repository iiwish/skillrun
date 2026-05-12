use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest;

#[derive(Debug)]
pub struct InspectOptions {
    pub cwd: PathBuf,
}

pub fn render(options: &InspectOptions) -> Result<String, String> {
    let cwd = options.cwd.clone();
    if !cwd.exists() {
        return Err(format!("cwd does not exist: {}", cwd.display()));
    }
    if !cwd.is_dir() {
        return Err(format!("cwd is not a directory: {}", cwd.display()));
    }

    let manifest_path = manifest::generated_manifest_path(&cwd);
    if manifest_path.is_file() {
        render_runnable(&cwd, &manifest_path)
    } else {
        render_instruction_only(&cwd, &manifest_path)
    }
}

fn render_runnable(cwd: &Path, manifest_path: &Path) -> Result<String, String> {
    let manifest_text = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest: Value = serde_yaml::from_str(&manifest_text)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;

    let fallback_name = cwd
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("skill");
    let name = string_at(&manifest, &["skill", "name"]).unwrap_or(fallback_name);
    let sop_hash = string_at(&manifest, &["sources", "skill", "sha256"])
        .or_else(|| string_at(&manifest, &["skill", "skill_hash"]))
        .unwrap_or("unknown");
    let action_hash = string_at(&manifest, &["sources", "action", "sha256"]).unwrap_or("unknown");
    let input_schema = presence(value_at(&manifest, &["schemas", "input"]));
    let output_schema = presence(value_at(&manifest, &["schemas", "output"]));
    let adapter = string_at(&manifest, &["runtime", "adapter"]).unwrap_or("unknown");
    let entrypoint = string_at(&manifest, &["runtime", "entrypoint"]).unwrap_or("action.py");
    let timeout = string_at(&manifest, &["runtime", "timeout"]).unwrap_or("unknown");
    let file_reads = strings_at(&manifest, &["permissions", "files", "read"]);
    let file_writes = strings_at(&manifest, &["permissions", "files", "write"]);
    let network = strings_at(&manifest, &["permissions", "network", "outbound"]);
    let env_reads = strings_at(&manifest, &["permissions", "env", "read"]);
    let examples = examples(&manifest);
    let tool_name = string_at(&manifest, &["tool", "name"]).unwrap_or(name);
    let tool_description = string_at(&manifest, &["tool", "description"]).unwrap_or("unknown");
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

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        let key = Value::String((*segment).to_string());
        current = current.as_mapping()?.get(&key)?;
    }
    Some(current)
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    value_at(value, path)?.as_str()
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

fn preflight_status(cwd: &Path, entrypoint: &str) -> String {
    let action_path = cwd.join(entrypoint);
    match fs::read_to_string(&action_path) {
        Ok(text) if text.contains("def preflight") => "present".to_string(),
        Ok(_) => "absent".to_string(),
        Err(_) => format!("source missing ({})", action_path.display()),
    }
}
