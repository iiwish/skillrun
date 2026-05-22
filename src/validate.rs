use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

use crate::readiness::{self, ReadinessReport};
use crate::runtime::{self, TestOptions};

#[derive(Debug)]
pub struct ValidateOptions {
    pub cwd: PathBuf,
    pub json: bool,
}

pub struct ValidateReport {
    pub output: String,
    pub ok: bool,
}

#[derive(Serialize)]
struct ValidateStage {
    name: &'static str,
    ok: bool,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
}

#[derive(Serialize)]
struct ValidateJsonReport {
    command: &'static str,
    ok: bool,
    cwd: String,
    status: String,
    stages: Vec<ValidateStage>,
    readiness: Value,
    next_step: String,
    note: &'static str,
}

pub fn run(options: &ValidateOptions) -> Result<ValidateReport, String> {
    let readiness = readiness::evaluate(&options.cwd)?;
    let mut stages = base_stages(&readiness);

    if readiness.ok {
        let test = runtime::run_test(&TestOptions {
            cwd: readiness.cwd.clone(),
        });
        stages.push(test_stage(test));
    } else {
        stages.push(ValidateStage {
            name: "test",
            ok: false,
            status: "skipped".to_string(),
            detail: Some("readiness failed; default example was not executed".to_string()),
            next_step: Some(readiness.next_step.clone()),
            run_id: None,
            error_code: None,
        });
    }

    stages.push(run_stage(&readiness, &stages));
    let ok = stages.iter().all(|stage| stage.ok);
    let status = if ok {
        "ok".to_string()
    } else if readiness.ok {
        "test-failed".to_string()
    } else {
        "readiness-failed".to_string()
    };
    let next_step = next_step(&readiness, &stages, ok);

    let output = if options.json {
        render_json(&readiness, stages, ok, status, next_step)?
    } else {
        render_human(&readiness, &stages, &status, &next_step)
    };

    Ok(ValidateReport { output, ok })
}

fn base_stages(readiness: &ReadinessReport) -> Vec<ValidateStage> {
    let manifest_ok = readiness.manifest_present && readiness.freshness == "fresh";
    let manifest_status = if readiness.manifest_present {
        readiness.freshness.clone()
    } else {
        "missing".to_string()
    };

    vec![
        ValidateStage {
            name: "manifest",
            ok: manifest_ok,
            status: manifest_status,
            detail: Some(format!(
                "{} ({})",
                readiness.manifest_path.display(),
                if readiness.manifest_present {
                    "present"
                } else {
                    "missing"
                }
            )),
            next_step: (!manifest_ok).then(|| readiness.next_step.clone()),
            run_id: None,
            error_code: None,
        },
        ValidateStage {
            name: "check",
            ok: readiness.ok,
            status: readiness.status.clone(),
            detail: readiness.reason.clone(),
            next_step: (!readiness.ok).then(|| readiness.next_step.clone()),
            run_id: None,
            error_code: None,
        },
        ValidateStage {
            name: "doctor",
            ok: true,
            status: if readiness.ok {
                "no-recovery-needed".to_string()
            } else {
                "recovery-advice".to_string()
            },
            detail: Some(format!("next step: {}", readiness.next_step)),
            next_step: (!readiness.ok).then(|| readiness.next_step.clone()),
            run_id: None,
            error_code: None,
        },
    ]
}

fn test_stage(result: Result<runtime::RunOutcome, String>) -> ValidateStage {
    match result {
        Ok(outcome) => {
            let parsed = serde_json::from_str::<Value>(&outcome.envelope).ok();
            let run_id = parsed
                .as_ref()
                .and_then(|value| value.get("run_id"))
                .and_then(Value::as_str)
                .map(ToString::to_string);
            let error_code = parsed
                .as_ref()
                .and_then(|value| value.get("error"))
                .and_then(|error| error.get("code"))
                .and_then(Value::as_str)
                .map(ToString::to_string);
            let detail = if outcome.success {
                "default example passed".to_string()
            } else {
                parsed
                    .as_ref()
                    .and_then(|value| value.get("error"))
                    .and_then(|error| error.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("default example failed")
                    .to_string()
            };
            ValidateStage {
                name: "test",
                ok: outcome.success,
                status: if outcome.success { "passed" } else { "failed" }.to_string(),
                detail: Some(detail),
                next_step: (!outcome.success)
                    .then(|| "Inspect the test envelope and run record, then fix the action or example.".to_string()),
                run_id,
                error_code,
            }
        }
        Err(error) => ValidateStage {
            name: "test",
            ok: false,
            status: "failed".to_string(),
            detail: Some(error),
            next_step: Some(
                "Fix the runtime error reported by the default example, then retry `skillrun validate`."
                    .to_string(),
            ),
            run_id: None,
            error_code: None,
        },
    }
}

fn run_stage(readiness: &ReadinessReport, stages: &[ValidateStage]) -> ValidateStage {
    let test_passed = stages
        .iter()
        .any(|stage| stage.name == "test" && stage.status == "passed");
    if test_passed {
        ValidateStage {
            name: "run",
            ok: true,
            status: "ready".to_string(),
            detail: Some(format!(
                "use `skillrun run --cwd {} --input <file>` for explicit inputs",
                readiness.cwd.display()
            )),
            next_step: Some("Run with an explicit input or package the capsule.".to_string()),
            run_id: None,
            error_code: None,
        }
    } else {
        ValidateStage {
            name: "run",
            ok: false,
            status: "skipped".to_string(),
            detail: Some("validate did not reach explicit run readiness".to_string()),
            next_step: Some(next_step(readiness, stages, false)),
            run_id: None,
            error_code: None,
        }
    }
}

fn next_step(readiness: &ReadinessReport, stages: &[ValidateStage], ok: bool) -> String {
    if ok {
        return format!(
            "Run `skillrun run --cwd {} --input <file>` or `skillrun pack --cwd {}`.",
            readiness.cwd.display(),
            readiness.cwd.display()
        );
    }

    stages
        .iter()
        .find_map(|stage| (!stage.ok).then(|| stage.next_step.clone()).flatten())
        .unwrap_or_else(|| readiness.next_step.clone())
}

fn render_json(
    readiness: &ReadinessReport,
    stages: Vec<ValidateStage>,
    ok: bool,
    status: String,
    next_step: String,
) -> Result<String, String> {
    let readiness_json = serde_json::from_str(
        &readiness::render_json("check", readiness)
            .map_err(|error| format!("failed to render readiness JSON: {error}"))?,
    )
    .map_err(|error| format!("failed to parse readiness JSON: {error}"))?;
    let report = ValidateJsonReport {
        command: "validate",
        ok,
        cwd: readiness.cwd.display().to_string(),
        status,
        stages,
        readiness: readiness_json,
        next_step,
        note: VALIDATE_NOTE,
    };
    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
}

fn render_human(
    readiness: &ReadinessReport,
    stages: &[ValidateStage],
    status: &str,
    next_step: &str,
) -> String {
    format!(
        "\
SkillRun Validate
cwd: {cwd}
status: {status}
stages:
{stages}
next step: {next_step}
note: {note}",
        cwd = readiness.cwd.display(),
        status = status,
        stages = render_stages(stages),
        next_step = next_step,
        note = VALIDATE_NOTE,
    )
}

fn render_stages(stages: &[ValidateStage]) -> String {
    stages
        .iter()
        .map(|stage| {
            let mut line = format!(
                "  {name}: {status} ({ok})",
                name = stage.name,
                status = stage.status,
                ok = if stage.ok { "ok" } else { "not ok" }
            );
            if let Some(run_id) = &stage.run_id {
                line.push_str(&format!(" run_id: {run_id}"));
            }
            if let Some(error_code) = &stage.error_code {
                line.push_str(&format!(" error: {error_code}"));
            }
            if let Some(detail) = &stage.detail {
                line.push_str(&format!("\n    {detail}"));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const VALIDATE_NOTE: &str = "validate runs the default example only after readiness passes; it does not install dependencies, download packages, or require Docker.";
