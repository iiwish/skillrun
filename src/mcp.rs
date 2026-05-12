use serde_json::{json, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::consumer::ValidManifest;

const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

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

pub fn serve_stdio(_manifest: &ValidManifest) -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve_stdio_io(stdin.lock(), stdout.lock())
}

fn serve_stdio_io<R, W>(reader: R, mut writer: W) -> Result<(), String>
where
    R: BufRead,
    W: Write,
{
    for line in reader.lines() {
        let line = line.map_err(|error| format!("failed to read MCP stdin: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<JsonValue>(&line) {
            Ok(message) => handle_message(message),
            Err(error) => Some(error_response(
                JsonValue::Null,
                -32700,
                format!("Parse error: {error}"),
            )),
        };

        if let Some(response) = response {
            write_json_line(&mut writer, &response)?;
        }
    }

    Ok(())
}

fn handle_message(message: JsonValue) -> Option<JsonValue> {
    let Some(object) = message.as_object() else {
        return Some(error_response(
            JsonValue::Null,
            -32600,
            "Invalid Request: expected JSON object",
        ));
    };

    let id = object.get("id").cloned();
    let method = object.get("method").and_then(JsonValue::as_str);

    match method {
        Some("initialize") => Some(success_response(
            id.unwrap_or(JsonValue::Null),
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {
                    "tools": {},
                    "resources": {}
                },
                "serverInfo": {
                    "name": "skillrun",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),
        Some("notifications/initialized") => None,
        Some(method) => {
            let id = id.unwrap_or(JsonValue::Null);
            if id.is_null() {
                None
            } else {
                Some(error_response(
                    id,
                    -32601,
                    format!("Method not found: {method}"),
                ))
            }
        }
        None => Some(error_response(
            id.unwrap_or(JsonValue::Null),
            -32600,
            "Invalid Request: missing method",
        )),
    }
}

fn success_response(id: JsonValue, result: JsonValue) -> JsonValue {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(
    id: JsonValue,
    code: i64,
    message: impl Into<String>,
) -> JsonValue {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into()
        }
    })
}

fn write_json_line(writer: &mut impl Write, value: &JsonValue) -> Result<(), String> {
    serde_json::to_writer(&mut *writer, value)
        .map_err(|error| format!("failed to serialize MCP response: {error}"))?;
    writer
        .write_all(b"\n")
        .map_err(|error| format!("failed to write MCP response: {error}"))?;
    writer
        .flush()
        .map_err(|error| format!("failed to flush MCP stdout: {error}"))
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
