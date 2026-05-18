use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct MountBackupFile {
    pub(super) schema_version: String,
    pub(super) created_by: String,
    pub(super) id: String,
    pub(super) client_id: String,
    pub(super) config_path: String,
    pub(super) router_entry: Value,
    pub(super) original_exists: bool,
    pub(super) original_config: Option<Value>,
}

pub(super) fn read_backup_file(path: &Path) -> Result<MountBackupFile, String> {
    let value = super::read_json(path)?;
    serde_json::from_value(value)
        .map_err(|error| format!("backup is not a valid SkillRun mount backup: {error}"))
}

pub(super) fn backup_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{millis}-{}", std::process::id())
}

pub(super) fn backup_path_for(config_path: &Path, backup_id: &str) -> PathBuf {
    let file_name = config_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("mcp_config.json");
    config_path.with_file_name(format!("{file_name}.skillrun.{backup_id}.bak.json"))
}

pub(super) fn backup_path_preview_for(config_path: &Path) -> PathBuf {
    backup_path_for(config_path, "<backup-id>")
}
