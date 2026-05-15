use std::path::Path;
use std::time::Duration;

use crate::schemas::Schemas;

pub mod command;
pub mod node;
mod process;
pub mod python;

pub struct RuntimeDiscovery {
    pub executable: DiscoveredDependency,
    pub packages: Vec<DiscoveredDependency>,
}

pub struct DiscoveredDependency {
    pub name: String,
    pub detected: Option<String>,
    pub available: bool,
}

pub struct ActionRunRequest<'a> {
    pub capsule_dir: &'a Path,
    pub entrypoint: &'a str,
    pub command: Option<&'a [String]>,
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
        "node" => node::extract_schemas(capsule_dir, action_path),
        "python" => python::extract_schemas(capsule_dir, action_path),
        _ => Err(format!("unsupported metadata adapter: {adapter}")),
    }
}

pub fn ensure_runtime_adapter(adapter: &str) -> Result<(), String> {
    match adapter {
        "command" => Ok(()),
        "node" => Ok(()),
        "python" => Ok(()),
        _ => Err(format!("unsupported runtime adapter: {adapter}")),
    }
}

pub fn discover_runtime(adapter: &str) -> Option<RuntimeDiscovery> {
    match adapter {
        "node" => Some(node::discover_runtime()),
        "python" => Some(python::discover_runtime()),
        _ => None,
    }
}

pub fn run_action(
    adapter: &str,
    request: &ActionRunRequest<'_>,
) -> Result<ActionRunOutput, String> {
    match adapter {
        "command" => command::run_action(request),
        "node" => node::run_action(request),
        "python" => python::run_action(request),
        _ => Err(format!("unsupported runtime adapter: {adapter}")),
    }
}
