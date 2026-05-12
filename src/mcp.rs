use serde_json::{json, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::Path;

use crate::consumer::ValidManifest;

pub fn dry_run_contract(capsule_dir: &Path, manifest: &ValidManifest) -> Result<String, String> {
    let skill_name = string_at(&manifest.value, &["skill", "name"]).unwrap_or("skill");
    let tool_name = string_at(&manifest.value, &["tool", "name"]).unwrap_or(skill_name);
    let tool_description =
        string_at(&manifest.value, &["tool", "description"]).unwrap_or("SkillRun MCP tool.");
    let input_schema =
        json_value_at(&manifest.value, &["schemas", "input"]).unwrap_or_else(|| json!({}));
    let output_schema =
        json_value_at(&manifest.value, &["schemas", "output"]).unwrap_or_else(|| json!({}));
    let skill_path =
        string_at(&manifest.value, &["sources", "skill", "path"]).unwrap_or("SKILL.md");
    let skill_text = fs::read_to_string(capsule_dir.join(skill_path))
        .map_err(|error| format!("failed to read MCP resource {skill_path}: {error}"))?;

    let contract = json!({
        "mcp": {
            "dry_run": true,
            "transport": "stdio",
            "protocol": "model-context-protocol"
        },
        "source": {
            "manifest": manifest.path.display().to_string(),
            "manifest_sha256": manifest.sha256
        },
        "tools": [
            {
                "name": tool_name,
                "description": tool_description,
                "input_schema": input_schema,
                "output_schema": output_schema,
                "result_contract": "SkillRun output/error envelope"
            }
        ],
        "resources": [
            {
                "uri": format!("skillrun://{skill_name}/SKILL.md"),
                "name": "SKILL.md",
                "path": skill_path,
                "mime_type": "text/markdown",
                "text": skill_text
            }
        ]
    });

    serde_json::to_string_pretty(&contract)
        .map_err(|error| format!("failed to serialize MCP dry-run contract: {error}"))
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

fn json_value_at(value: &YamlValue, path: &[&str]) -> Option<JsonValue> {
    let value = value_at(value, path)?;
    serde_json::to_value(value).ok()
}
