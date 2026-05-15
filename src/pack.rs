use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tar::Builder;

use crate::consumer::{self, ValidManifest};
use crate::manifest_access::string_at;

#[derive(Debug)]
pub struct PackOptions {
    pub cwd: PathBuf,
}

pub fn create(options: &PackOptions) -> Result<String, String> {
    let capsule_dir = absolute_path(&options.cwd)?;
    require_dir(&capsule_dir)?;
    let manifest = consumer::validate(&capsule_dir, "skillrun pack")?;
    let skill_name = string_at(&manifest.value, &["skill", "name"]).unwrap_or_else(|| {
        capsule_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("skill")
    });
    let archive_stem = validate_archive_stem(skill_name)?;
    let archive_name = format!("{archive_stem}-{}.skr", env!("CARGO_PKG_VERSION"));
    let dist_dir = capsule_dir.join("dist");
    fs::create_dir_all(&dist_dir).map_err(|error| {
        format!(
            "failed to create package directory {}: {error}",
            dist_dir.display()
        )
    })?;

    let archive_path = dist_dir.join(&archive_name);
    let temp_path = dist_dir.join(format!(".{archive_name}.tmp"));
    write_archive(&capsule_dir, &manifest, &temp_path)?;
    if archive_path.exists() {
        fs::remove_file(&archive_path).map_err(|error| {
            format!(
                "failed to replace existing package {}: {error}",
                archive_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, &archive_path).map_err(|error| {
        format!(
            "failed to move package {} to {}: {error}",
            temp_path.display(),
            archive_path.display()
        )
    })?;

    Ok(format!(
        "\
created {archive}
format: tar.gz .skr
contents: source files, examples, and .skillrun/manifest.generated.yaml
note: .skr does not vendor dependencies; recreate the runtime environment separately.",
        archive = archive_path.display()
    ))
}

fn write_archive(
    capsule_dir: &Path,
    manifest: &ValidManifest,
    temp_path: &Path,
) -> Result<(), String> {
    let file = fs::File::create(temp_path)
        .map_err(|error| format!("failed to create {}: {error}", temp_path.display()))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

    let skill_path =
        string_at(&manifest.value, &["sources", "skill", "path"]).unwrap_or("SKILL.md");
    let action_path =
        string_at(&manifest.value, &["sources", "action", "path"]).unwrap_or("action.py");
    append_file(&mut builder, capsule_dir, Path::new(skill_path))?;
    append_file(&mut builder, capsule_dir, Path::new(action_path))?;

    if let Some(config_path) = string_at(&manifest.value, &["sources", "config", "path"]) {
        append_file(&mut builder, capsule_dir, Path::new(config_path))?;
    }

    append_directory_files(&mut builder, capsule_dir, Path::new("examples"))?;
    append_file(
        &mut builder,
        capsule_dir,
        Path::new(".skillrun")
            .join("manifest.generated.yaml")
            .as_path(),
    )?;

    builder
        .finish()
        .map_err(|error| format!("failed to finish package archive: {error}"))?;
    let encoder = builder
        .into_inner()
        .map_err(|error| format!("failed to finish gzip stream: {error}"))?;
    encoder
        .finish()
        .map_err(|error| format!("failed to write gzip stream: {error}"))?;

    Ok(())
}

fn append_directory_files(
    builder: &mut Builder<GzEncoder<fs::File>>,
    capsule_dir: &Path,
    relative_dir: &Path,
) -> Result<(), String> {
    let directory = capsule_dir.join(relative_dir);
    if !directory.exists() {
        return Ok(());
    }
    if !directory.is_dir() {
        return Err(format!(
            "package source is not a directory: {}",
            directory.display()
        ));
    }

    let mut files = Vec::new();
    collect_files(&directory, &mut files)?;
    files.sort();
    for file in files {
        let relative = file.strip_prefix(capsule_dir).map_err(|error| {
            format!(
                "failed to normalize package path {}: {error}",
                file.display()
            )
        })?;
        append_file(builder, capsule_dir, relative)?;
    }

    Ok(())
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn append_file(
    builder: &mut Builder<GzEncoder<fs::File>>,
    capsule_dir: &Path,
    relative_path: &Path,
) -> Result<(), String> {
    let source = capsule_dir.join(relative_path);
    if !source.is_file() {
        return Err(format!(
            "package source file is missing: {}",
            source.display()
        ));
    }
    let archive_path = normalize_archive_path(relative_path)?;
    builder
        .append_path_with_name(&source, archive_path)
        .map_err(|error| format!("failed to add {} to package: {error}", source.display()))
}

fn normalize_archive_path(relative_path: &Path) -> Result<String, String> {
    if relative_path.is_absolute() {
        return Err(format!(
            "package path must be relative: {}",
            relative_path.display()
        ));
    }
    let mut components = Vec::new();
    for component in relative_path.components() {
        match component {
            Component::Normal(value) => components.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            _ => {
                return Err(format!(
                    "package path escapes capsule: {}",
                    relative_path.display()
                ));
            }
        }
    }
    let normalized = components.join("/");
    if normalized.is_empty() || normalized.starts_with("../") || normalized.contains("/../") {
        return Err(format!(
            "package path escapes capsule: {}",
            relative_path.display()
        ));
    }
    Ok(normalized)
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

fn validate_archive_stem(name: &str) -> Result<&str, String> {
    let is_valid = !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if is_valid {
        Ok(name)
    } else {
        Err(format!(
            "invalid package name from Manifest: {name}. Use only ASCII letters, numbers, '-' or '_'"
        ))
    }
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
