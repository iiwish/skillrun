use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::hashing;
use crate::manifest;
use crate::manifest_access::{string_at, value_at, ManifestView};
use crate::schemas;

#[derive(Clone)]
pub struct ValidManifest {
    pub value: Value,
    pub path: PathBuf,
    pub sha256: String,
}

pub fn validate(cwd: &Path, command: &str) -> Result<ValidManifest, String> {
    let capsule_dir = absolute_path(cwd)?;
    require_dir(&capsule_dir)?;

    let manifest_path = manifest::generated_manifest_path(&capsule_dir);
    if !manifest_path.is_file() {
        return Err(missing_manifest_error(
            &capsule_dir,
            &manifest_path,
            command,
        ));
    }

    let text = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let value: Value = serde_yaml::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;

    validate_source(&capsule_dir, &value, command, "skill")?;
    validate_source(&capsule_dir, &value, command, "action")?;
    match value_at(&value, &["sources", "config"]) {
        Some(Value::Mapping(_)) => validate_source(&capsule_dir, &value, command, "config")?,
        _ if capsule_dir.join("skillrun.config.json").is_file() => {
            return Err(format!(
                "{command} refused stale Manifest: skillrun.config.json exists but Manifest does not record its source hash. Run `skillrun manifest --cwd {}` before using Consumer Mode.",
                capsule_dir.display()
            ));
        }
        _ => {}
    }
    validate_manifest_contract(&value)?;

    let sha256 = hashing::sha256_file(&manifest_path)?;
    Ok(ValidManifest {
        value,
        path: manifest_path,
        sha256,
    })
}

fn validate_manifest_contract(manifest: &Value) -> Result<(), String> {
    let manifest_view = ManifestView::new(manifest);
    manifest_view
        .runtime_adapter()
        .ok_or_else(|| "invalid Manifest: missing runtime.adapter".to_string())?;
    manifest_view
        .runtime_entrypoint()
        .ok_or_else(|| "invalid Manifest: missing runtime.entrypoint".to_string())?;

    let input_schema = manifest_view
        .input_schema_json()
        .ok_or_else(|| "invalid Manifest: missing schemas.input".to_string())?;
    schemas::validate_schema_contract(&input_schema)
        .map_err(|error| format!("invalid Manifest: schemas.input {error}"))?;

    let output_schema = manifest_view
        .output_schema_json()
        .ok_or_else(|| "invalid Manifest: missing schemas.output".to_string())?;
    schemas::validate_schema_contract(&output_schema)
        .map_err(|error| format!("invalid Manifest: schemas.output {error}"))
}

fn validate_source(
    capsule_dir: &Path,
    manifest: &Value,
    command: &str,
    source_key: &str,
) -> Result<(), String> {
    let source_path_key = ["sources", source_key, "path"];
    let hash_key = ["sources", source_key, "sha256"];
    let source_path = string_at(manifest, &source_path_key).ok_or_else(|| {
        format!(
            "{command} refused stale Manifest: missing source path for {source_key}. Run `skillrun manifest --cwd {}` before using Consumer Mode.",
            capsule_dir.display()
        )
    })?;
    let expected = string_at(manifest, &hash_key).ok_or_else(|| {
        format!(
            "{command} refused stale Manifest: missing source hash for {source_path}. Run `skillrun manifest --cwd {}` before using Consumer Mode.",
            capsule_dir.display()
        )
    })?;
    let source_file = capsule_dir.join(source_path);
    let actual = hashing::sha256_file(&source_file).map_err(|error| {
        format!(
            "{command} refused stale Manifest: source listed by Manifest is missing or unreadable: {source_path}: {error}. Restore the explicit source file, then run `skillrun manifest --cwd {}`.",
            capsule_dir.display()
        )
    })?;
    if actual != expected {
        return Err(format!(
            "{command} refused stale Manifest: source hash mismatch for {source_path}. Run `skillrun manifest --cwd {}` before using Consumer Mode.",
            capsule_dir.display()
        ));
    }

    Ok(())
}

fn missing_manifest_error(capsule_dir: &Path, manifest_path: &Path, command: &str) -> String {
    let skill_path = capsule_dir.join("SKILL.md");
    let python_action_path = capsule_dir.join("action.py");
    let node_action_path = capsule_dir.join("action.mjs");
    let typescript_action_path = capsule_dir.join("action.ts");
    let has_python_action = python_action_path.is_file();
    let has_node_action = node_action_path.is_file();
    let has_typescript_action = typescript_action_path.is_file();

    if skill_path.is_file() && !has_python_action && !has_node_action && !has_typescript_action {
        return format!(
            "{command} refused instruction-only Skill at {}: missing action.py or action.mjs and {}. Add an explicit action.py or action.mjs, then run `skillrun manifest --cwd {}`. SkillRun does not infer actions from Markdown, scripts, references, assets, or examples.",
            capsule_dir.display(),
            manifest_path.display(),
            capsule_dir.display()
        );
    }

    if has_typescript_action && !has_python_action && !has_node_action {
        return format!(
            "{command} refused unsupported TypeScript action at {}: action.ts is not supported in v0.3 JS alpha. compile to action.mjs, then run `skillrun manifest --cwd {}`.",
            capsule_dir.display(),
            capsule_dir.display()
        );
    }

    if !has_python_action && !has_node_action {
        return format!(
            "{command} refused non-runnable SkillRun directory at {}: missing action.py or action.mjs. Add an explicit action.py or action.mjs, then run `skillrun manifest --cwd {}`.",
            capsule_dir.display(),
            capsule_dir.display()
        );
    }

    format!(
        "{command} refused Consumer Mode for {}: missing Manifest at {}. Run `skillrun manifest --cwd {}` before using Consumer Mode.",
        capsule_dir.display(),
        manifest_path.display(),
        capsule_dir.display()
    )
}

fn require_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("cwd does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("cwd is not a directory: {}", path.display()));
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| format!("failed to resolve current directory: {error}"))
    }
}
