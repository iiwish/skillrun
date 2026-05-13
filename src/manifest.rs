use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters;
use crate::config::{self, ManifestPermissions, RuntimeConfig};
use crate::hashing;
use crate::schemas::Schemas;

#[derive(Debug)]
pub struct ManifestOptions {
    pub cwd: PathBuf,
}

pub fn generated_manifest_path(capsule_dir: &Path) -> PathBuf {
    capsule_dir
        .join(".skillrun")
        .join("manifest.generated.yaml")
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
    let config_path = capsule_dir.join("skillrun.config.json");

    require_file(&skill_path, "missing SKILL.md")?;
    let config = resolve_capsule_config(&capsule_dir, &config_path)?;
    let action_entrypoint = config.runtime.entrypoint.clone();
    let adapter = config.runtime.adapter.clone();
    ensure_supported_entrypoint(&action_entrypoint)?;
    let action_path = capsule_dir.join(&action_entrypoint);
    if !action_path.is_file() {
        return Err(format!(
            "missing {action_entrypoint}: {}. SkillRun does not infer actions from Markdown, scripts, references, assets, or examples; add an explicit {action_entrypoint} before running `skillrun manifest`.",
            action_path.display(),
        ));
    }

    let schemas = adapters::extract_schemas(&adapter, &capsule_dir, &action_path)?;
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
                path: action_entrypoint,
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
        runtime: config.runtime,
        permissions: config.permissions,
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
    let manifest_path = generated_manifest_path(&capsule_dir);
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

fn resolve_capsule_config(
    capsule_dir: &Path,
    config_path: &Path,
) -> Result<config::CapsuleConfig, String> {
    if config_path.exists() {
        return config::load_config(config_path);
    }

    Ok(config::CapsuleConfig {
        runtime: convention_runtime(capsule_dir)?,
        permissions: config::default_permissions(),
    })
}

fn convention_runtime(capsule_dir: &Path) -> Result<RuntimeConfig, String> {
    let known_actions = [
        ("action.py", "python"),
        ("action.mjs", "node"),
        ("action.ts", "typescript"),
    ];
    let found: Vec<(&str, &str)> = known_actions
        .into_iter()
        .filter(|(path, _)| capsule_dir.join(path).is_file())
        .collect();

    match found.as_slice() {
        [] => Err(format!(
            "missing action.py or action.mjs: {}. SkillRun does not infer actions from Markdown, scripts, references, assets, or examples; add an explicit action.py or action.mjs before running `skillrun manifest`.",
            capsule_dir.display()
        )),
        [(entrypoint, "typescript")] => Err(unsupported_typescript_message(entrypoint)),
        [(entrypoint, adapter)] => Ok(RuntimeConfig {
            adapter: (*adapter).to_string(),
            entrypoint: (*entrypoint).to_string(),
            timeout: "30s".to_string(),
            requirements: config::runtime_requirements_for_adapter(adapter),
        }),
        _ => {
            let names = found
                .iter()
                .map(|(path, _)| *path)
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "ambiguous action files without skillrun.config.json: found {names}. Add skillrun.config.json with runtime.adapter and runtime.entrypoint before running `skillrun manifest`."
            ))
        }
    }
}

fn ensure_supported_entrypoint(entrypoint: &str) -> Result<(), String> {
    if Path::new(entrypoint)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("ts"))
    {
        return Err(unsupported_typescript_message(entrypoint));
    }

    Ok(())
}

fn unsupported_typescript_message(entrypoint: &str) -> String {
    format!(
        "{entrypoint} is not supported in v0.3 JS alpha. compile to action.mjs and set runtime.entrypoint to action.mjs before running `skillrun manifest`."
    )
}
