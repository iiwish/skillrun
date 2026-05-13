use std::path::Path;
use std::time::Duration;

use crate::schemas::Schemas;

pub mod python;

pub struct ActionRunRequest<'a> {
    pub capsule_dir: &'a Path,
    pub entrypoint: &'a str,
    pub context_json: &'a Path,
    pub input_json: &'a Path,
    pub output_json: &'a Path,
    pub artifact_dir: &'a Path,
    pub env: &'a [(String, String)],
    pub timeout: Duration,
}

pub struct ActionRunOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn extract_schemas(
    adapter: &str,
    capsule_dir: &Path,
    action_path: &Path,
) -> Result<Schemas, String> {
    match adapter {
        "python" => python::extract_schemas(capsule_dir, action_path),
        _ => Err(format!("unsupported metadata adapter: {adapter}")),
    }
}

pub fn ensure_runtime_adapter(adapter: &str) -> Result<(), String> {
    match adapter {
        "python" => Ok(()),
        _ => Err(format!("unsupported runtime adapter: {adapter}")),
    }
}

pub fn run_action(
    adapter: &str,
    request: &ActionRunRequest<'_>,
) -> Result<ActionRunOutput, String> {
    match adapter {
        "python" => python::run_action(request),
        _ => Err(format!("unsupported runtime adapter: {adapter}")),
    }
}
