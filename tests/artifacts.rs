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

fn patch_action_run(capsule: &Path, run_body: &str) {
    let action_path = capsule.join("action.py");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action
        .replace(
            "from typing import Literal\n",
            "from typing import Literal\nimport os\nfrom pathlib import Path\n",
        )
        .replace("def run(input: Input, ctx) -> Output:\n", run_body);
    fs::write(&action_path, action).expect("action should be updated");
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).expect("json file should be readable");
    serde_json::from_str(&text).expect("json file should parse")
}

fn run_dir(capsule: &Path, run_id: &str) -> PathBuf {
    capsule.join(".skillrun").join("runs").join(run_id)
}

fn assert_error_envelope(output: &std::process::Output, code: &str) -> Value {
    assert!(
        !output.status.success(),
        "command should fail with a structured error\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    let envelope: Value = serde_json::from_str(&stdout).expect("stdout should be a JSON envelope");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], code);
    assert!(envelope["display"]["markdown"].as_str().unwrap().len() > 3);
    assert!(!envelope["display"]["markdown"]
        .as_str()
        .unwrap()
        .contains("Traceback"));
    envelope
}

fn assert_success_envelope(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    let envelope: Value = serde_json::from_str(&stdout).expect("stdout should be a JSON envelope");
    assert_eq!(envelope["ok"], true);
    envelope
}

#[test]
fn artifact_parent_traversal_returns_permission_denied() {
    let (output_root, capsule) = init_capsule("artifact-traversal");
    patch_action_run(
        &capsule,
        "def run(input: Input, ctx) -> Output:\n    return {\n        \"output\": Output(\n            decision=\"approved\",\n            amount=input.amount,\n            reasoning_summary=\"Declared artifact escapes the run artifact directory.\",\n            audit_note=\"artifact traversal should fail\",\n        ).model_dump(mode=\"json\"),\n        \"artifacts\": [{\"name\": \"receipt\", \"kind\": \"text\", \"path\": \"../outside.txt\"}],\n    }\n",
    );
    write_manifest(&capsule);

    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let envelope = assert_error_envelope(&run, "PermissionDenied");

    let run_id = envelope["run_id"].as_str().unwrap();
    let record = read_json(&run_dir(&capsule, run_id).join("record.json"));
    assert_eq!(record["status"], "failed");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn absolute_artifact_path_returns_permission_denied() {
    let (output_root, capsule) = init_capsule("artifact-absolute");
    let outside_path = output_root
        .join("outside.txt")
        .to_string_lossy()
        .to_string();
    let outside_path = serde_json::to_string(&outside_path).expect("path should serialize");
    patch_action_run(
        &capsule,
        &format!(
            "def run(input: Input, ctx) -> Output:\n    return {{\n        \"output\": Output(\n            decision=\"approved\",\n            amount=input.amount,\n            reasoning_summary=\"Declared artifact uses an absolute path.\",\n            audit_note=\"absolute artifact should fail\",\n        ).model_dump(mode=\"json\"),\n        \"artifacts\": [{{\"name\": \"receipt\", \"kind\": \"text\", \"path\": {outside_path}}}],\n    }}\n"
        ),
    );
    write_manifest(&capsule);

    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    assert_error_envelope(&run, "PermissionDenied");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn missing_artifact_file_returns_permission_denied() {
    let (output_root, capsule) = init_capsule("artifact-missing");
    patch_action_run(
        &capsule,
        "def run(input: Input, ctx) -> Output:\n    return {\n        \"output\": Output(\n            decision=\"approved\",\n            amount=input.amount,\n            reasoning_summary=\"Declared artifact file was never written.\",\n            audit_note=\"missing artifact should fail\",\n        ).model_dump(mode=\"json\"),\n        \"artifacts\": [{\"name\": \"receipt\", \"kind\": \"text\", \"path\": \"receipt.txt\"}],\n    }\n",
    );
    write_manifest(&capsule);

    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    assert_error_envelope(&run, "PermissionDenied");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn valid_declared_artifact_is_accepted() {
    let (output_root, capsule) = init_capsule("artifact-valid");
    patch_action_run(
        &capsule,
        "def run(input: Input, ctx) -> Output:\n    artifact = Path(os.environ[\"SKILLRUN_ARTIFACT_DIR\"]) / \"receipt.txt\"\n    artifact.write_text(\"refund receipt\", encoding=\"utf-8\")\n    return {\n        \"output\": Output(\n            decision=\"approved\",\n            amount=input.amount,\n            reasoning_summary=\"Declared artifact stays inside the run artifact directory.\",\n            audit_note=\"artifact accepted\",\n        ).model_dump(mode=\"json\"),\n        \"artifacts\": [{\"name\": \"receipt\", \"kind\": \"text\", \"path\": \"receipt.txt\"}],\n    }\n",
    );
    write_manifest(&capsule);

    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let envelope = assert_success_envelope(&run);
    assert_eq!(envelope["artifacts"][0]["path"], "receipt.txt");

    let run_id = envelope["run_id"].as_str().unwrap();
    assert!(run_dir(&capsule, run_id)
        .join("artifacts")
        .join("receipt.txt")
        .is_file());

    fs::remove_dir_all(output_root).ok();
}
