use serde::Serialize;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters;
use crate::hashing;
use crate::manifest;

pub struct ReadinessReport {
    pub cwd: PathBuf,
    pub manifest_path: PathBuf,
    pub files: FileStatus,
    pub manifest_present: bool,
    pub status: String,
    pub freshness: String,
    pub reason: Option<String>,
    pub next_step: String,
    pub adapter: Option<String>,
    pub entrypoint: Option<String>,
    pub requirements: RequirementsView,
    pub dependency_checks: Vec<HostDependencyCheck>,
    pub source_checks: Vec<SourceCheck>,
    pub example_checks: Vec<ExampleCheck>,
    pub ok: bool,
}

#[derive(Serialize)]
pub struct FileStatus {
    pub skill: bool,
    pub python_action: bool,
    pub node_action: bool,
    pub typescript: bool,
    pub config: bool,
}

#[derive(Serialize)]
pub struct SourceCheck {
    pub path: String,
    pub status: &'static str,
}

#[derive(Serialize)]
pub struct ExampleCheck {
    pub path: String,
    pub present: bool,
}

#[derive(Default, Serialize)]
pub struct RequirementsView {
    pub present: bool,
    pub executable: Option<ExecutableView>,
    pub packages: Vec<PackageView>,
}

#[derive(Serialize)]
pub struct ExecutableView {
    pub name: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct PackageView {
    pub name: String,
    pub version: String,
    pub required_for: Vec<String>,
}

#[derive(Serialize)]
pub struct HostDependencyCheck {
    pub kind: &'static str,
    pub name: String,
    pub required: String,
    pub detected: Option<String>,
    pub status: &'static str,
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

pub fn evaluate(cwd: &Path) -> Result<ReadinessReport, String> {
    let cwd = absolute_path(cwd)?;
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
            cwd,
            manifest_path,
            files,
            "invalid",
            "missing SKILL.md",
            "Add SKILL.md before running `skillrun manifest`.",
        ));
    }

    if !manifest_path.is_file() {
        if files.typescript && !files.python_action && !files.node_action {
            return Ok(report_without_manifest(
                cwd,
                manifest_path,
                files,
                "unsupported-typescript",
                "action.ts is not supported in v0.3 JS alpha.",
                "compile to action.mjs, then run `skillrun manifest --cwd {cwd}`.",
            ));
        }

        if !files.python_action && !files.node_action {
            return Ok(report_without_manifest(
                cwd,
                manifest_path,
                files,
                "instruction-only",
                "SkillRun does not infer actions from Markdown, scripts, references, assets, or examples.",
                "Add action.py or action.mjs, then run `skillrun manifest --cwd {cwd}`.",
            ));
        }

        return Ok(report_without_manifest(
            cwd,
            manifest_path,
            files,
            "missing-manifest",
            "missing Manifest for Consumer Mode.",
            "Run `skillrun manifest --cwd {cwd}`.",
        ));
    }

    report_with_manifest(cwd, manifest_path, files)
}

fn report_without_manifest(
    cwd: PathBuf,
    manifest_path: PathBuf,
    files: FileStatus,
    status: &str,
    reason: &str,
    next_step: &str,
) -> ReadinessReport {
    let next_step = next_step.replace("{cwd}", &cwd.display().to_string());
    ReadinessReport {
        example_checks: vec![ExampleCheck {
            path: "examples/default.input.json".to_string(),
            present: cwd.join("examples").join("default.input.json").is_file(),
        }],
        cwd,
        manifest_path,
        files,
        manifest_present: false,
        status: status.to_string(),
        freshness: "missing".to_string(),
        reason: Some(reason.to_string()),
        next_step,
        adapter: None,
        entrypoint: None,
        requirements: RequirementsView::default(),
        dependency_checks: Vec::new(),
        source_checks: Vec::new(),
        ok: false,
    }
}

fn report_with_manifest(
    cwd: PathBuf,
    manifest_path: PathBuf,
    files: FileStatus,
) -> Result<ReadinessReport, String> {
    let text = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest: Value = serde_yaml::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;

    let adapter = string_at(&manifest, &["runtime", "adapter"]).map(ToString::to_string);
    let entrypoint = string_at(&manifest, &["runtime", "entrypoint"]).map(ToString::to_string);
    let source_checks = source_checks(&cwd, &manifest);
    let example_checks = example_checks(&cwd, &manifest);
    let requirements = requirements_view(&manifest);
    let dependency_checks = dependency_checks(adapter.as_deref(), &requirements);
    let runtime_present = adapter.is_some() && entrypoint.is_some();
    let all_fresh = source_checks.iter().all(|check| check.status == "fresh");
    let examples_present = example_checks.iter().all(|check| check.present);
    let dependencies_ready = dependency_checks
        .iter()
        .all(|check| check.status == "satisfied");
    let status = if !all_fresh {
        "stale-manifest"
    } else if !runtime_present {
        "invalid-manifest"
    } else if !examples_present {
        "missing-examples"
    } else if !dependencies_ready {
        "dependency-error"
    } else {
        "ok"
    };
    let freshness = if all_fresh { "fresh" } else { "stale" };
    let next_step = match status {
        "ok" => "none".to_string(),
        "missing-examples" => {
            format!(
                "Restore the example files, then run `skillrun manifest --cwd {}`.",
                cwd.display()
            )
        }
        "dependency-error" => "Install or select a runtime matching the declared requirements, then retry `skillrun check`.".to_string(),
        _ => format!("Run `skillrun manifest --cwd {}`.", cwd.display()),
    };

    Ok(ReadinessReport {
        cwd,
        manifest_path,
        files,
        manifest_present: true,
        status: status.to_string(),
        freshness: freshness.to_string(),
        reason: None,
        next_step,
        adapter,
        entrypoint,
        requirements,
        dependency_checks,
        source_checks,
        example_checks,
        ok: status == "ok",
    })
}

fn source_checks(cwd: &Path, manifest: &Value) -> Vec<SourceCheck> {
    let mut checks = Vec::new();
    for (key, fallback_path, required) in [
        ("skill", "SKILL.md", true),
        ("action", "action.py or action.mjs", true),
        ("config", "skillrun.config.json", false),
    ] {
        if let Some(check) = source_check(cwd, manifest, key, fallback_path, required) {
            checks.push(check);
        }
    }
    checks
}

fn source_check(
    cwd: &Path,
    manifest: &Value,
    key: &str,
    fallback_path: &str,
    required: bool,
) -> Option<SourceCheck> {
    let source_path = string_at(manifest, &["sources", key, "path"]);
    let should_check =
        required || source_path.is_some() || (key == "config" && cwd.join(fallback_path).is_file());
    if !should_check {
        return None;
    }

    let Some(path) = source_path else {
        return Some(SourceCheck {
            path: format!("sources.{key}.path"),
            status: "missing-path",
        });
    };

    let Some(expected) = string_at(manifest, &["sources", key, "sha256"]) else {
        return Some(SourceCheck {
            path: path.to_string(),
            status: "missing-hash",
        });
    };

    let status = match hashing::sha256_file(&cwd.join(path)) {
        Ok(actual) if actual == expected => "fresh",
        Ok(_) => "stale",
        Err(_) => "missing",
    };
    Some(SourceCheck {
        path: path.to_string(),
        status,
    })
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

fn requirements_view(manifest: &Value) -> RequirementsView {
    let Some(requirements) = value_at(manifest, &["runtime", "requirements"]) else {
        return RequirementsView::default();
    };

    let executable = match (
        string_at(requirements, &["executable", "name"]),
        string_at(requirements, &["executable", "version"]),
    ) {
        (Some(name), Some(version)) => Some(ExecutableView {
            name: name.to_string(),
            version: version.to_string(),
        }),
        _ => None,
    };

    let packages = match value_at(requirements, &["packages"]) {
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(|item| {
                let name = string_at(item, &["name"])?;
                let version = string_at(item, &["version"])?;
                let required_for = match value_at(item, &["required_for"]) {
                    Some(Value::Sequence(values)) => values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect(),
                    _ => Vec::new(),
                };
                Some(PackageView {
                    name: name.to_string(),
                    version: version.to_string(),
                    required_for,
                })
            })
            .collect(),
        _ => Vec::new(),
    };

    RequirementsView {
        present: true,
        executable,
        packages,
    }
}

fn dependency_checks(
    adapter: Option<&str>,
    requirements: &RequirementsView,
) -> Vec<HostDependencyCheck> {
    let Some(adapter) = adapter else {
        return Vec::new();
    };
    if !requirements.present {
        return Vec::new();
    }

    if adapter == "command" {
        return command_dependency_checks(requirements);
    }

    let Some(discovery) = adapters::discover_runtime(adapter) else {
        return Vec::new();
    };
    let mut checks = Vec::new();

    if let Some(required) = requirements.executable.as_ref() {
        checks.push(host_dependency_check(
            "executable",
            &required.name,
            &required.version,
            &discovery.executable,
        ));
    }

    for required in &requirements.packages {
        let discovered = discovery
            .packages
            .iter()
            .find(|item| item.name == required.name);
        checks.push(match discovered {
            Some(discovered) => {
                host_dependency_check("package", &required.name, &required.version, discovered)
            }
            None => HostDependencyCheck {
                kind: "package",
                name: required.name.clone(),
                required: required.version.clone(),
                detected: None,
                status: "missing",
            },
        });
    }

    checks
}

fn command_dependency_checks(requirements: &RequirementsView) -> Vec<HostDependencyCheck> {
    let Some(required) = requirements.executable.as_ref() else {
        return Vec::new();
    };

    let detected = detect_command_executable(&required.name);
    let status = if detected.is_some() {
        "satisfied"
    } else {
        "missing"
    };

    vec![HostDependencyCheck {
        kind: "executable",
        name: required.name.clone(),
        required: required.version.clone(),
        detected,
        status,
    }]
}

fn detect_command_executable(name: &str) -> Option<String> {
    if name.trim().is_empty() {
        return None;
    }

    if has_path_separator(name) {
        return Path::new(name).is_file().then(|| "present".to_string());
    }

    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|directory| command_candidates(&directory, name))
        .any(|candidate| candidate.is_file())
        .then(|| "present".to_string())
}

fn has_path_separator(name: &str) -> bool {
    name.contains('/') || name.contains('\\')
}

fn command_candidates(directory: &Path, name: &str) -> Vec<PathBuf> {
    let path = Path::new(name);
    if path.extension().is_some() {
        return vec![directory.join(path)];
    }

    let mut candidates = vec![directory.join(path)];
    if cfg!(windows) {
        let extensions = std::env::var("PATHEXT").unwrap_or_else(|_| {
            ".COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC".to_string()
        });
        candidates.extend(
            extensions
                .split(';')
                .filter(|extension| !extension.trim().is_empty())
                .map(|extension| directory.join(format!("{name}{extension}"))),
        );
    }
    candidates
}

fn host_dependency_check(
    kind: &'static str,
    name: &str,
    required: &str,
    discovered: &adapters::DiscoveredDependency,
) -> HostDependencyCheck {
    let status = if !discovered.available {
        "missing"
    } else if discovered
        .detected
        .as_deref()
        .is_some_and(|detected| version_satisfies(detected, required))
    {
        "satisfied"
    } else {
        "unsupported-version"
    };

    HostDependencyCheck {
        kind,
        name: name.to_string(),
        required: required.to_string(),
        detected: discovered.detected.clone(),
        status,
    }
}

fn version_satisfies(detected: &str, requirement: &str) -> bool {
    let Some(detected) = parse_version(detected) else {
        return false;
    };

    requirement.split(',').all(|clause| {
        let clause = clause.trim();
        if let Some(required) = clause.strip_prefix(">=") {
            parse_version(required).is_some_and(|required| detected >= required)
        } else if let Some(required) = clause.strip_prefix('<') {
            parse_version(required).is_some_and(|required| detected < required)
        } else {
            false
        }
    })
}

fn parse_version(value: &str) -> Option<Vec<u64>> {
    let numbers = value
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .take(3)
        .filter_map(|part| part.parse::<u64>().ok())
        .collect::<Vec<_>>();
    if numbers.is_empty() {
        None
    } else {
        Some(numbers)
    }
}

pub fn render_check(report: &ReadinessReport) -> String {
    format!(
        "\
SkillRun Check
cwd: {cwd}
status: {status}
files:
{files}
manifest:
  path: {manifest_path}
  present: {manifest_present}
  manifest freshness: {freshness}
runtime:
{runtime}
requirements:
{requirements}
host readiness:
{host_readiness}
sources:
{sources}
examples:
{examples}
{reason}next step: {next_step}
note: check reads Manifest, files and hashes only; it does not run or import action source.",
        cwd = report.cwd.display(),
        status = report.status,
        files = render_files(&report.files),
        manifest_path = report.manifest_path.display(),
        manifest_present = yes_no(report.manifest_present),
        freshness = report.freshness,
        runtime = render_runtime(report),
        requirements = render_requirements(&report.requirements),
        host_readiness = render_host_readiness(&report.dependency_checks),
        sources = render_source_checks(&report.source_checks),
        examples = render_example_checks(&report.example_checks),
        reason = render_reason(report),
        next_step = report.next_step,
    )
}

pub fn render_doctor(report: &ReadinessReport) -> String {
    if !report.manifest_present {
        return format!(
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
{examples}
reason: {reason}
next step: {next_step}
note: doctor reads files and hashes only; it does not run or import action source.",
            cwd = report.cwd.display(),
            status = report.status,
            manifest_path = report.manifest_path.display(),
            files = render_files(&report.files),
            examples = render_example_checks(&report.example_checks),
            reason = report.reason.as_deref().unwrap_or("unknown"),
            next_step = report.next_step,
        );
    }

    format!(
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
        cwd = report.cwd.display(),
        status = report.status,
        files = render_files(&report.files),
        manifest_path = report.manifest_path.display(),
        freshness = report.freshness,
        adapter = report.adapter.as_deref().unwrap_or("unknown"),
        entrypoint = report.entrypoint.as_deref().unwrap_or("unknown"),
        sources = render_source_checks(&report.source_checks),
        examples = render_example_checks(&report.example_checks),
        next_step = report.next_step,
    )
}

#[derive(Serialize)]
struct JsonReadinessReport<'a> {
    command: &'a str,
    ok: bool,
    cwd: String,
    status: &'a str,
    manifest: JsonManifest,
    files: &'a FileStatus,
    runtime: JsonRuntime<'a>,
    requirements: &'a RequirementsView,
    dependency_checks: &'a [HostDependencyCheck],
    source_checks: &'a [SourceCheck],
    example_checks: &'a [ExampleCheck],
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
    next_step: &'a str,
    note: &'static str,
}

#[derive(Serialize)]
struct JsonManifest {
    path: String,
    present: bool,
    freshness: String,
}

#[derive(Serialize)]
struct JsonRuntime<'a> {
    adapter: Option<&'a str>,
    entrypoint: Option<&'a str>,
}

pub fn render_json(command: &str, report: &ReadinessReport) -> Result<String, String> {
    let json = JsonReadinessReport {
        command,
        ok: report.ok,
        cwd: report.cwd.display().to_string(),
        status: &report.status,
        manifest: JsonManifest {
            path: display_path(&report.cwd, &report.manifest_path),
            present: report.manifest_present,
            freshness: report.freshness.clone(),
        },
        files: &report.files,
        runtime: JsonRuntime {
            adapter: report.adapter.as_deref(),
            entrypoint: report.entrypoint.as_deref(),
        },
        requirements: &report.requirements,
        dependency_checks: &report.dependency_checks,
        source_checks: &report.source_checks,
        example_checks: &report.example_checks,
        reason: report.reason.as_deref(),
        next_step: &report.next_step,
        note: "readiness reads Manifest, files and hashes only; it does not run or import action source.",
    };

    serde_json::to_string_pretty(&json).map_err(|error| error.to_string())
}

fn render_runtime(report: &ReadinessReport) -> String {
    match (&report.adapter, &report.entrypoint) {
        (Some(adapter), Some(entrypoint)) => {
            format!("  adapter: {adapter}\n  entrypoint: {entrypoint}")
        }
        _ => "  none".to_string(),
    }
}

fn render_requirements(requirements: &RequirementsView) -> String {
    if !requirements.present {
        return "  absent".to_string();
    }

    let executable = requirements
        .executable
        .as_ref()
        .map(|item| format!("  executable: {} {}", item.name, item.version))
        .unwrap_or_else(|| "  executable: missing".to_string());
    let packages = if requirements.packages.is_empty() {
        "  packages: none".to_string()
    } else {
        requirements
            .packages
            .iter()
            .map(|package| {
                if package.required_for.is_empty() {
                    format!("  package: {} {}", package.name, package.version)
                } else {
                    format!(
                        "  package: {} {} ({})",
                        package.name,
                        package.version,
                        package.required_for.join(",")
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!("{executable}\n{packages}")
}

fn render_host_readiness(checks: &[HostDependencyCheck]) -> String {
    if checks.is_empty() {
        return "  none".to_string();
    }

    checks
        .iter()
        .map(|check| {
            format!(
                "  {kind}: {name} required: {required} detected: {detected} status: {status}",
                kind = check.kind,
                name = check.name,
                required = check.required,
                detected = check.detected.as_deref().unwrap_or("missing"),
                status = check.status,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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

fn render_reason(report: &ReadinessReport) -> String {
    report
        .reason
        .as_ref()
        .map(|reason| format!("reason: {reason}\n"))
        .unwrap_or_default()
}

fn presence(value: bool) -> &'static str {
    if value {
        "present"
    } else {
        "absent"
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn display_path(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
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
