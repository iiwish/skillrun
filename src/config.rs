use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct RuntimeConfig {
    pub adapter: String,
    pub entrypoint: String,
    pub timeout: String,
}

#[derive(Debug, Serialize)]
pub struct ManifestPermissions {
    pub files: FilePermissions,
    pub network: NetworkPermissions,
    pub env: EnvPermissions,
}

#[derive(Debug, Serialize)]
pub struct FilePermissions {
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct NetworkPermissions {
    pub outbound: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EnvPermissions {
    pub read: Vec<String>,
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, String> {
    if !path.exists() {
        return Ok(default_runtime_config());
    }

    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    let runtime = json.get("runtime").unwrap_or(&serde_json::Value::Null);

    Ok(RuntimeConfig {
        adapter: runtime
            .get("adapter")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("python")
            .to_string(),
        entrypoint: runtime
            .get("entrypoint")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("action.py")
            .to_string(),
        timeout: runtime
            .get("timeout")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("30s")
            .to_string(),
    })
}

pub fn default_permissions() -> ManifestPermissions {
    ManifestPermissions {
        files: FilePermissions {
            read: Vec::new(),
            write: vec![".skillrun/runs/**".to_string()],
        },
        network: NetworkPermissions {
            outbound: Vec::new(),
        },
        env: EnvPermissions { read: Vec::new() },
    }
}

fn default_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        adapter: "python".to_string(),
        entrypoint: "action.py".to_string(),
        timeout: "30s".to_string(),
    }
}
