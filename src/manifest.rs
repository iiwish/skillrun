use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters::python;
use crate::config::{self, ManifestPermissions, RuntimeConfig};
use crate::hashing;
use crate::schemas::Schemas;

#[derive(Debug)]
pub struct ManifestOptions {
    pub cwd: PathBuf,
}

#[derive(Debug, Serialize)]
struct ManifestDocument {
    manifest_version: String,
    generated_by: String,
    generated_at: String,
    sources: Sources,
    skill: SkillInfo,
    tool: ToolInfo,
    schemas: Schemas,
    runtime: RuntimeConfig,
    permissions: ManifestPermissions,
    ipc: IpcInfo,
    examples: Vec<ExampleInfo>,
    artifacts: ArtifactInfo,
    errors: ErrorInfo,
}

#[derive(Debug, Serialize)]
struct Sources {
    skill: SourceInfo,
    action: SourceInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<SourceInfo>,
}

#[derive(Debug, Serialize)]
struct SourceInfo {
    path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct SkillInfo {
    name: String,
    sop_summary: String,
    skill_hash: String,
}

#[derive(Debug, Serialize)]
struct ToolInfo {
    name: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct IpcInfo {
    protocol_version: String,
}

#[derive(Debug, Serialize)]
struct ExampleInfo {
    id: String,
    input: String,
}

#[derive(Debug, Serialize)]
struct ArtifactInfo {
    allowed_kinds: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ErrorInfo {
    envelope: bool,
}

pub fn generate(options: &ManifestOptions) -> Result<PathBuf, String> {
    let capsule_dir = options.cwd.clone();
    let skill_path = capsule_dir.join("SKILL.md");
    let action_path = capsule_dir.join("action.py");
    let config_path = capsule_dir.join("skillrun.config.json");

    require_file(&skill_path, "missing SKILL.md")?;
    require_file(&action_path, "missing action.py")?;

    let schemas = python::extract_schemas(&capsule_dir, &action_path)?;
    let runtime = config::load_runtime_config(&config_path)?;
    let skill_hash = hashing::sha256_file(&skill_path)?;
    let action_hash = hashing::sha256_file(&action_path)?;
    let config_source = if config_path.exists() {
        Some(SourceInfo {
            path: "skillrun.config.json".to_string(),
            sha256: hashing::sha256_file(&config_path)?,
        })
    } else {
        None
    };
    let name = capsule_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("skill")
        .to_string();

    let manifest = ManifestDocument {
        manifest_version: "0.1.0".to_string(),
        generated_by: format!("skillrun@{}", env!("CARGO_PKG_VERSION")),
        generated_at: Utc::now().to_rfc3339(),
        sources: Sources {
            skill: SourceInfo {
                path: "SKILL.md".to_string(),
                sha256: skill_hash.clone(),
            },
            action: SourceInfo {
                path: "action.py".to_string(),
                sha256: action_hash,
            },
            config: config_source,
        },
        skill: SkillInfo {
            name: name.clone(),
            sop_summary: "Generated from SKILL.md. Inspect support lands in T004.".to_string(),
            skill_hash,
        },
        tool: ToolInfo {
            name: name.clone(),
            description: format!("Execute the {name} SkillRun capsule."),
        },
        schemas,
        runtime,
        permissions: config::default_permissions(),
        ipc: IpcInfo {
            protocol_version: "0.1.0".to_string(),
        },
        examples: vec![ExampleInfo {
            id: "default".to_string(),
            input: "examples/default.input.json".to_string(),
        }],
        artifacts: ArtifactInfo {
            allowed_kinds: ["json", "markdown", "html", "pdf", "text", "file"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        },
        errors: ErrorInfo { envelope: true },
    };

    let manifest_dir = capsule_dir.join(".skillrun");
    fs::create_dir_all(&manifest_dir).map_err(|error| {
        format!(
            "failed to create manifest directory {}: {error}",
            manifest_dir.display()
        )
    })?;
    let manifest_path = manifest_dir.join("manifest.generated.yaml");
    let yaml = serde_yaml::to_string(&manifest)
        .map_err(|error| format!("failed to serialize manifest: {error}"))?;
    fs::write(&manifest_path, yaml)
        .map_err(|error| format!("failed to write {}: {error}", manifest_path.display()))?;

    Ok(manifest_path)
}

fn require_file(path: &Path, message: &str) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("{message}: {}", path.display()))
    }
}
