use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::schemas::Schemas;

#[derive(Debug, Serialize)]
pub struct CapsuleConfig {
    pub runtime: RuntimeConfig,
    pub permissions: ManifestPermissions,
    #[serde(skip_serializing)]
    pub schemas: Option<Schemas>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeConfig {
    pub adapter: String,
    pub entrypoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    pub timeout: String,
    pub requirements: RuntimeRequirements,
}

#[derive(Debug, Serialize)]
pub struct RuntimeRequirements {
    pub executable: ExecutableRequirement,
    pub packages: Vec<PackageRequirement>,
}

#[derive(Debug, Serialize)]
pub struct ExecutableRequirement {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct PackageRequirement {
    pub name: String,
    pub version: String,
    pub required_for: Vec<String>,
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

pub fn load_config(path: &Path) -> Result<CapsuleConfig, String> {
    if !path.exists() {
        return Ok(CapsuleConfig {
            runtime: default_runtime_config(),
            permissions: default_permissions(),
            schemas: None,
        });
    }

    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    let runtime = json.get("runtime").unwrap_or(&serde_json::Value::Null);

    let adapter = runtime
        .get("adapter")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("python")
        .to_string();

    let timeout = runtime
        .get("timeout")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("30s")
        .to_string();

    let (runtime, schemas) = if adapter == "command" {
        let command = parse_command(runtime.get("command"))?;
        let entrypoint = runtime
            .get("entrypoint")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| command_entrypoint(&command));
        let executable = command
            .first()
            .expect("parse_command should reject empty command")
            .clone();
        (
            RuntimeConfig {
                adapter,
                entrypoint,
                command: Some(command),
                protocol_version: Some("adapter.v1".to_string()),
                timeout,
                requirements: runtime_requirements_for_command(&executable),
            },
            Some(parse_static_schemas(&json)?),
        )
    } else {
        (
            RuntimeConfig {
                entrypoint: runtime
                    .get("entrypoint")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("action.py")
                    .to_string(),
                command: None,
                protocol_version: None,
                timeout,
                requirements: runtime_requirements_for_adapter(&adapter),
                adapter,
            },
            None,
        )
    };

    Ok(CapsuleConfig {
        runtime,
        permissions: parse_permissions(json.get("permissions"))?,
        schemas,
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

fn parse_permissions(value: Option<&serde_json::Value>) -> Result<ManifestPermissions, String> {
    let defaults = default_permissions();
    let Some(value) = value else {
        return Ok(defaults);
    };
    if value.is_null() {
        return Ok(defaults);
    }

    Ok(ManifestPermissions {
        files: FilePermissions {
            read: string_array_at(value, &["files", "read"], defaults.files.read)?,
            write: string_array_at(value, &["files", "write"], defaults.files.write)?,
        },
        network: NetworkPermissions {
            outbound: string_array_at(value, &["network", "outbound"], defaults.network.outbound)?,
        },
        env: EnvPermissions {
            read: string_array_at(value, &["env", "read"], defaults.env.read)?,
        },
    })
}

fn string_array_at(
    value: &serde_json::Value,
    path: &[&str],
    default: Vec<String>,
) -> Result<Vec<String>, String> {
    let Some(value) = json_value_at(value, path) else {
        return Ok(default);
    };
    if value.is_null() {
        return Ok(default);
    }
    let values = value
        .as_array()
        .ok_or_else(|| format!("permissions.{} must be an array", path.join(".")))?;
    values
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("permissions.{} must contain only strings", path.join(".")))
        })
        .collect()
}

fn json_value_at<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn default_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        adapter: "python".to_string(),
        entrypoint: "action.py".to_string(),
        command: None,
        protocol_version: None,
        timeout: "30s".to_string(),
        requirements: runtime_requirements_for_adapter("python"),
    }
}

pub fn runtime_requirements_for_adapter(adapter: &str) -> RuntimeRequirements {
    match adapter {
        "node" => RuntimeRequirements {
            executable: ExecutableRequirement {
                name: "node".to_string(),
                version: ">=18".to_string(),
            },
            packages: Vec::new(),
        },
        "python" => RuntimeRequirements {
            executable: ExecutableRequirement {
                name: "python".to_string(),
                version: ">=3.10".to_string(),
            },
            packages: vec![PackageRequirement {
                name: "pydantic".to_string(),
                version: ">=2,<3".to_string(),
                required_for: vec!["metadata".to_string(), "runtime".to_string()],
            }],
        },
        "command" => RuntimeRequirements {
            executable: ExecutableRequirement {
                name: "command".to_string(),
                version: "present".to_string(),
            },
            packages: Vec::new(),
        },
        other => RuntimeRequirements {
            executable: ExecutableRequirement {
                name: other.to_string(),
                version: "unsupported".to_string(),
            },
            packages: Vec::new(),
        },
    }
}

pub fn runtime_requirements_for_command(executable: &str) -> RuntimeRequirements {
    RuntimeRequirements {
        executable: ExecutableRequirement {
            name: executable.to_string(),
            version: "present".to_string(),
        },
        packages: Vec::new(),
    }
}

fn parse_command(value: Option<&serde_json::Value>) -> Result<Vec<String>, String> {
    let Some(value) = value else {
        return Err("runtime.command is required when runtime.adapter is command".to_string());
    };
    if value.is_string() {
        return Err(
            "runtime.command must be an array of strings; shell strings are not supported"
                .to_string(),
        );
    }
    let values = value
        .as_array()
        .ok_or_else(|| "runtime.command must be an array of strings".to_string())?;
    if values.is_empty() {
        return Err("runtime.command must not be empty".to_string());
    }
    values
        .iter()
        .map(|item| {
            item.as_str()
                .filter(|part| !part.trim().is_empty())
                .map(str::to_string)
                .ok_or_else(|| "runtime.command must contain only non-empty strings".to_string())
        })
        .collect()
}

fn command_entrypoint(command: &[String]) -> String {
    command
        .iter()
        .skip(1)
        .find(|part| !part.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| command[0].clone())
}

fn parse_static_schemas(json: &serde_json::Value) -> Result<Schemas, String> {
    let input = json
        .get("input_schema")
        .cloned()
        .ok_or_else(|| "input_schema is required when runtime.adapter is command".to_string())?;
    let output = json
        .get("output_schema")
        .cloned()
        .ok_or_else(|| "output_schema is required when runtime.adapter is command".to_string())?;
    ensure_schema_object(&input, "input_schema")?;
    ensure_schema_object(&output, "output_schema")?;
    Ok(Schemas { input, output })
}

fn ensure_schema_object(value: &serde_json::Value, name: &str) -> Result<(), String> {
    if value.as_object().is_some() {
        Ok(())
    } else {
        Err(format!("{name} must be a JSON Schema object"))
    }
}
