use serde_json::{json, Value as JsonValue};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Component, Path, PathBuf};

use crate::consumer::ValidManifest;
use crate::manifest_access::{string_at, ManifestView};
use crate::runtime;

const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

#[derive(Clone)]
pub struct McpRoute {
    pub capsule_id: String,
    pub capsule_dir: PathBuf,
    pub manifest: ValidManifest,
}

pub fn dry_run_contract(capsule_dir: &Path, manifest: &ValidManifest) -> Result<String, String> {
    let manifest_view = ManifestView::new(&manifest.value);
    let skill_name = manifest_view.skill_name().unwrap_or("skill");
    let tool_name = manifest_view.tool_name().unwrap_or(skill_name);
    let tool_description = manifest_view
        .tool_description()
        .unwrap_or("SkillRun MCP tool.");
    let input_schema = manifest_view
        .input_schema_json()
        .unwrap_or_else(|| json!({}));
    let output_schema = manifest_view
        .output_schema_json()
        .unwrap_or_else(|| json!({}));
    let skill_path = manifest_view.source_path("skill").unwrap_or("SKILL.md");
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

pub fn router_dry_run_contract(routes: &[McpRoute]) -> Result<String, String> {
    let tools = routes
        .iter()
        .map(|route| {
            let manifest_view = ManifestView::new(&route.manifest.value);
            let skill_name = manifest_view.skill_name().unwrap_or("skill");
            let tool_name = manifest_view.tool_name().unwrap_or(skill_name);
            let tool_description = manifest_view
                .tool_description()
                .unwrap_or("SkillRun MCP tool.");
            let input_schema = manifest_view
                .input_schema_json()
                .unwrap_or_else(|| json!({}));
            let output_schema = manifest_view
                .output_schema_json()
                .unwrap_or_else(|| json!({}));
            json!({
                "capsule_id": route.capsule_id,
                "capsule_path": route.capsule_dir.display().to_string(),
                "name": tool_name,
                "description": tool_description,
                "input_schema": input_schema,
                "output_schema": output_schema,
                "manifest_sha256": route.manifest.sha256,
                "result_contract": "SkillRun output/error envelope"
            })
        })
        .collect::<Vec<_>>();
    let resources = routes
        .iter()
        .flat_map(|route| {
            resource_registry(&route.capsule_dir, &route.manifest)
                .into_iter()
                .map(|resource| {
                    json!({
                        "capsule_id": route.capsule_id,
                        "uri": router_resource_uri(route, &resource),
                        "name": resource.name,
                        "path": resource.relative_path,
                        "mime_type": resource.mime_type
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let contract = json!({
        "command": "router serve --mcp",
        "schema_version": "router.mcp.v1",
        "mcp": {
            "dry_run": true,
            "transport": "stdio",
            "protocol": "model-context-protocol"
        },
        "router": {
            "snapshot": true,
            "capsules": routes.len()
        },
        "tools": tools,
        "resources": resources
    });

    serde_json::to_string_pretty(&contract)
        .map_err(|error| format!("failed to serialize Router dry-run contract: {error}"))
}

pub fn serve_stdio(capsule_dir: &Path, manifest: &ValidManifest) -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve_stdio_io(stdin.lock(), stdout.lock(), capsule_dir, manifest)
}

pub fn serve_router_stdio(routes: &[McpRoute]) -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve_router_stdio_io(stdin.lock(), stdout.lock(), routes)
}

fn serve_stdio_io<R, W>(
    reader: R,
    mut writer: W,
    capsule_dir: &Path,
    manifest: &ValidManifest,
) -> Result<(), String>
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
            Ok(message) => handle_message(message, capsule_dir, manifest),
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

fn handle_message(
    message: JsonValue,
    capsule_dir: &Path,
    manifest: &ValidManifest,
) -> Option<JsonValue> {
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
        Some("tools/list") => Some(success_response(
            id.unwrap_or(JsonValue::Null),
            tools_list_result(manifest),
        )),
        Some("tools/call") => {
            let id = id.unwrap_or(JsonValue::Null);
            Some(handle_tools_call(id, object, capsule_dir, manifest))
        }
        Some("resources/list") => Some(success_response(
            id.unwrap_or(JsonValue::Null),
            resources_list_result(capsule_dir, manifest),
        )),
        Some("resources/read") => {
            let id = id.unwrap_or(JsonValue::Null);
            Some(handle_resources_read(id, object, capsule_dir, manifest))
        }
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

fn serve_router_stdio_io<R, W>(reader: R, mut writer: W, routes: &[McpRoute]) -> Result<(), String>
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
            Ok(message) => handle_router_message(message, routes),
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

fn handle_router_message(message: JsonValue, routes: &[McpRoute]) -> Option<JsonValue> {
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
                    "name": "skillrun-router",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),
        Some("notifications/initialized") => None,
        Some("tools/list") => Some(success_response(
            id.unwrap_or(JsonValue::Null),
            router_tools_list_result(routes),
        )),
        Some("tools/call") => {
            let id = id.unwrap_or(JsonValue::Null);
            Some(handle_router_tools_call(id, object, routes))
        }
        Some("resources/list") => Some(success_response(
            id.unwrap_or(JsonValue::Null),
            router_resources_list_result(routes),
        )),
        Some("resources/read") => {
            let id = id.unwrap_or(JsonValue::Null);
            Some(handle_router_resources_read(id, object, routes))
        }
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

#[derive(Clone)]
struct McpResource {
    uri: String,
    name: String,
    relative_path: String,
    mime_type: String,
}

fn resource_registry(capsule_dir: &Path, manifest: &ValidManifest) -> Vec<McpResource> {
    let manifest_view = ManifestView::new(&manifest.value);
    let skill_name = manifest_view.skill_name().unwrap_or("skill");
    let mut resources = Vec::new();

    let skill_path = manifest_view.source_path("skill").unwrap_or("SKILL.md");
    if let Some(resource) = resource_for_path(skill_name, "SKILL.md", skill_path, "text/markdown") {
        resources.push(resource);
    }

    if let Some(examples) = manifest_view.examples() {
        for example in examples {
            let Some(input_path) = string_at(example, &["input"]) else {
                continue;
            };
            let name = input_path
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(input_path)
                .to_string();
            if let Some(resource) =
                resource_for_path(skill_name, &name, input_path, "application/json")
            {
                resources.push(resource);
            }
        }
    }

    resources
        .into_iter()
        .filter(|resource| {
            safe_relative_path(&resource.relative_path)
                .map(|path| capsule_dir.join(path).is_file())
                .unwrap_or(false)
        })
        .collect()
}

fn resource_for_path(
    skill_name: &str,
    name: &str,
    relative_path: &str,
    mime_type: &str,
) -> Option<McpResource> {
    let safe_path = safe_relative_path(relative_path).ok()?;
    let uri_path = safe_path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    Some(McpResource {
        uri: format!("skillrun://{skill_name}/{uri_path}"),
        name: name.to_string(),
        relative_path: uri_path,
        mime_type: mime_type.to_string(),
    })
}

fn resources_list_result(capsule_dir: &Path, manifest: &ValidManifest) -> JsonValue {
    let resources = resource_registry(capsule_dir, manifest)
        .into_iter()
        .map(|resource| {
            json!({
                "uri": resource.uri,
                "name": resource.name,
                "mimeType": resource.mime_type
            })
        })
        .collect::<Vec<_>>();

    json!({ "resources": resources })
}

fn router_resources_list_result(routes: &[McpRoute]) -> JsonValue {
    let resources = routes
        .iter()
        .flat_map(|route| {
            resource_registry(&route.capsule_dir, &route.manifest)
                .into_iter()
                .map(|resource| {
                    json!({
                        "uri": router_resource_uri(route, &resource),
                        "name": resource.name,
                        "mimeType": resource.mime_type
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    json!({ "resources": resources })
}

fn handle_resources_read(
    id: JsonValue,
    request: &serde_json::Map<String, JsonValue>,
    capsule_dir: &Path,
    manifest: &ValidManifest,
) -> JsonValue {
    let Some(params) = request.get("params").and_then(JsonValue::as_object) else {
        return error_response(id, -32602, "Invalid params: resources/read requires params");
    };
    let Some(uri) = params.get("uri").and_then(JsonValue::as_str) else {
        return error_response(id, -32602, "Invalid params: resources/read requires uri");
    };

    let Some(resource) = resource_registry(capsule_dir, manifest)
        .into_iter()
        .find(|resource| resource.uri == uri)
    else {
        return error_response(id, -32002, format!("Resource not found: {uri}"));
    };

    let path = match safe_relative_path(&resource.relative_path) {
        Ok(path) => capsule_dir.join(path),
        Err(error) => return error_response(id, -32002, error),
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            return error_response(
                id,
                -32002,
                format!("Failed to read resource {uri}: {error}"),
            );
        }
    };

    success_response(
        id,
        json!({
            "contents": [
                {
                    "uri": resource.uri,
                    "mimeType": resource.mime_type,
                    "text": text
                }
            ]
        }),
    )
}

fn handle_router_resources_read(
    id: JsonValue,
    request: &serde_json::Map<String, JsonValue>,
    routes: &[McpRoute],
) -> JsonValue {
    let Some(params) = request.get("params").and_then(JsonValue::as_object) else {
        return error_response(id, -32602, "Invalid params: resources/read requires params");
    };
    let Some(uri) = params.get("uri").and_then(JsonValue::as_str) else {
        return error_response(id, -32602, "Invalid params: resources/read requires uri");
    };

    for route in routes {
        if let Some(resource) = resource_registry(&route.capsule_dir, &route.manifest)
            .into_iter()
            .find(|resource| router_resource_uri(route, resource) == uri)
        {
            let path = match safe_relative_path(&resource.relative_path) {
                Ok(path) => route.capsule_dir.join(path),
                Err(error) => return error_response(id, -32002, error),
            };
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(error) => {
                    return error_response(
                        id,
                        -32002,
                        format!("Failed to read resource {uri}: {error}"),
                    );
                }
            };

            return success_response(
                id,
                json!({
                    "contents": [
                        {
                            "uri": resource.uri,
                            "mimeType": resource.mime_type,
                            "text": text
                        }
                    ]
                }),
            );
        }
    }

    error_response(id, -32002, format!("Resource not found: {uri}"))
}

fn router_resource_uri(route: &McpRoute, resource: &McpResource) -> String {
    format!(
        "skillrun://router/{}/{}",
        route.capsule_id, resource.relative_path
    )
}

fn safe_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!("resource path must be relative: {value}"));
    }

    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => safe.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!("resource path escapes capsule: {value}"));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("resource path must stay inside capsule: {value}"));
            }
        }
    }

    if safe.as_os_str().is_empty() {
        return Err("resource path must not be empty".to_string());
    }

    Ok(safe)
}

fn tools_list_result(manifest: &ValidManifest) -> JsonValue {
    let manifest_view = ManifestView::new(&manifest.value);
    let skill_name = manifest_view.skill_name().unwrap_or("skill");
    let tool_name = manifest_view.tool_name().unwrap_or(skill_name);
    let tool_description = manifest_view
        .tool_description()
        .unwrap_or("SkillRun MCP tool.");
    let input_schema = manifest_view
        .input_schema_json()
        .unwrap_or_else(|| json!({}));
    let output_schema = manifest_view
        .output_schema_json()
        .unwrap_or_else(|| json!({}));

    json!({
        "tools": [
            {
                "name": tool_name,
                "description": tool_description,
                "inputSchema": input_schema,
                "outputSchema": output_schema
            }
        ]
    })
}

fn router_tools_list_result(routes: &[McpRoute]) -> JsonValue {
    let tools = routes
        .iter()
        .map(|route| {
            let manifest_view = ManifestView::new(&route.manifest.value);
            let skill_name = manifest_view.skill_name().unwrap_or("skill");
            let tool_name = manifest_view.tool_name().unwrap_or(skill_name);
            let tool_description = manifest_view
                .tool_description()
                .unwrap_or("SkillRun MCP tool.");
            let input_schema = manifest_view
                .input_schema_json()
                .unwrap_or_else(|| json!({}));
            let output_schema = manifest_view
                .output_schema_json()
                .unwrap_or_else(|| json!({}));

            json!({
                "name": tool_name,
                "description": tool_description,
                "inputSchema": input_schema,
                "outputSchema": output_schema
            })
        })
        .collect::<Vec<_>>();

    json!({ "tools": tools })
}

fn handle_tools_call(
    id: JsonValue,
    request: &serde_json::Map<String, JsonValue>,
    capsule_dir: &Path,
    manifest: &ValidManifest,
) -> JsonValue {
    let Some(params) = request.get("params").and_then(JsonValue::as_object) else {
        return error_response(id, -32602, "Invalid params: tools/call requires params");
    };
    let Some(name) = params.get("name").and_then(JsonValue::as_str) else {
        return error_response(id, -32602, "Invalid params: tools/call requires name");
    };

    let manifest_view = ManifestView::new(&manifest.value);
    let skill_name = manifest_view.skill_name().unwrap_or("skill");
    let tool_name = manifest_view.tool_name().unwrap_or(skill_name);
    if name != tool_name {
        return error_response(id, -32602, format!("Unknown tool: {name}"));
    }

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !arguments.is_object() {
        return error_response(id, -32602, "Invalid params: arguments must be an object");
    }

    match runtime::run_with_json_input(capsule_dir, arguments, "mcp") {
        Ok(outcome) => match serde_json::from_str::<JsonValue>(&outcome.envelope) {
            Ok(envelope) => success_response(id, tool_call_result(&envelope, outcome.success)),
            Err(error) => error_response(
                id,
                -32603,
                format!("SkillRun envelope was not valid JSON: {error}"),
            ),
        },
        Err(error) => error_response(id, -32603, error),
    }
}

fn handle_router_tools_call(
    id: JsonValue,
    request: &serde_json::Map<String, JsonValue>,
    routes: &[McpRoute],
) -> JsonValue {
    let Some(params) = request.get("params").and_then(JsonValue::as_object) else {
        return error_response(id, -32602, "Invalid params: tools/call requires params");
    };
    let Some(name) = params.get("name").and_then(JsonValue::as_str) else {
        return error_response(id, -32602, "Invalid params: tools/call requires name");
    };

    let Some(route) = routes.iter().find(|route| route_tool_name(route) == name) else {
        return error_response(id, -32602, format!("Unknown tool: {name}"));
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !arguments.is_object() {
        return error_response(id, -32602, "Invalid params: arguments must be an object");
    }

    match runtime::run_with_json_input(&route.capsule_dir, arguments, "mcp") {
        Ok(outcome) => match serde_json::from_str::<JsonValue>(&outcome.envelope) {
            Ok(envelope) => success_response(id, tool_call_result(&envelope, outcome.success)),
            Err(error) => error_response(
                id,
                -32603,
                format!("SkillRun envelope was not valid JSON: {error}"),
            ),
        },
        Err(error) => error_response(id, -32603, error),
    }
}

pub fn route_tool_name(route: &McpRoute) -> String {
    let manifest_view = ManifestView::new(&route.manifest.value);
    let skill_name = manifest_view.skill_name().unwrap_or("skill");
    manifest_view.tool_name().unwrap_or(skill_name).to_string()
}

fn tool_call_result(envelope: &JsonValue, success: bool) -> JsonValue {
    json!({
        "content": [
            {
                "type": "text",
                "text": tool_call_text(envelope, success)
            }
        ],
        "isError": !success
    })
}

fn tool_call_text(envelope: &JsonValue, success: bool) -> String {
    if success {
        let display = envelope
            .get("display")
            .and_then(|display| display.get("markdown"))
            .and_then(JsonValue::as_str);
        let output = envelope.get("output").or_else(|| envelope.get("result"));
        match (display, output) {
            (Some(display), Some(output)) => {
                format!("{display}\n\n{}", pretty_json(output))
            }
            (Some(display), None) => display.to_string(),
            (None, Some(output)) => pretty_json(output),
            (None, None) => "SkillRun tool completed.".to_string(),
        }
    } else {
        let error = envelope.get("error");
        let code = error
            .and_then(|error| error.get("code"))
            .and_then(JsonValue::as_str)
            .unwrap_or("RuntimeError");
        let message = error
            .and_then(|error| error.get("message"))
            .and_then(JsonValue::as_str)
            .unwrap_or("SkillRun tool failed.");
        let hint = error
            .and_then(|error| error.get("llm_hint"))
            .and_then(JsonValue::as_str);
        match hint {
            Some(hint) => format!("{code}: {message}\n\nllm_hint: {hint}"),
            None => format!("{code}: {message}"),
        }
    }
}

fn pretty_json(value: &JsonValue) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn success_response(id: JsonValue, result: JsonValue) -> JsonValue {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: JsonValue, code: i64, message: impl Into<String>) -> JsonValue {
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
