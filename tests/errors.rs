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

fn generated_capsule(label: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();

    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(
        init.status.success(),
        "init should succeed\nstderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );

    (output_root, capsule)
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
        "structured error command should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    let envelope: Value = serde_json::from_str(&stdout).expect("stdout should be a JSON envelope");

    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], code);
    assert!(envelope["error"]["message"].as_str().unwrap().len() > 3);
    assert!(envelope["error"]["recoverable"].is_boolean());
    assert!(envelope["display"]["markdown"].as_str().unwrap().len() > 3);
    assert!(!envelope["display"]["markdown"]
        .as_str()
        .unwrap()
        .contains("Traceback"));
    assert!(envelope["run_id"].as_str().unwrap().starts_with("run-"));
    assert!(envelope["record"]
        .as_str()
        .unwrap()
        .ends_with("record.json"));

    envelope
}

#[test]
fn invalid_input_returns_validation_error_envelope() {
    let (output_root, capsule) = generated_capsule("error-validation");
    let invalid_input = capsule.join("examples").join("invalid.input.json");
    fs::write(
        &invalid_input,
        r#"{
  "order_id": "",
  "amount": 0,
  "reason": "unknown",
  "customer_tier": "standard"
}"#,
    )
    .expect("invalid input should be written");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let input_arg = invalid_input.to_string_lossy().to_string();
    let run = run_skillrun(&["run", "--cwd", &cwd_arg, "--input", &input_arg]);
    let envelope = assert_error_envelope(&run, "ValidationError");
    assert_eq!(envelope["error"]["recoverable"], false);

    let run_id = envelope["run_id"].as_str().unwrap();
    let record = read_json(&run_dir(&capsule, run_id).join("record.json"));
    assert_eq!(record["status"], "failed");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn preflight_rejection_returns_policy_violation_with_hint() {
    let (output_root, capsule) = generated_capsule("error-policy");
    let policy_input = capsule.join("examples").join("policy.input.json");
    fs::write(
        &policy_input,
        r#"{
  "order_id": "order_9001",
  "amount": 900,
  "reason": "damaged",
  "customer_tier": "standard"
}"#,
    )
    .expect("policy input should be written");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let input_arg = policy_input.to_string_lossy().to_string();
    let run = run_skillrun(&["run", "--cwd", &cwd_arg, "--input", &input_arg]);
    let envelope = assert_error_envelope(&run, "PolicyViolation");

    assert_eq!(envelope["error"]["recoverable"], true);
    assert!(envelope["error"]["llm_hint"]
        .as_str()
        .unwrap()
        .contains("approval"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn missing_output_returns_protocol_violation_not_stdout_success() {
    let output_root = temp_dir("error-protocol");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let action_path = capsule.join("action.py");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action
        .replace("from typing import Literal\n", "from typing import Literal\nimport os\n")
        .replace(
            "def run(input: Input, ctx) -> Output:\n",
            "def run(input: Input, ctx) -> Output:\n    print(\"FAKE_OK_FROM_STDOUT\", flush=True)\n    os._exit(0)\n",
        );
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());

    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let stdout = String::from_utf8(run.stdout.clone()).expect("stdout should be utf-8");
    assert!(!stdout.contains("FAKE_OK_FROM_STDOUT"));
    let envelope = assert_error_envelope(&run, "ProtocolViolation");

    let run_id = envelope["run_id"].as_str().unwrap();
    let stdout_log = fs::read_to_string(run_dir(&capsule, run_id).join("stdout.log"))
        .expect("stdout log should be readable");
    assert!(stdout_log.contains("FAKE_OK_FROM_STDOUT"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn uncategorized_action_failure_returns_runtime_error() {
    let output_root = temp_dir("error-runtime");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let action_path = capsule.join("action.py");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action.replace(
        "def run(input: Input, ctx) -> Output:\n",
        "def run(input: Input, ctx) -> Output:\n    raise RuntimeError(\"boom internal\")\n",
    );
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());

    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let envelope = assert_error_envelope(&run, "RuntimeError");

    let run_id = envelope["run_id"].as_str().unwrap();
    let stderr_log = fs::read_to_string(run_dir(&capsule, run_id).join("stderr.log"))
        .expect("stderr log should be readable");
    assert!(stderr_log.contains("Traceback"));
    assert!(!envelope["display"]["markdown"]
        .as_str()
        .unwrap()
        .contains("Traceback"));

    fs::remove_dir_all(output_root).ok();
}
