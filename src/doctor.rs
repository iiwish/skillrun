use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::hashing;
use crate::manifest;

#[derive(Debug)]
pub struct DoctorOptions {
    pub cwd: PathBuf,
}

pub struct DoctorReport {
    pub output: String,
    pub ok: bool,
}

pub fn check(options: &DoctorOptions) -> Result<DoctorReport, String> {
    let cwd = absolute_path(&options.cwd)?;
    if !cwd.exists() {
        return Err(format!("cwd does not exist: {}", cwd.display()));
    }
    if !cwd.is_dir() {
        return Err(format!("cwd is not a directory: {}", cwd.display()));
    }

    let manifest_path = manifest::generated_manifest_path(&cwd);
    let files = FileStatus::read(&cwd);

    if !files.skill {
        return Ok(report_without_manifest(
            &cwd,
            &manifest_path,
            &files,
            "invalid",
            "missing SKILL.md",
            "Add SKILL.md before running `skillrun manifest`.",
        ));
    }

    if !manifest_path.is_file() {
        if files.typescript && !files.python_action && !files.node_action {
            return Ok(report_without_manifest(
                &cwd,
                &manifest_path,
                &files,
                "unsupported-typescript",
                "action.ts is not supported in v0.3 JS alpha.",
                &format!(
                    "compile to action.mjs, then run `skillrun manifest --cwd {}`.",
                    cwd.display()
                ),
            ));
        }

        if !files.python_action && !files.node_action {
            return Ok(report_without_manifest(
                &cwd,
                &manifest_path,
                &files,
                "instruction-only",
                "SkillRun does not infer actions from Markdown, scripts, references, assets, or examples.",
                &format!(
                    "Add action.py or action.mjs, then run `skillrun manifest --cwd {}`.",
                    cwd.display()
                ),
            ));
        }

        return Ok(report_without_manifest(
            &cwd,
            &manifest_path,
            &files,
            "missing-manifest",
            "missing Manifest for Consumer Mode.",
            &format!("Run `skillrun manifest --cwd {}`.", cwd.display()),
        ));
    }

    report_with_manifest(&cwd, &manifest_path, &files)
}

struct FileStatus {
    skill: bool,
    python_action: bool,
    node_action: bool,
    typescript: bool,
    config: bool,
}

impl FileStatus {
    fn read(cwd: &Path) -> Self {
        Self {
            skill: cwd.join("SKILL.md").is_file(),
            python_action: cwd.join("action.py").is_file(),
            node_action: cwd.join("action.mjs").is_file(),
            typescript: cwd.join("action.ts").is_file(),
            config: cwd.join("skillrun.config.json").is_file(),
        }
    }
}

fn report_without_manifest(
    cwd: &Path,
    manifest_path: &Path,
    files: &FileStatus,
    status: &str,
    reason: &str,
    next_step: &str,
) -> DoctorReport {
    DoctorReport {
        output: format!(
            "\
SkillRun Doctor
cwd: {cwd}
status: {status}
files:
{files}
manifest:
  path: {manifest_path}
  present: no
  manifest freshness: missing
examples:
  examples/default.input.json: {example_status}
reason: {reason}
next step: {next_step}
note: doctor reads files and hashes only; it does not run or import action source.",
            cwd = cwd.display(),
            manifest_path = manifest_path.display(),
            files = render_files(files),
            example_status = presence(cwd.join("examples").join("default.input.json").is_file()),
        ),
        ok: false,
    }
}

fn report_with_manifest(
    cwd: &Path,
    manifest_path: &Path,
    files: &FileStatus,
) -> Result<DoctorReport, String> {
    let text = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest: Value = serde_yaml::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;

    let adapter = string_at(&manifest, &["runtime", "adapter"]).unwrap_or("unknown");
    let entrypoint = string_at(&manifest, &["runtime", "entrypoint"]).unwrap_or("unknown");
    let source_checks = source_checks(cwd, &manifest);
    let examples = example_checks(cwd, &manifest);
    let all_fresh = source_checks.iter().all(|check| check.status == "fresh");
    let status = if all_fresh { "ok" } else { "stale-manifest" };
    let freshness = if all_fresh { "fresh" } else { "stale" };
    let next_step = if all_fresh {
        "none".to_string()
    } else {
        format!("Run `skillrun manifest --cwd {}`.", cwd.display())
    };

    Ok(DoctorReport {
        output: format!(
            "\
SkillRun Doctor
cwd: {cwd}
status: {status}
files:
{files}
manifest:
  path: {manifest_path}
  present: yes
  manifest freshness: {freshness}
runtime:
  adapter: {adapter}
  entrypoint: {entrypoint}
sources:
{sources}
examples:
{examples}
next step: {next_step}
note: doctor reads files and hashes only; it does not run or import action source.",
            cwd = cwd.display(),
            files = render_files(files),
            manifest_path = manifest_path.display(),
            sources = render_source_checks(&source_checks),
            examples = render_example_checks(&examples),
        ),
        ok: all_fresh,
    })
}

struct SourceCheck {
    path: String,
    status: &'static str,
}

fn source_checks(cwd: &Path, manifest: &Value) -> Vec<SourceCheck> {
    let mut checks = Vec::new();
    for key in ["skill", "action", "config"] {
        let Some(path) = string_at(manifest, &["sources", key, "path"]) else {
            continue;
        };
        let expected = string_at(manifest, &["sources", key, "sha256"]);
        let status = match expected {
            Some(expected) => match hashing::sha256_file(&cwd.join(path)) {
                Ok(actual) if actual == expected => "fresh",
                Ok(_) => "stale",
                Err(_) => "missing",
            },
            None => "missing-hash",
        };
        checks.push(SourceCheck {
            path: path.to_string(),
            status,
        });
    }
    checks
}

struct ExampleCheck {
    path: String,
    present: bool,
}

fn example_checks(cwd: &Path, manifest: &Value) -> Vec<ExampleCheck> {
    let Some(Value::Sequence(items)) = value_at(manifest, &["examples"]) else {
        return vec![ExampleCheck {
            path: "examples/default.input.json".to_string(),
            present: cwd.join("examples").join("default.input.json").is_file(),
        }];
    };

    let checks = items
        .iter()
        .filter_map(|item| string_at(item, &["input"]))
        .map(|path| ExampleCheck {
            path: path.to_string(),
            present: cwd.join(path).is_file(),
        })
        .collect::<Vec<_>>();

    if checks.is_empty() {
        vec![ExampleCheck {
            path: "examples/default.input.json".to_string(),
            present: cwd.join("examples").join("default.input.json").is_file(),
        }]
    } else {
        checks
    }
}

fn render_files(files: &FileStatus) -> String {
    format!(
        "\
  SKILL.md: {skill}
  action.py: {python}
  action.mjs: {node}
  action.ts: {typescript}
  skillrun.config.json: {config}",
        skill = presence(files.skill),
        python = presence(files.python_action),
        node = presence(files.node_action),
        typescript = presence(files.typescript),
        config = presence(files.config),
    )
}

fn render_source_checks(checks: &[SourceCheck]) -> String {
    if checks.is_empty() {
        return "  none".to_string();
    }

    checks
        .iter()
        .map(|check| format!("  {}: {}", check.path, check.status))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_example_checks(checks: &[ExampleCheck]) -> String {
    if checks.is_empty() {
        return "  none".to_string();
    }

    checks
        .iter()
        .map(|check| format!("  {}: {}", check.path, presence(check.present)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn presence(value: bool) -> &'static str {
    if value {
        "present"
    } else {
        "absent"
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

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        let key = Value::String((*segment).to_string());
        current = current.as_mapping()?.get(&key)?;
    }
    Some(current)
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    value_at(value, path)?.as_str()
}
