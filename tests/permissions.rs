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

fn run_skillrun_with_env(args: &[&str], key: &str, value: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .env(key, value)
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

fn write_config_env(capsule: &Path, env_reads: &[&str]) {
    let reads = env_reads
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let config = format!(
        "{{\n  \"runtime\": {{\n    \"adapter\": \"python\",\n    \"entrypoint\": \"action.py\",\n    \"timeout\": \"30s\"\n  }},\n  \"permissions\": {{\n    \"files\": {{\n      \"read\": [],\n      \"write\": [\".skillrun/runs/**\"]\n    }},\n    \"network\": {{\n      \"outbound\": []\n    }},\n    \"env\": {{\n      \"read\": [{reads}]\n    }}\n  }}\n}}\n"
    );
    fs::write(capsule.join("skillrun.config.json"), config).expect("config should be written");
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

fn patch_action_audit_note_from_env(capsule: &Path, env_name: &str) {
    let action_path = capsule.join("action.py");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let env_name = serde_json::to_string(env_name).expect("env name should serialize");
    let action = action
        .replace(
            "from typing import Literal\n",
            "from typing import Literal\nimport os\n",
        )
        .replace(
            "def run(input: Input, ctx) -> Output:\n",
            &format!(
                "def run(input: Input, ctx) -> Output:\n    return Output(\n        decision=\"approved\",\n        amount=input.amount,\n        reasoning_summary=\"Environment visibility was captured by the action.\",\n        audit_note=os.environ.get({env_name}, \"missing\"),\n    )\n"
            ),
        );
    fs::write(&action_path, action).expect("action should be updated");
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).expect("json file should be readable");
    serde_json::from_str(&text).expect("json file should parse")
}

fn run_dir(capsule: &Path, run_id: &str) -> PathBuf {
    capsule.join(".skillrun").join("runs").join(run_id)
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
fn undeclared_env_is_not_injected_into_action_process() {
    let (output_root, capsule) = init_capsule("permissions-undeclared-env");
    patch_action_audit_note_from_env(&capsule, "SKILLRUN_SECRET");
    write_manifest(&capsule);

    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun_with_env(&["test", "--cwd", &cwd_arg], "SKILLRUN_SECRET", "secret");
    let envelope = assert_success_envelope(&run);
    assert_eq!(envelope["output"]["audit_note"], "missing");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn declared_env_is_injected_and_recorded() {
    let (output_root, capsule) = init_capsule("permissions-declared-env");
    write_config_env(&capsule, &["SKILLRUN_ALLOWED_ENV"]);
    patch_action_audit_note_from_env(&capsule, "SKILLRUN_ALLOWED_ENV");
    write_manifest(&capsule);

    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun_with_env(
        &["test", "--cwd", &cwd_arg],
        "SKILLRUN_ALLOWED_ENV",
        "allowed-value",
    );
    let envelope = assert_success_envelope(&run);
    assert_eq!(envelope["output"]["audit_note"], "allowed-value");

    let run_id = envelope["run_id"].as_str().unwrap();
    let record = read_json(&run_dir(&capsule, run_id).join("record.json"));
    assert_eq!(
        record["permissions"]["env"]["read"][0],
        "SKILLRUN_ALLOWED_ENV"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn declared_env_cannot_override_ipc_envs() {
    let (output_root, capsule) = init_capsule("permissions-ipc-env");
    write_config_env(&capsule, &["SKILLRUN_OUTPUT_JSON"]);
    write_manifest(&capsule);

    let outside_output = output_root.join("outside-output.json");
    let outside_arg = outside_output.to_string_lossy().to_string();
    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun_with_env(
        &["test", "--cwd", &cwd_arg],
        "SKILLRUN_OUTPUT_JSON",
        &outside_arg,
    );
    let envelope = assert_success_envelope(&run);
    assert!(!outside_output.exists());

    let run_id = envelope["run_id"].as_str().unwrap();
    assert!(run_dir(&capsule, run_id).join("output.json").is_file());

    fs::remove_dir_all(output_root).ok();
}
