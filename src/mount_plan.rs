mod backup;
mod client;

use backup::{
    backup_id, backup_path_for, backup_path_preview_for, read_backup_file, MountBackupFile,
};
use client::{client_spec, selected_config_path, ClientSpec};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub struct MountPlanOptions {
    pub client: String,
    pub config: Option<PathBuf>,
    pub json: bool,
}

pub struct MountApplyOptions {
    pub client: String,
    pub config: Option<PathBuf>,
    pub json: bool,
}

pub struct MountRollbackOptions {
    pub client: String,
    pub config: Option<PathBuf>,
    pub backup: PathBuf,
    pub json: bool,
}

pub struct MountPlanOutput {
    pub output: String,
}

pub struct MountApplyOutput {
    pub output: String,
}

pub struct MountRollbackOutput {
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
struct MountApplyView {
    command: &'static str,
    schema_version: &'static str,
    client: ClientView,
    config: Option<ApplyConfigView>,
    backup: Option<ApplyBackupView>,
    applied: bool,
    changes: Vec<ChangeView>,
    warnings: Vec<WarningView>,
}

#[derive(Debug, Serialize)]
struct MountRollbackView {
    command: &'static str,
    schema_version: &'static str,
    client: ClientView,
    config: Option<RollbackConfigView>,
    backup: Option<RollbackBackupView>,
    rolled_back: bool,
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
struct ApplyConfigView {
    path: String,
    path_source: &'static str,
    exists_before: bool,
    written: bool,
}

#[derive(Debug, Serialize)]
struct RollbackConfigView {
    path: String,
    written: bool,
}

#[derive(Debug, Serialize)]
struct BackupView {
    path: String,
    required_before_apply: bool,
}

#[derive(Debug, Serialize)]
struct ApplyBackupView {
    id: Option<String>,
    path: Option<String>,
    created: bool,
}

#[derive(Debug, Serialize)]
struct RollbackBackupView {
    id: String,
    path: String,
    consumed: bool,
}

#[derive(Debug, Serialize)]
struct RouterView {
    server_name: &'static str,
    command: &'static str,
    args: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
struct ChangeView {
    kind: &'static str,
    server_name: &'static str,
    before: Option<Value>,
    after: Value,
}

#[derive(Debug, Clone, Serialize)]
struct WarningView {
    code: &'static str,
    message: String,
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

pub fn apply(options: &MountApplyOptions) -> Result<MountApplyOutput, String> {
    let view = build_apply(options)?;
    if options.json {
        return Ok(MountApplyOutput {
            output: serde_json::to_string_pretty(&view).map_err(|error| error.to_string())?,
        });
    }

    let output = format!(
        "SkillRun Mount Apply\nclient: {}\napplied: {}\nchanges: {}",
        view.client.id,
        view.applied,
        view.changes.len()
    );
    Ok(MountApplyOutput { output })
}

pub fn rollback(options: &MountRollbackOptions) -> Result<MountRollbackOutput, String> {
    let view = build_rollback(options)?;
    if options.json {
        return Ok(MountRollbackOutput {
            output: serde_json::to_string_pretty(&view).map_err(|error| error.to_string())?,
        });
    }

    let output = format!(
        "SkillRun Mount Rollback\nclient: {}\nrolled_back: {}",
        view.client.id, view.rolled_back
    );
    Ok(MountRollbackOutput { output })
}

fn build_plan(options: &MountPlanOptions) -> MountPlanView {
    let router = router_view();

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
    let config_path = selected_config_path(&options.config, &spec);
    let exists = config_path.is_file();
    let mut warnings = Vec::new();
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
            path: display_path(&backup_path_preview_for(&config_path)),
            required_before_apply: true,
        }),
        router,
        changes,
        warnings,
    }
}

fn build_apply(options: &MountApplyOptions) -> Result<MountApplyView, String> {
    let router = router_view();
    let Some(spec) = client_spec(&options.client) else {
        return Ok(apply_warning(
            &options.client,
            "unsupported-client",
            "client is not supported by mount apply",
        ));
    };
    if spec.id != "claude-desktop" {
        return Ok(apply_warning(
            spec.id,
            "unsupported-client",
            "client is plan-only in this release; mount apply supports claude-desktop only",
        ));
    }

    let plan_options = MountPlanOptions {
        client: options.client.clone(),
        config: options.config.clone(),
        json: true,
    };
    let plan = build_plan(&plan_options);
    let Some(config) = plan.config.as_ref() else {
        return Ok(apply_warning(
            spec.id,
            "unsupported-client",
            "client is not supported by mount apply",
        ));
    };
    if !config.parseable {
        return Ok(MountApplyView {
            command: "consumer mount apply",
            schema_version: "consumer.mount_apply.v1",
            client: plan.client,
            config: Some(ApplyConfigView {
                path: config.path.clone(),
                path_source: config.path_source,
                exists_before: config.exists,
                written: false,
            }),
            backup: Some(ApplyBackupView {
                id: None,
                path: None,
                created: false,
            }),
            applied: false,
            changes: Vec::new(),
            warnings: plan.warnings,
        });
    }
    if plan.changes.is_empty() {
        return Ok(MountApplyView {
            command: "consumer mount apply",
            schema_version: "consumer.mount_apply.v1",
            client: plan.client,
            config: Some(ApplyConfigView {
                path: config.path.clone(),
                path_source: config.path_source,
                exists_before: config.exists,
                written: false,
            }),
            backup: Some(ApplyBackupView {
                id: None,
                path: None,
                created: false,
            }),
            applied: false,
            changes: Vec::new(),
            warnings: plan.warnings,
        });
    }

    let config_path = selected_config_path(&options.config, &spec);
    let original_exists = config_path.is_file();
    let original_config = if original_exists {
        Some(read_json(&config_path)?)
    } else {
        None
    };
    let backup_id = backup_id();
    let backup_path = backup_path_for(&config_path, &backup_id);
    let backup_file = MountBackupFile {
        schema_version: "consumer.mount_backup.v1".to_string(),
        created_by: "skillrun".to_string(),
        id: backup_id.clone(),
        client_id: spec.id.to_string(),
        config_path: display_path(&config_path),
        router_entry: router_entry(&router),
        original_exists,
        original_config: original_config.clone(),
    };
    write_json_file(&backup_path, &backup_file)?;

    let mut next_config = original_config.unwrap_or_else(|| json!({}));
    upsert_skillrun_entry(&mut next_config, router_entry(&router))?;
    write_json_file(&config_path, &next_config)?;

    Ok(MountApplyView {
        command: "consumer mount apply",
        schema_version: "consumer.mount_apply.v1",
        client: plan.client,
        config: Some(ApplyConfigView {
            path: config.path.clone(),
            path_source: config.path_source,
            exists_before: config.exists,
            written: true,
        }),
        backup: Some(ApplyBackupView {
            id: Some(backup_id),
            path: Some(display_path(&backup_path)),
            created: true,
        }),
        applied: true,
        changes: plan.changes,
        warnings: plan.warnings,
    })
}

fn build_rollback(options: &MountRollbackOptions) -> Result<MountRollbackView, String> {
    let Some(spec) = client_spec(&options.client) else {
        return Ok(rollback_warning(
            &options.client,
            "unsupported-client",
            "client is not supported by mount rollback",
        ));
    };
    if spec.id != "claude-desktop" {
        return Ok(rollback_warning(
            spec.id,
            "unsupported-client",
            "client is plan-only in this release; mount rollback supports claude-desktop only",
        ));
    }

    if !options.backup.is_file() {
        return Ok(rollback_warning_for_spec(
            &spec,
            "missing-backup",
            "backup file does not exist; rollback did not modify config",
        ));
    }
    let backup = match read_backup_file(&options.backup) {
        Ok(backup) => backup,
        Err(error) => {
            return Ok(rollback_warning_for_spec(
                &spec,
                "invalid-backup",
                &format!("{error}; rollback did not modify config"),
            ));
        }
    };
    if backup.schema_version != "consumer.mount_backup.v1"
        || backup.created_by != "skillrun"
        || backup.client_id != spec.id
    {
        return Ok(rollback_warning_for_spec(
            &spec,
            "invalid-backup",
            "backup is not a SkillRun mount backup for this client",
        ));
    }

    let config_path = options
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from(&backup.config_path));
    if display_path(&config_path) != backup.config_path {
        return Ok(MountRollbackView {
            command: "consumer mount rollback",
            schema_version: "consumer.mount_rollback.v1",
            client: client_view(&spec, config_path.is_file()),
            config: Some(RollbackConfigView {
                path: display_path(&config_path),
                written: false,
            }),
            backup: Some(RollbackBackupView {
                id: backup.id,
                path: display_path(&options.backup),
                consumed: false,
            }),
            rolled_back: false,
            warnings: vec![WarningView {
                code: "backup-config-mismatch",
                message: "backup target config does not match the selected config path".to_string(),
            }],
        });
    }

    let current_exists = config_path.is_file();
    if !current_exists && !backup.original_exists {
        return Ok(MountRollbackView {
            command: "consumer mount rollback",
            schema_version: "consumer.mount_rollback.v1",
            client: client_view(&spec, false),
            config: Some(RollbackConfigView {
                path: display_path(&config_path),
                written: false,
            }),
            backup: Some(RollbackBackupView {
                id: backup.id,
                path: display_path(&options.backup),
                consumed: false,
            }),
            rolled_back: false,
            warnings: Vec::new(),
        });
    }

    let mut current = if current_exists {
        match read_json(&config_path) {
            Ok(config) => config,
            Err(error) => {
                return Ok(MountRollbackView {
                    command: "consumer mount rollback",
                    schema_version: "consumer.mount_rollback.v1",
                    client: client_view(&spec, true),
                    config: Some(RollbackConfigView {
                        path: display_path(&config_path),
                        written: false,
                    }),
                    backup: Some(RollbackBackupView {
                        id: backup.id,
                        path: display_path(&options.backup),
                        consumed: false,
                    }),
                    rolled_back: false,
                    warnings: vec![WarningView {
                        code: "unparseable-config",
                        message: format!("{error}; rollback did not modify config"),
                    }],
                });
            }
        }
    } else {
        json!({})
    };
    if current_exists && skillrun_entry_raw(&current) != Some(backup.router_entry.clone()) {
        return Ok(MountRollbackView {
            command: "consumer mount rollback",
            schema_version: "consumer.mount_rollback.v1",
            client: client_view(&spec, true),
            config: Some(RollbackConfigView {
                path: display_path(&config_path),
                written: false,
            }),
            backup: Some(RollbackBackupView {
                id: backup.id,
                path: display_path(&options.backup),
                consumed: false,
            }),
            rolled_back: false,
            warnings: vec![WarningView {
                code: "rollback-conflict",
                message:
                    "current skillrun MCP entry no longer matches the entry created by mount apply"
                        .to_string(),
            }],
        });
    }

    if let Err(error) =
        restore_original_skillrun_entry(&mut current, backup.original_config.as_ref())
    {
        return Ok(MountRollbackView {
            command: "consumer mount rollback",
            schema_version: "consumer.mount_rollback.v1",
            client: client_view(&spec, current_exists),
            config: Some(RollbackConfigView {
                path: display_path(&config_path),
                written: false,
            }),
            backup: Some(RollbackBackupView {
                id: backup.id,
                path: display_path(&options.backup),
                consumed: false,
            }),
            rolled_back: false,
            warnings: vec![WarningView {
                code: "unsafe-config-shape",
                message: format!("{error}; rollback did not modify config"),
            }],
        });
    }
    if should_remove_config_after_rollback(&current, &backup) {
        fs::remove_file(&config_path)
            .map_err(|error| format!("failed to remove {}: {error}", config_path.display()))?;
    } else {
        write_json_file(&config_path, &current)?;
    }

    Ok(MountRollbackView {
        command: "consumer mount rollback",
        schema_version: "consumer.mount_rollback.v1",
        client: client_view(&spec, config_path.is_file()),
        config: Some(RollbackConfigView {
            path: display_path(&config_path),
            written: true,
        }),
        backup: Some(RollbackBackupView {
            id: backup.id,
            path: display_path(&options.backup),
            consumed: false,
        }),
        rolled_back: true,
        warnings: Vec::new(),
    })
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

fn apply_warning(client: &str, code: &'static str, message: &str) -> MountApplyView {
    MountApplyView {
        command: "consumer mount apply",
        schema_version: "consumer.mount_apply.v1",
        client: ClientView {
            id: client.to_string(),
            name: client.to_string(),
            supported: false,
            detected: false,
        },
        config: None,
        backup: None,
        applied: false,
        changes: Vec::new(),
        warnings: vec![WarningView {
            code,
            message: message.to_string(),
        }],
    }
}

fn rollback_warning(client: &str, code: &'static str, message: &str) -> MountRollbackView {
    MountRollbackView {
        command: "consumer mount rollback",
        schema_version: "consumer.mount_rollback.v1",
        client: ClientView {
            id: client.to_string(),
            name: client.to_string(),
            supported: false,
            detected: false,
        },
        config: None,
        backup: None,
        rolled_back: false,
        warnings: vec![WarningView {
            code,
            message: message.to_string(),
        }],
    }
}

fn rollback_warning_for_spec(
    spec: &ClientSpec,
    code: &'static str,
    message: &str,
) -> MountRollbackView {
    MountRollbackView {
        command: "consumer mount rollback",
        schema_version: "consumer.mount_rollback.v1",
        client: client_view(spec, false),
        config: None,
        backup: None,
        rolled_back: false,
        warnings: vec![WarningView {
            code,
            message: message.to_string(),
        }],
    }
}

fn router_view() -> RouterView {
    RouterView {
        server_name: "skillrun",
        command: "skillrun",
        args: vec!["router", "serve", "--mcp"],
    }
}

fn client_view(spec: &ClientSpec, detected: bool) -> ClientView {
    ClientView {
        id: spec.id.to_string(),
        name: spec.name.to_string(),
        supported: true,
        detected,
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

fn skillrun_entry_raw(config: &Value) -> Option<Value> {
    config
        .get("mcpServers")
        .and_then(Value::as_object)
        .and_then(|servers| servers.get("skillrun"))
        .cloned()
}

fn original_had_mcp_servers(config: Option<&Value>) -> bool {
    config
        .and_then(|value| value.get("mcpServers"))
        .and_then(Value::as_object)
        .is_some()
}

fn upsert_skillrun_entry(config: &mut Value, entry: Value) -> Result<(), String> {
    let Some(root) = config.as_object_mut() else {
        return Err(
            "config root must be a JSON object before SkillRun can apply a patch".to_string(),
        );
    };
    if !root.contains_key("mcpServers") {
        root.insert("mcpServers".to_string(), json!({}));
    }
    let Some(servers) = root.get_mut("mcpServers").and_then(Value::as_object_mut) else {
        return Err(
            "mcpServers must be a JSON object before SkillRun can apply a patch".to_string(),
        );
    };
    servers.insert("skillrun".to_string(), entry);
    Ok(())
}

fn restore_original_skillrun_entry(
    current: &mut Value,
    original_config: Option<&Value>,
) -> Result<(), String> {
    let original_entry = original_config.and_then(skillrun_entry_raw);
    let original_had_servers = original_had_mcp_servers(original_config);
    let Some(root) = current.as_object_mut() else {
        return Err(
            "current config root must be a JSON object before SkillRun can rollback".to_string(),
        );
    };

    if !root.contains_key("mcpServers") {
        if original_entry.is_some() {
            root.insert("mcpServers".to_string(), json!({}));
        } else {
            return Ok(());
        }
    }

    let Some(servers) = root.get_mut("mcpServers").and_then(Value::as_object_mut) else {
        return Err(
            "current mcpServers must be a JSON object before SkillRun can rollback".to_string(),
        );
    };
    if let Some(entry) = original_entry {
        servers.insert("skillrun".to_string(), entry);
    } else {
        servers.remove("skillrun");
    }
    if servers.is_empty() && !original_had_servers {
        root.remove("mcpServers");
    }
    Ok(())
}

fn should_remove_config_after_rollback(current: &Value, backup: &MountBackupFile) -> bool {
    !backup.original_exists && current.as_object().is_some_and(|root| root.is_empty())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("config exists but is not valid JSON: {error}"))
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }
    let text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, format!("{text}\n"))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
