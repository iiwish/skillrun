use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SKILL_TEMPLATE: &str = include_str!("../templates/python/SKILL.md");
const ACTION_TEMPLATE: &str = include_str!("../templates/python/action.py");
const DEFAULT_INPUT_TEMPLATE: &str =
    include_str!("../templates/python/examples/default.input.json");
const CONFIG_TEMPLATE: &str = include_str!("../templates/python/skillrun.config.json");

pub struct InitOptions {
    pub name: String,
    pub output_dir: PathBuf,
}

pub fn create_python_capsule(options: &InitOptions) -> Result<PathBuf, String> {
    validate_capsule_name(&options.name)?;

    let target = options.output_dir.join(&options.name);
    ensure_target_is_writable(&target)?;

    fs::create_dir_all(target.join("examples")).map_err(|error| {
        format!(
            "failed to create capsule directories at {}: {error}",
            target.display()
        )
    })?;

    write_template(&target.join("SKILL.md"), SKILL_TEMPLATE, &options.name)?;
    write_template(&target.join("action.py"), ACTION_TEMPLATE, &options.name)?;
    write_template(
        &target.join("examples").join("default.input.json"),
        DEFAULT_INPUT_TEMPLATE,
        &options.name,
    )?;
    write_template(
        &target.join("skillrun.config.json"),
        CONFIG_TEMPLATE,
        &options.name,
    )?;

    Ok(target)
}

fn validate_capsule_name(name: &str) -> Result<(), String> {
    let is_valid = !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');

    if is_valid {
        Ok(())
    } else {
        Err(format!(
            "invalid capsule name: {name}. Use only ASCII letters, numbers, '-' or '_'"
        ))
    }
}

fn ensure_target_is_writable(target: &Path) -> Result<(), String> {
    if !target.exists() {
        return Ok(());
    }

    let metadata = fs::metadata(target)
        .map_err(|error| format!("failed to inspect {}: {error}", target.display()))?;

    if !metadata.is_dir() {
        return Err(format!(
            "target path is not a directory: {}",
            target.display()
        ));
    }

    let mut entries = fs::read_dir(target)
        .map_err(|error| format!("failed to read {}: {error}", target.display()))?;

    match entries.next() {
        Some(Ok(_)) => Err(format!(
            "target directory is not empty: {}",
            target.display()
        )),
        Some(Err(error)) => Err(format!("failed to inspect {}: {error}", target.display())),
        None => Ok(()),
    }
}

fn write_template(path: &Path, template: &str, name: &str) -> Result<(), String> {
    let rendered = template.replace("{{name}}", name);
    write_new_file(path, rendered.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn write_new_file(path: &Path, content: &[u8]) -> io::Result<()> {
    fs::write(path, content)
}
