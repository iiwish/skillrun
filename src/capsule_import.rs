use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Archive;

use crate::consumer;
use crate::manifest_access::string_at;
use crate::registry;

#[derive(Debug)]
pub struct ImportOptions {
    pub package: PathBuf,
    pub id: Option<String>,
    pub target_dir: Option<PathBuf>,
    pub json: bool,
    pub replace: bool,
}

pub struct ImportOutput {
    pub output: String,
}

#[derive(Debug, Serialize)]
struct ImportView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    package_path: String,
    registry_path: String,
    capsule: ImportedCapsuleView,
    warnings: Vec<ImportWarningView>,
}

#[derive(Debug, Serialize)]
struct ImportErrorView {
    command: &'static str,
    schema_version: &'static str,
    ok: bool,
    error: ImportErrorDetailView,
    warnings: Vec<ImportWarningView>,
}

#[derive(Debug, Serialize)]
struct ImportErrorDetailView {
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct ImportedCapsuleView {
    id: String,
    path: String,
    source_type: String,
    enabled: bool,
    replaced: bool,
}

#[derive(Debug, Serialize)]
struct ImportWarningView {
    code: &'static str,
    message: &'static str,
}

pub fn error_json(message: &str) -> Result<String, String> {
    let view = ImportErrorView {
        command: "import",
        schema_version: "import.v1",
        ok: false,
        error: ImportErrorDetailView {
            code: import_error_code(message),
            message: message.to_string(),
        },
        warnings: Vec::new(),
    };
    serde_json::to_string_pretty(&view).map_err(|error| error.to_string())
}

pub fn run(options: &ImportOptions) -> Result<ImportOutput, String> {
    let package_path = absolute_existing_file(&options.package)?;
    let import_root = match &options.target_dir {
        Some(target_dir) => absolute_path(target_dir)?,
        None => default_import_root()?,
    };
    fs::create_dir_all(&import_root)
        .map_err(|error| format!("failed to create {}: {error}", import_root.display()))?;

    let temp_dir = import_root.join(format!(".import-{}-{}.tmp", std::process::id(), nonce()?));
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("failed to create {}: {error}", temp_dir.display()))?;

    let imported = import_from_package(options, &package_path, &import_root, &temp_dir);
    if imported.is_err() {
        fs::remove_dir_all(&temp_dir).ok();
    }
    imported
}

fn import_from_package(
    options: &ImportOptions,
    package_path: &Path,
    import_root: &Path,
    temp_dir: &Path,
) -> Result<ImportOutput, String> {
    extract_package(package_path, temp_dir)?;
    let manifest = consumer::validate(temp_dir, "skillrun import")?;
    let registry_id = match &options.id {
        Some(id) => id.clone(),
        None => string_at(&manifest.value, &["skill", "name"])
            .ok_or_else(|| "imported Manifest is missing skill.name".to_string())?
            .to_string(),
    };
    registry::validate_registry_id(&registry_id)?;
    if !options.replace {
        registry::ensure_registry_id_available(&registry_id)?;
    }

    let final_dir = import_root.join(&registry_id);
    if options.replace {
        return replace_imported_package(options, package_path, temp_dir, &registry_id, &final_dir);
    }

    if final_dir.exists() {
        return Err(format!(
            "import target already exists: {}",
            final_dir.display()
        ));
    }

    fs::rename(temp_dir, &final_dir).map_err(|error| {
        format!(
            "failed to move imported capsule {} to {}: {error}",
            temp_dir.display(),
            final_dir.display()
        )
    })?;

    let registered = registry::register_capsule_with_source(
        &final_dir,
        Some(&registry_id),
        registry::IMPORTED_SKR_SOURCE_TYPE,
    );
    if let Err(error) = registered {
        fs::remove_dir_all(&final_dir).ok();
        return Err(error);
    }

    let registry_path = registry::registry_path_display()?;
    let source_type = registry::IMPORTED_SKR_SOURCE_TYPE.to_string();
    let view = ImportView {
        command: "import",
        schema_version: "import.v1",
        ok: true,
        package_path: display_path(package_path),
        registry_path,
        capsule: ImportedCapsuleView {
            id: registry_id.clone(),
            path: display_path(&final_dir),
            source_type,
            enabled: false,
            replaced: false,
        },
        warnings: import_warnings(),
    };

    if options.json {
        return Ok(ImportOutput {
            output: serde_json::to_string_pretty(&view).map_err(|error| error.to_string())?,
        });
    }

    Ok(ImportOutput {
        output: format!(
            "imported {id}\npath: {path}\nenabled: false\nnote: .skr import does not install dependencies or mark the capsule trusted.",
            id = registry_id,
            path = final_dir.display()
        ),
    })
}

fn replace_imported_package(
    options: &ImportOptions,
    package_path: &Path,
    temp_dir: &Path,
    registry_id: &str,
    final_dir: &Path,
) -> Result<ImportOutput, String> {
    let existing = registry::imported_capsule_for_replace(registry_id)?;
    let requested_path = if final_dir.exists() {
        fs::canonicalize(final_dir)
            .map_err(|error| format!("failed to resolve {}: {error}", final_dir.display()))?
    } else {
        final_dir.to_path_buf()
    };
    if existing.path != requested_path {
        return Err(format!(
            "import replace target mismatch for {registry_id}: existing path is {}; requested target is {}",
            existing.path.display(),
            final_dir.display()
        ));
    }

    let backup_dir = final_dir.with_file_name(format!(
        ".replace-{registry_id}-{}-{}.bak",
        std::process::id(),
        nonce()?
    ));
    let had_existing_dir = final_dir.exists();
    if had_existing_dir {
        fs::rename(final_dir, &backup_dir).map_err(|error| {
            format!(
                "failed to stage existing capsule {} for replacement: {error}",
                final_dir.display()
            )
        })?;
    }

    let moved = fs::rename(temp_dir, final_dir);
    if let Err(error) = moved {
        restore_replacement_backup(final_dir, &backup_dir, had_existing_dir);
        return Err(format!(
            "failed to move replacement capsule {} to {}: {error}",
            temp_dir.display(),
            final_dir.display()
        ));
    }

    if let Err(error) = registry::replace_imported_capsule(registry_id, final_dir) {
        fs::remove_dir_all(final_dir).ok();
        restore_replacement_backup(final_dir, &backup_dir, had_existing_dir);
        return Err(error);
    }

    if had_existing_dir {
        fs::remove_dir_all(&backup_dir).map_err(|error| {
            format!(
                "failed to remove replacement backup {}: {error}",
                backup_dir.display()
            )
        })?;
    }

    let registry_path = registry::registry_path_display()?;
    let source_type = registry::IMPORTED_SKR_SOURCE_TYPE.to_string();
    let view = ImportView {
        command: "import",
        schema_version: "import.v1",
        ok: true,
        package_path: display_path(package_path),
        registry_path,
        capsule: ImportedCapsuleView {
            id: registry_id.to_string(),
            path: display_path(final_dir),
            source_type,
            enabled: existing.enabled,
            replaced: true,
        },
        warnings: import_warnings(),
    };

    if options.json {
        return Ok(ImportOutput {
            output: serde_json::to_string_pretty(&view).map_err(|error| error.to_string())?,
        });
    }

    Ok(ImportOutput {
        output: format!(
            "replaced {id}\npath: {path}\nenabled: {enabled}\nnote: replacement validates the .skr before swapping files and does not install dependencies.",
            id = registry_id,
            path = final_dir.display(),
            enabled = existing.enabled,
        ),
    })
}

fn restore_replacement_backup(final_dir: &Path, backup_dir: &Path, had_existing_dir: bool) {
    if had_existing_dir && backup_dir.exists() {
        fs::remove_dir_all(final_dir).ok();
        fs::rename(backup_dir, final_dir).ok();
    }
}

fn extract_package(package_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(package_path)
        .map_err(|error| format!("failed to open package {}: {error}", package_path.display()))?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|error| format!("failed to read package archive: {error}"))?;

    for entry in entries {
        let mut entry = entry.map_err(|error| format!("failed to read package entry: {error}"))?;
        let entry_path = entry
            .path()
            .map_err(|error| format!("failed to read package entry path: {error}"))?;
        let relative_path = normalize_package_path(entry_path.as_ref())?;
        let output_path = target_dir.join(relative_path);

        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            fs::create_dir_all(&output_path).map_err(|error| {
                format!(
                    "failed to create package directory {}: {error}",
                    output_path.display()
                )
            })?;
            continue;
        }
        if !entry_type.is_file() {
            return Err(format!(
                "unsupported package entry type for {}",
                output_path.display()
            ));
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create package directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        entry
            .unpack(&output_path)
            .map_err(|error| format!("failed to unpack {}: {error}", output_path.display()))?;
    }

    Ok(())
}

fn normalize_package_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Err(format!(
            "package path escapes import target: {}",
            path.display()
        ));
    }
    let mut normalized = PathBuf::new();
    let mut has_component = false;
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                normalized.push(value);
                has_component = true;
            }
            Component::CurDir => {}
            _ => {
                return Err(format!(
                    "package path escapes import target: {}",
                    path.display()
                ));
            }
        }
    }
    if !has_component {
        return Err("package path escapes import target: empty entry path".to_string());
    }
    Ok(normalized)
}

fn import_error_code(message: &str) -> &'static str {
    if message.starts_with("registry id already exists") {
        "registry-id-exists"
    } else if message.starts_with("import target already exists") {
        "import-target-exists"
    } else if message.starts_with("registry id not found") {
        "registry-id-not-found"
    } else if message.starts_with("import replace requires imported_skr") {
        "replace-source-type-unsupported"
    } else if message.starts_with("import replace target mismatch") {
        "replace-target-mismatch"
    } else if message.starts_with("package does not exist") {
        "package-not-found"
    } else if message.starts_with("package is not a file") {
        "package-not-file"
    } else if message.starts_with("package path escapes import target") {
        "package-path-escape"
    } else if message.starts_with("unsupported package entry type") {
        "unsupported-package-entry"
    } else if message.contains("Manifest") || message.contains("manifest") {
        "invalid-manifest"
    } else {
        "import-failed"
    }
}

fn import_warnings() -> Vec<ImportWarningView> {
    vec![
        ImportWarningView {
            code: "not-enabled",
            message: "Imported capsule is disabled until switchboard enable is called.",
        },
        ImportWarningView {
            code: "dependencies-not-installed",
            message: ".skr import does not install runtime dependencies.",
        },
    ]
}

fn default_import_root() -> Result<PathBuf, String> {
    Ok(skillrun_home()?.join("capsules"))
}

fn skillrun_home() -> Result<PathBuf, String> {
    if let Some(home) = std::env::var_os("SKILLRUN_HOME") {
        return Ok(PathBuf::from(home));
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| "SKILLRUN_HOME, USERPROFILE, or HOME must be set".to_string())?;
    Ok(PathBuf::from(home).join(".skillrun"))
}

fn absolute_existing_file(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("package does not exist: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("package is not a file: {}", path.display()));
    }
    fs::canonicalize(path).map_err(|error| format!("failed to resolve {}: {error}", path.display()))
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

fn nonce() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| format!("system clock is before unix epoch: {error}"))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
