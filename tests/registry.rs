use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_skillrun(args: &[&str], skillrun_home: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .env("SKILLRUN_HOME", skillrun_home)
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

fn generated_capsule(label: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();
    let skillrun_home = output_root.join("skillrun-home");

    let init = run_skillrun(
        &["init", "refund", "--python", "--output", &output_arg],
        &skillrun_home,
    );
    assert!(
        init.status.success(),
        "init should succeed\nstderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );

    (output_root, capsule)
}

fn assert_success_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn assert_missing_capsule(entry: &Value) {
    assert_eq!(entry["id"], "refund");
    assert_eq!(entry["enabled"], false);
    assert_eq!(entry["manifest"]["present"], false);
    assert_eq!(entry["manifest"]["freshness"], "missing");
    assert!(entry.get("skill").is_none());
    assert!(entry.get("runtime").is_none());
    assert!(entry.get("tool").is_none());
    assert_eq!(entry["readiness"]["ok"], false);
    assert_eq!(entry["readiness"]["status"], "missing-path");
    assert!(entry["readiness"]["reason"]
        .as_str()
        .expect("missing path should include reason")
        .contains("cwd does not exist"));
    assert!(entry["readiness"]["next_step"]
        .as_str()
        .expect("missing path should include next step")
        .contains("registry remove"));
}

#[test]
fn registry_list_json_treats_missing_registry_as_empty() {
    let skillrun_home = temp_dir("registry-empty-home");

    let list = run_skillrun(&["registry", "list", "--json"], &skillrun_home);
    let report = assert_success_json(&list);

    assert_eq!(report["command"], "registry list");
    assert_eq!(report["version"], 1);
    assert_eq!(report["capsules"].as_array().unwrap().len(), 0);
    assert!(report["registry_path"]
        .as_str()
        .expect("registry path should be a string")
        .contains("registry.json"));

    fs::remove_dir_all(skillrun_home).ok();
}

#[test]
fn registry_add_list_inspect_and_remove_local_capsule() {
    let (output_root, capsule) = generated_capsule("registry-add-list");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        add.status.success(),
        "registry add should succeed\nstderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let add_stdout = String::from_utf8(add.stdout).expect("stdout should be utf-8");
    assert!(add_stdout.contains("registered refund"));
    assert!(add_stdout.contains("enabled: false"));

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    let capsules = list["capsules"]
        .as_array()
        .expect("capsules should be array");
    assert_eq!(capsules.len(), 1);
    let entry = &capsules[0];
    assert_eq!(entry["id"], "refund");
    assert_eq!(entry["source_type"], "local_path");
    assert_eq!(entry["enabled"], false);
    assert_eq!(entry["manifest"]["present"], true);
    assert_eq!(entry["manifest"]["freshness"], "fresh");
    assert_eq!(entry["skill"]["name"], "refund");
    assert_eq!(entry["runtime"]["adapter"], "python");
    assert_eq!(entry["runtime"]["entrypoint"], "action.py");
    assert_eq!(entry["tool"]["name"], "refund");
    assert_eq!(entry["readiness"]["ok"], true);
    assert_eq!(entry["readiness"]["status"], "ok");

    let inspect = assert_success_json(&run_skillrun(
        &["registry", "inspect", "refund", "--json"],
        &skillrun_home,
    ));
    assert_eq!(inspect["command"], "registry inspect");
    assert_eq!(inspect["capsule"]["id"], "refund");
    assert_eq!(inspect["capsule"]["enabled"], false);

    let remove = run_skillrun(&["registry", "remove", "refund"], &skillrun_home);
    assert!(
        remove.status.success(),
        "registry remove should succeed\nstderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    assert!(
        capsule.join("SKILL.md").is_file(),
        "registry remove must not delete capsule files"
    );

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(list["capsules"].as_array().unwrap().len(), 0);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn switchboard_enable_disable_and_list_json_updates_registry_state() {
    let (output_root, capsule) = generated_capsule("switchboard-happy");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());

    let before = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(before["command"], "switchboard list");
    assert_eq!(before["capsules"][0]["id"], "refund");
    assert_eq!(before["capsules"][0]["enabled"], false);
    assert_eq!(before["capsules"][0]["readiness"]["ok"], true);

    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(
        enable.status.success(),
        "switchboard enable should succeed\nstderr: {}",
        String::from_utf8_lossy(&enable.stderr)
    );
    let enable_stdout = String::from_utf8(enable.stdout).expect("stdout should be utf-8");
    assert!(enable_stdout.contains("enabled refund"));

    let enabled = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(enabled["capsules"][0]["enabled"], true);

    let disable = run_skillrun(&["switchboard", "disable", "refund"], &skillrun_home);
    assert!(
        disable.status.success(),
        "switchboard disable should succeed\nstderr: {}",
        String::from_utf8_lossy(&disable.stderr)
    );
    let disabled = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(disabled["capsules"][0]["enabled"], false);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn registry_and_switchboard_lists_tolerate_missing_capsule_paths() {
    let (output_root, capsule) = generated_capsule("registry-missing-path");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    fs::remove_dir_all(&capsule).expect("test should remove registered capsule path");

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(list["capsules"].as_array().unwrap().len(), 1);
    assert_missing_capsule(&list["capsules"][0]);

    let inspect = assert_success_json(&run_skillrun(
        &["registry", "inspect", "refund", "--json"],
        &skillrun_home,
    ));
    assert_missing_capsule(&inspect["capsule"]);

    let switchboard = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(switchboard["capsules"].as_array().unwrap().len(), 1);
    assert_missing_capsule(&switchboard["capsules"][0]);

    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(!enable.status.success());
    let stderr = String::from_utf8(enable.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("cannot enable refund"));
    assert!(stderr.contains("missing-path"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn registry_add_rejects_duplicate_ids() {
    let (output_root, capsule) = generated_capsule("registry-duplicate-id");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let first = run_skillrun(
        &["registry", "add", "--cwd", &cwd_arg, "--id", "refund-prod"],
        &skillrun_home,
    );
    assert!(first.status.success());

    let duplicate = run_skillrun(
        &["registry", "add", "--cwd", &cwd_arg, "--id", "refund-prod"],
        &skillrun_home,
    );
    assert!(!duplicate.status.success());
    let stderr = String::from_utf8(duplicate.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("registry id already exists"));
    assert!(stderr.contains("refund-prod"));

    fs::remove_dir_all(output_root).ok();
}
