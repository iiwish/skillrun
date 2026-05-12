use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
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

    let capsule = output_root.join("refund");
    (output_root, capsule)
}

fn write_manifest(capsule: &Path) {
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );
}

fn generated_capsule(label: &str) -> (PathBuf, PathBuf) {
    let (output_root, capsule) = init_capsule(label);
    write_manifest(&capsule);
    (output_root, capsule)
}

fn assert_success_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    serde_json::from_str(&stdout).expect("stdout should be JSON")
}

#[test]
fn mcp_dry_run_maps_manifest_tool_schema_and_skill_resource() {
    let (output_root, capsule) = generated_capsule("mcp-dry-run");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    let contract = assert_success_json(&serve);

    assert_eq!(contract["mcp"]["dry_run"], true);
    assert_eq!(contract["mcp"]["transport"], "stdio");
    assert_eq!(contract["tools"][0]["name"], "refund");
    assert!(contract["tools"][0]["description"]
        .as_str()
        .unwrap()
        .contains("refund"));
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["order_id"]["type"],
        "string"
    );
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["amount"]["type"],
        "integer"
    );
    assert_eq!(
        contract["tools"][0]["result_contract"],
        "SkillRun output/error envelope"
    );
    assert_eq!(contract["resources"][0]["path"], "SKILL.md");
    assert_eq!(contract["resources"][0]["mime_type"], "text/markdown");
    let resource_text = contract["resources"][0]["text"]
        .as_str()
        .expect("resource text should be a string");
    assert!(resource_text.to_ascii_lowercase().contains("refund"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_dry_run_does_not_import_action_for_metadata() {
    let (output_root, capsule) = init_capsule("mcp-no-import");
    let marker = output_root.join("import-marker.txt");
    let marker_literal = serde_json::to_string(&marker.to_string_lossy().to_string())
        .expect("marker should serialize");
    let action_path = capsule.join("action.py");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action.replace(
        "from typing import Literal\n",
        &format!(
            "from typing import Literal\nfrom pathlib import Path\nPath({marker_literal}).write_text(\"imported\", encoding=\"utf-8\")\n"
        ),
    );
    fs::write(&action_path, action).expect("action should be updated");

    write_manifest(&capsule);
    assert!(
        marker.is_file(),
        "manifest metadata extraction imports action.py in Author Mode"
    );
    fs::remove_file(&marker).expect("marker should be removed before serve");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert_success_json(&serve);
    assert!(
        !marker.exists(),
        "serve --mcp --dry-run must not import action.py for metadata"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_dry_run_fails_closed_when_manifest_is_stale() {
    let (output_root, capsule) = generated_capsule("mcp-stale");
    let action_path = capsule.join("action.py");
    let mut action = fs::read_to_string(&action_path).expect("action should be readable");
    action.push_str("\n# stale after manifest\n");
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert!(!serve.status.success());
    let stderr = String::from_utf8(serve.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("stale Manifest"));
    assert!(stderr.contains("action.py"));
    assert!(!stderr.contains("command not implemented yet"));

    fs::remove_dir_all(output_root).ok();
}
