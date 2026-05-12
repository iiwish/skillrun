use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct RunRecord<'a> {
    pub run_id: &'a str,
    pub mode: &'a str,
    pub status: &'a str,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u128,
    pub capsule_dir: String,
    pub manifest_path: String,
    pub manifest_sha256: &'a str,
    pub skill_sha256: &'a str,
    pub action_sha256: &'a str,
    pub permissions: serde_json::Value,
    pub input: &'a str,
    pub context: &'a str,
    pub output: &'a str,
    pub stdout: &'a str,
    pub stderr: &'a str,
    pub artifacts: &'a str,
}

pub struct RunRecordInput<'a> {
    pub run_id: &'a str,
    pub mode: &'a str,
    pub status: &'a str,
    pub started_at: chrono::DateTime<Utc>,
    pub finished_at: chrono::DateTime<Utc>,
    pub duration: Duration,
    pub capsule_dir: &'a Path,
    pub manifest_path: &'a Path,
    pub manifest_sha256: &'a str,
    pub skill_sha256: &'a str,
    pub action_sha256: &'a str,
    pub permissions: serde_json::Value,
}

pub fn write(path: &Path, input: RunRecordInput<'_>) -> Result<(), String> {
    let record = RunRecord {
        run_id: input.run_id,
        mode: input.mode,
        status: input.status,
        started_at: input.started_at.to_rfc3339(),
        finished_at: input.finished_at.to_rfc3339(),
        duration_ms: input.duration.as_millis(),
        capsule_dir: input.capsule_dir.display().to_string(),
        manifest_path: input.manifest_path.display().to_string(),
        manifest_sha256: input.manifest_sha256,
        skill_sha256: input.skill_sha256,
        action_sha256: input.action_sha256,
        permissions: input.permissions,
        input: "input.json",
        context: "context.json",
        output: "output.json",
        stdout: "stdout.log",
        stderr: "stderr.log",
        artifacts: "artifacts",
    };
    let json = serde_json::to_string_pretty(&record)
        .map_err(|error| format!("failed to serialize run record: {error}"))?;
    fs::write(path, json).map_err(|error| format!("failed to write {}: {error}", path.display()))
}
