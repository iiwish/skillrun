use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_skillrun(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .output()
        .expect("skillrun binary should run")
}

fn temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("skillrun-{label}-{}-{stamp}", std::process::id()))
}

fn init_capsule(label: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(
        init.status.success(),
        "init should succeed\nstderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    (output_root.clone(), output_root.join("refund"))
}

fn generated_capsule(label: &str) -> (PathBuf, PathBuf) {
    let (output_root, capsule) = init_capsule(label);
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );
    (output_root, capsule)
}

fn stage<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["stages"]
        .as_array()
        .expect("stages should be an array")
        .iter()
        .find(|stage| stage["name"] == name)
        .unwrap_or_else(|| panic!("missing stage {name} in {report}"))
}

#[test]
fn validate_runs_readiness_then_default_example() {
    let (output_root, capsule) = generated_capsule("validate-success");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let validate = run_skillrun(&["validate", "--cwd", &cwd_arg]);
    assert!(
        validate.status.success(),
        "validate should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    let stdout = String::from_utf8(validate.stdout).expect("stdout should be utf-8");

    for expected in [
        "SkillRun Validate",
        "status: ok",
        "manifest: fresh (ok)",
        "check: ok (ok)",
        "doctor: no-recovery-needed (ok)",
        "test: passed (ok)",
        "run: ready (ok)",
        "does not install dependencies",
        "require Docker",
    ] {
        assert!(
            stdout.contains(expected),
            "validate output missing {expected:?}\n{stdout}"
        );
    }
    assert!(
        capsule.join(".skillrun").join("runs").is_dir(),
        "validate should create a run record for the default example"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn validate_json_reports_stage_contract() {
    let (output_root, capsule) = generated_capsule("validate-json-success");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let validate = run_skillrun(&["validate", "--json", "--cwd", &cwd_arg]);
    assert!(
        validate.status.success(),
        "validate json should succeed\nstderr: {}",
        String::from_utf8_lossy(&validate.stderr)
    );
    let report: Value =
        serde_json::from_slice(&validate.stdout).expect("validate stdout should be JSON");

    assert_eq!(report["command"], "validate");
    assert_eq!(report["ok"], true);
    assert_eq!(report["status"], "ok");
    assert_eq!(report["readiness"]["command"], "check");
    assert_eq!(report["readiness"]["ok"], true);
    assert_eq!(stage(&report, "manifest")["status"], "fresh");
    assert_eq!(stage(&report, "check")["status"], "ok");
    assert_eq!(stage(&report, "doctor")["status"], "no-recovery-needed");
    assert_eq!(stage(&report, "test")["status"], "passed");
    assert!(stage(&report, "test")["run_id"]
        .as_str()
        .expect("test stage should include run_id")
        .starts_with("run-"));
    assert_eq!(stage(&report, "run")["status"], "ready");
    assert!(report["note"]
        .as_str()
        .expect("note should be present")
        .contains("does not install dependencies"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn validate_skips_test_when_manifest_is_missing() {
    let (output_root, capsule) = init_capsule("validate-missing-manifest");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let validate = run_skillrun(&["validate", "--cwd", &cwd_arg]);
    assert!(!validate.status.success());
    let stdout = String::from_utf8(validate.stdout).expect("stdout should be utf-8");

    for expected in [
        "status: readiness-failed",
        "manifest: missing (not ok)",
        "check: missing-manifest (not ok)",
        "doctor: recovery-advice (ok)",
        "test: skipped (not ok)",
        "run: skipped (not ok)",
        "skillrun manifest",
    ] {
        assert!(
            stdout.contains(expected),
            "validate missing-manifest output missing {expected:?}\n{stdout}"
        );
    }
    assert!(
        !capsule.join(".skillrun").join("runs").exists(),
        "validate must not execute default example when readiness fails"
    );

    fs::remove_dir_all(output_root).ok();
}
