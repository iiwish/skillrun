use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub struct MountPlanOptions {
    pub client: String,
    pub config: Option<PathBuf>,
    pub json: bool,
}

pub struct MountPlanOutput {
    pub output: String,
}

#[derive(Debug, Serialize)]
struct MountPlanView {
    command: &'static str,
    schema_version: &'static str,
    client: ClientView,
    operation: &'static str,
    config: Option<ConfigView>,
    backup: Option<BackupView>,
    router: RouterView,
    changes: Vec<ChangeView>,
    warnings: Vec<WarningView>,
}

#[derive(Debug, Serialize)]
struct ClientView {
    id: String,
    name: String,
    supported: bool,
    detected: bool,
}

#[derive(Debug, Serialize)]
struct ConfigView {
    path: String,
    path_source: &'static str,
    exists: bool,
    parseable: bool,
}

#[derive(Debug, Serialize)]
struct BackupView {
    path: String,
    required_before_apply: bool,
}

#[derive(Debug, Serialize)]
struct RouterView {
    server_name: &'static str,
    command: &'static str,
    args: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct ChangeView {
    kind: &'static str,
    server_name: &'static str,
    before: Option<Value>,
    after: Value,
}

#[derive(Debug, Serialize)]
struct WarningView {
    code: &'static str,
    message: String,
}

struct ClientSpec {
    id: &'static str,
    name: &'static str,
    default_config: PathBuf,
}

pub fn plan(options: &MountPlanOptions) -> Result<MountPlanOutput, String> {
    let view = build_plan(options);
    if options.json {
        return Ok(MountPlanOutput {
            output: serde_json::to_string_pretty(&view).map_err(|error| error.to_string())?,
        });
    }

    let output = if let Some(config) = &view.config {
        format!(
            "SkillRun Mount Plan\nclient: {}\nconfig: {}\nchanges: {}",
            view.client.id,
            config.path,
            view.changes.len()
        )
    } else {
        format!(
            "SkillRun Mount Plan\nclient: {}\nsupported: false\nchanges: 0",
            view.client.id
        )
    };
    Ok(MountPlanOutput { output })
}

fn build_plan(options: &MountPlanOptions) -> MountPlanView {
    let router = RouterView {
        server_name: "skillrun",
        command: "skillrun",
        args: vec!["router", "serve", "--mcp"],
    };

    let Some(spec) = client_spec(&options.client) else {
        return MountPlanView {
            command: "consumer mount plan",
            schema_version: "consumer.mount_plan.v1",
            client: ClientView {
                id: options.client.clone(),
                name: options.client.clone(),
                supported: false,
                detected: false,
            },
            operation: "install_or_update_router",
            config: None,
            backup: None,
            router,
            changes: Vec::new(),
            warnings: vec![WarningView {
                code: "unsupported-client",
                message: "client is not supported by mount planning".to_string(),
            }],
        };
    };

    let path_source = if options.config.is_some() {
        "override"
    } else {
        "default"
    };
    let config_path = options
        .config
        .clone()
        .unwrap_or_else(|| spec.default_config.clone());
    let exists = config_path.is_file();
    let mut warnings = Vec::new();
    warnings.push(WarningView {
        code: "router-runtime-not-implemented",
        message: "plan targets the future SkillRun Router; v0.5.6 does not start or install it"
            .to_string(),
    });

    let (parseable, changes) = plan_changes(&config_path, exists, &router, &mut warnings);
    let config_path_display = display_path(&config_path);
    MountPlanView {
        command: "consumer mount plan",
        schema_version: "consumer.mount_plan.v1",
        client: ClientView {
            id: spec.id.to_string(),
            name: spec.name.to_string(),
            supported: true,
            detected: exists,
        },
        operation: "install_or_update_router",
        config: Some(ConfigView {
            path: config_path_display.clone(),
            path_source,
            exists,
            parseable,
        }),
        backup: Some(BackupView {
            path: format!("{config_path_display}.skillrun.bak"),
            required_before_apply: true,
        }),
        router,
        changes,
        warnings,
    }
}

fn plan_changes(
    config_path: &Path,
    exists: bool,
    router: &RouterView,
    warnings: &mut Vec<WarningView>,
) -> (bool, Vec<ChangeView>) {
    let after = router_entry(router);
    if !exists {
        return (
            true,
            vec![ChangeView {
                kind: "upsert_mcp_server",
                server_name: "skillrun",
                before: None,
                after,
            }],
        );
    }

    let config = match read_json(config_path) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(WarningView {
                code: "unparseable-config",
                message: error,
            });
            return (false, Vec::new());
        }
    };

    let Some(root) = config.as_object() else {
        warnings.push(WarningView {
            code: "invalid-config-root",
            message: "config root must be a JSON object before SkillRun can plan a patch"
                .to_string(),
        });
        return (false, Vec::new());
    };
    let before = match root.get("mcpServers") {
        None => None,
        Some(servers) => {
            let Some(servers) = servers.as_object() else {
                warnings.push(WarningView {
                    code: "invalid-mcp-servers",
                    message: "mcpServers must be a JSON object before SkillRun can plan a patch"
                        .to_string(),
                });
                return (false, Vec::new());
            };
            servers.get("skillrun").map(server_entry_summary)
        }
    };

    if before.as_ref() == Some(&after) {
        return (true, Vec::new());
    }

    (
        true,
        vec![ChangeView {
            kind: "upsert_mcp_server",
            server_name: "skillrun",
            before,
            after,
        }],
    )
}

fn client_spec(client: &str) -> Option<ClientSpec> {
    match client {
        "claude-desktop" => Some(ClientSpec {
            id: "claude-desktop",
            name: "Claude Desktop",
            default_config: appdata_path(&["Claude", "claude_desktop_config.json"])
                .unwrap_or_else(|| PathBuf::from("claude_desktop_config.json")),
        }),
        "cursor" => Some(ClientSpec {
            id: "cursor",
            name: "Cursor",
            default_config: home_path(&[".cursor", "mcp.json"])
                .unwrap_or_else(|| PathBuf::from("mcp.json")),
        }),
        _ => None,
    }
}

fn router_entry(router: &RouterView) -> Value {
    json!({
        "command": router.command,
        "args": router.args,
    })
}

fn server_entry_summary(value: &Value) -> Value {
    json!({
        "command": value.get("command").and_then(Value::as_str),
        "args": value.get("args").and_then(Value::as_array).cloned().unwrap_or_default(),
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("config exists but is not valid JSON: {error}"))
}

fn appdata_path(parts: &[&str]) -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|root| join_parts(PathBuf::from(root), parts))
}

fn home_path(parts: &[&str]) -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|root| join_parts(PathBuf::from(root), parts))
}

fn join_parts(mut root: PathBuf, parts: &[&str]) -> PathBuf {
    for part in parts {
        root.push(part);
    }
    root
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
