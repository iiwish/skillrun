use serde_json::Value;
use serde_yaml::Value as YamlValue;
use std::path::{Component, Path};

pub fn declared_env_values(manifest: &YamlValue) -> Vec<(String, String)> {
    declared_env_keys(manifest)
        .into_iter()
        .filter(|key| !key.is_empty())
        .filter_map(|key| std::env::var(&key).ok().map(|value| (key, value)))
        .collect()
}

pub fn validate_artifacts(envelope: &Value, artifact_dir: &Path) -> Result<(), String> {
    let Some(artifacts) = envelope.get("artifacts") else {
        return Ok(());
    };
    let artifacts = artifacts
        .as_array()
        .ok_or_else(|| "artifacts must be an array".to_string())?;
    let artifact_root = artifact_dir.canonicalize().map_err(|error| {
        format!(
            "SKILLRUN_ARTIFACT_DIR is not accessible at {}: {error}",
            artifact_dir.display()
        )
    })?;

    for (index, artifact) in artifacts.iter().enumerate() {
        let path = artifact
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("artifact[{index}].path must be a string"))?;
        validate_relative_artifact_path(path)
            .map_err(|error| format!("artifact[{index}].path {error}: {path}"))?;

        let artifact_path = artifact_dir.join(path);
        let artifact_path = artifact_path.canonicalize().map_err(|error| {
            format!(
                "artifact[{index}].path must point to an existing file inside SKILLRUN_ARTIFACT_DIR: {path}: {error}"
            )
        })?;
        if !artifact_path.starts_with(&artifact_root) {
            return Err(format!(
                "artifact[{index}].path resolves outside SKILLRUN_ARTIFACT_DIR: {path}"
            ));
        }
        if !artifact_path.is_file() {
            return Err(format!("artifact[{index}].path is not a file: {path}"));
        }
    }

    Ok(())
}

fn declared_env_keys(manifest: &YamlValue) -> Vec<String> {
    yaml_value_at(manifest, &["permissions", "env", "read"])
        .and_then(YamlValue::as_sequence)
        .map(|items| {
            items
                .iter()
                .filter_map(YamlValue::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn validate_relative_artifact_path(path: &str) -> Result<(), &'static str> {
    if path.trim().is_empty() {
        return Err("must not be empty");
    }

    let path = Path::new(path);
    if path.is_absolute() {
        return Err("must be relative");
    }

    let mut has_normal_component = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_normal_component = true,
            Component::CurDir => {}
            Component::ParentDir => return Err("must not contain parent directory traversal"),
            Component::Prefix(_) | Component::RootDir => {
                return Err("must not contain a drive prefix or root")
            }
        }
    }

    if has_normal_component {
        Ok(())
    } else {
        Err("must include a file name")
    }
}

fn yaml_value_at<'a>(value: &'a YamlValue, path: &[&str]) -> Option<&'a YamlValue> {
    let mut current = value;
    for segment in path {
        let key = YamlValue::String((*segment).to_string());
        current = current.as_mapping()?.get(&key)?;
    }
    Some(current)
}
