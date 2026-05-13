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

fn run_skillrun_with_path(args: &[&str], path: &Path) -> std::process::Output {
    run_skillrun_with_path_and_env(args, path, &[])
}

fn run_skillrun_with_path_and_env(
    args: &[&str],
    path: &Path,
    envs: &[(&str, &str)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_skillrun"));
    command.args(args).env("PATH", path);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("skillrun binary should run")
}

fn temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("skillrun-{label}-{}-{stamp}", std::process::id()))
}

fn write_fake_python(fake_dir: &Path, pydantic_version: Option<&str>) {
    fs::create_dir_all(fake_dir).expect("fake runtime dir should be created");
    let pydantic_branch = match pydantic_version {
        Some(version) => format!("echo {version}"),
        None => "echo ModuleNotFoundError: No module named pydantic 1>&2\nexit /b 1".to_string(),
    };

    #[cfg(windows)]
    {
        let script = format!(
            "@echo off\r\n\
if not \"%SKILLRUN_FAKE_PYTHON_LOG%\"==\"\" echo %*>>\"%SKILLRUN_FAKE_PYTHON_LOG%\"\r\n\
if \"%1\"==\"--version\" (\r\n\
  echo Python 3.11.0\r\n\
  exit /b 0\r\n\
)\r\n\
{pydantic_branch}\r\n\
exit /b 0\r\n"
        );
        fs::write(fake_dir.join("python.cmd"), script).expect("fake python should be written");
    }

    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;

        let pydantic_branch = match pydantic_version {
            Some(version) => format!("echo {version}"),
            None => "echo 'ModuleNotFoundError: No module named pydantic' >&2\nexit 1".to_string(),
        };
        let script = format!(
            "#!/bin/sh\n\
if [ -n \"$SKILLRUN_FAKE_PYTHON_LOG\" ]; then printf '%s\\n' \"$*\" >> \"$SKILLRUN_FAKE_PYTHON_LOG\"; fi\n\
if [ \"$1\" = \"--version\" ]; then\n\
  echo 'Python 3.11.0'\n\
  exit 0\n\
fi\n\
{pydantic_branch}\n"
        );
        let path = fake_dir.join("python");
        fs::write(&path, script).expect("fake python should be written");
        let mut permissions = fs::metadata(&path)
            .expect("fake python metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("fake python should be executable");
    }
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "text should contain {expected:?}\n{text}"
    );
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

fn generated_js_capsule(label: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();

    let init = run_skillrun(&["init", "refund", "--js", "--output", &output_arg]);
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

fn generated_capsule_under(output_root: &Path) -> PathBuf {
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

    capsule
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).expect("json file should be readable");
    serde_json::from_str(&text).expect("json file should parse")
}

fn run_dir(capsule: &Path, run_id: &str) -> PathBuf {
    capsule.join(".skillrun").join("runs").join(run_id)
}

fn run_id_from(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "command should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    let envelope: Value = serde_json::from_str(&stdout).expect("stdout should be a JSON envelope");
    assert_eq!(envelope["ok"], true);
    envelope["run_id"]
        .as_str()
        .expect("run_id should be present")
        .to_string()
}

fn is_64_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn assert_dependency_error(output: &std::process::Output, expected: &[&str]) -> Value {
    assert!(
        !output.status.success(),
        "command should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    let envelope: Value = serde_json::from_str(&stdout).expect("stdout should be a JSON envelope");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "DependencyError");
    assert_eq!(envelope["error"]["recoverable"], true);
    assert!(envelope["display"]["markdown"]
        .as_str()
        .expect("display markdown should be present")
        .contains("declared runtime requirements"));
    for expected in expected {
        assert!(
            stdout.contains(expected),
            "DependencyError envelope missing {expected:?}\n{stdout}"
        );
    }
    envelope
}

#[test]
fn runtime_rejects_unknown_adapter_before_creating_run_records() {
    let (output_root, capsule) = generated_capsule("runtime-unsupported-adapter");
    let manifest_path = capsule.join(".skillrun").join("manifest.generated.yaml");
    let manifest = fs::read_to_string(&manifest_path).expect("manifest should be readable");
    let manifest = manifest.replace("adapter: python", "adapter: ruby");
    fs::write(&manifest_path, manifest).expect("manifest should be writable");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let output = run_skillrun(&["test", "--cwd", &cwd_arg]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert_contains(&stderr, "unsupported runtime adapter: ruby");
    assert!(
        !capsule.join(".skillrun").join("runs").exists(),
        "unsupported adapter should fail before creating run records"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn test_returns_dependency_error_when_python_is_missing() {
    let (output_root, capsule) = generated_capsule("runtime-missing-python");
    let fake_path = output_root.join("empty-path");
    fs::create_dir_all(&fake_path).expect("empty PATH dir should be created");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let output = run_skillrun_with_path(&["test", "--cwd", &cwd_arg], &fake_path);
    let envelope = assert_dependency_error(&output, &["python", ">=3.10", "missing"]);
    let run_id = envelope["run_id"]
        .as_str()
        .expect("run_id should be present");
    let dir = run_dir(&capsule, run_id);
    assert!(dir.join("record.json").is_file());
    let record = read_json(&dir.join("record.json"));
    assert_eq!(record["status"], "failed");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn test_returns_dependency_error_when_node_is_missing() {
    let (output_root, capsule) = generated_js_capsule("runtime-missing-node");
    let fake_path = output_root.join("empty-path");
    fs::create_dir_all(&fake_path).expect("empty PATH dir should be created");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let output = run_skillrun_with_path(&["test", "--cwd", &cwd_arg], &fake_path);
    assert_dependency_error(&output, &["node", ">=18", "missing"]);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn run_returns_dependency_error_when_pydantic_is_missing_without_importing_action() {
    let (output_root, capsule) = generated_capsule("runtime-missing-pydantic");
    let fake_path = output_root.join("fake-path");
    write_fake_python(&fake_path, None);
    let fake_log = output_root.join("fake-python.log");
    let fake_log_arg = fake_log.to_string_lossy().to_string();

    let cwd_arg = capsule.to_string_lossy().to_string();
    let output = run_skillrun_with_path_and_env(
        &[
            "run",
            "--cwd",
            &cwd_arg,
            "--input",
            "examples/default.input.json",
        ],
        &fake_path,
        &[("SKILLRUN_FAKE_PYTHON_LOG", &fake_log_arg)],
    );
    assert_dependency_error(&output, &["pydantic", ">=2,<3", "missing"]);

    let fake_log = fs::read_to_string(fake_log).expect("fake python log should be readable");
    assert!(fake_log.contains("pydantic"));
    assert!(
        !fake_log.contains("action.py"),
        "runtime dependency precheck must not import action source\n{fake_log}"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn stale_manifest_fails_before_dependency_precheck() {
    let (output_root, capsule) = generated_js_capsule("runtime-stale-before-dependency");
    let fake_path = output_root.join("empty-path");
    fs::create_dir_all(&fake_path).expect("empty PATH dir should be created");
    let action_path = capsule.join("action.mjs");
    let mut action = fs::read_to_string(&action_path).expect("action should be readable");
    action.push_str("\n// stale before dependency precheck\n");
    fs::write(&action_path, action).expect("action should be writable");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let output = run_skillrun_with_path(&["test", "--cwd", &cwd_arg], &fake_path);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert_contains(&stderr, "stale Manifest");
    assert_contains(&stderr, "action.mjs");
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("DependencyError"),
        "stale Manifest should fail before dependency envelope"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn js_test_command_runs_action_and_writes_run_record() {
    let (output_root, capsule) = generated_js_capsule("runtime-js-test-command");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let output = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let run_id = run_id_from(&output);
    let dir = run_dir(&capsule, &run_id);

    let output_json = read_json(&dir.join("output.json"));
    assert_eq!(output_json["ok"], true);
    assert_eq!(output_json["output"]["decision"], "approved");
    assert_eq!(output_json["output"]["amount"], 120);

    let record = read_json(&dir.join("record.json"));
    assert_eq!(record["run_id"], run_id);
    assert_eq!(record["mode"], "test");
    assert_eq!(record["status"], "succeeded");
    assert!(is_64_hex(record["manifest_sha256"].as_str().unwrap()));
    assert!(is_64_hex(record["skill_sha256"].as_str().unwrap()));
    assert!(is_64_hex(record["action_sha256"].as_str().unwrap()));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn js_sync_run_function_is_supported() {
    let (output_root, capsule) = generated_js_capsule("runtime-js-sync-run");
    let action_path = capsule.join("action.mjs");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action.replace("export async function run", "export function run");
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());

    let output = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let run_id = run_id_from(&output);
    let output_json = read_json(&run_dir(&capsule, &run_id).join("output.json"));
    assert_eq!(output_json["ok"], true);
    assert_eq!(output_json["output"]["decision"], "approved");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn test_command_uses_default_example_and_writes_run_record() {
    let (output_root, capsule) = generated_capsule("runtime-test-command");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let output = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let run_id = run_id_from(&output);
    let dir = run_dir(&capsule, &run_id);

    assert!(dir.join("input.json").is_file());
    assert!(dir.join("context.json").is_file());
    assert!(dir.join("output.json").is_file());
    assert!(dir.join("stdout.log").is_file());
    assert!(dir.join("stderr.log").is_file());
    assert!(dir.join("artifacts").is_dir());
    assert!(dir.join("record.json").is_file());

    let output_json = read_json(&dir.join("output.json"));
    assert_eq!(output_json["ok"], true);
    assert_eq!(output_json["output"]["decision"], "approved");
    assert_eq!(output_json["output"]["amount"], 120);

    let record = read_json(&dir.join("record.json"));
    assert_eq!(record["run_id"], run_id);
    assert_eq!(record["mode"], "test");
    assert_eq!(record["status"], "succeeded");
    assert!(is_64_hex(record["manifest_sha256"].as_str().unwrap()));
    assert!(is_64_hex(record["skill_sha256"].as_str().unwrap()));
    assert!(is_64_hex(record["action_sha256"].as_str().unwrap()));
    assert!(record["started_at"].as_str().unwrap().contains('T'));
    assert!(record["finished_at"].as_str().unwrap().contains('T'));
    assert_contains(&record.to_string(), ".skillrun/runs/**");

    let context = read_json(&dir.join("context.json"));
    assert_eq!(context["run_id"], run_id);
    assert_eq!(context["mode"], "test");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn run_command_uses_explicit_input_and_unique_run_ids() {
    let (output_root, capsule) = generated_capsule("runtime-run-command");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let first = run_skillrun(&[
        "run",
        "--cwd",
        &cwd_arg,
        "--input",
        "examples/default.input.json",
    ]);
    let second = run_skillrun(&[
        "run",
        "--cwd",
        &cwd_arg,
        "--input",
        "examples/default.input.json",
    ]);

    let first_id = run_id_from(&first);
    let second_id = run_id_from(&second);

    assert_ne!(first_id, second_id);
    assert!(run_dir(&capsule, &first_id).is_dir());
    assert!(run_dir(&capsule, &second_id).is_dir());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn adapter_stdout_is_captured_as_log_not_result() {
    let output_root = temp_dir("runtime-stdout-log");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let action_path = capsule.join("action.py");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action.replace(
        "def run(input: Input, ctx) -> Output:\n",
        "def run(input: Input, ctx) -> Output:\n    print(\"adapter stdout noise\")\n",
    );
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());

    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let stdout = String::from_utf8(run.stdout.clone()).expect("stdout should be utf-8");
    assert!(!stdout.contains("adapter stdout noise"));
    let run_id = run_id_from(&run);
    let stdout_log = fs::read_to_string(run_dir(&capsule, &run_id).join("stdout.log"))
        .expect("stdout log should be readable");
    assert_contains(&stdout_log, "adapter stdout noise");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn js_adapter_stdout_is_captured_as_log_not_result() {
    let (output_root, capsule) = generated_js_capsule("runtime-js-stdout-log");
    let action_path = capsule.join("action.mjs");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action.replace(
        "export async function run(input, ctx) {\n",
        "export async function run(input, ctx) {\n  console.log(\"adapter stdout noise\");\n",
    );
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());

    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let stdout = String::from_utf8(run.stdout.clone()).expect("stdout should be utf-8");
    assert!(!stdout.contains("adapter stdout noise"));
    let run_id = run_id_from(&run);
    let stdout_log = fs::read_to_string(run_dir(&capsule, &run_id).join("stdout.log"))
        .expect("stdout log should be readable");
    assert_contains(&stdout_log, "adapter stdout noise");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn runtime_accepts_relative_cwd_paths() {
    let output_root = PathBuf::from("target")
        .join("skillrun-runtime-tests")
        .join(format!("relative-{}", std::process::id()));
    fs::remove_dir_all(&output_root).ok();
    fs::create_dir_all(&output_root).expect("relative output root should be created");
    let capsule = generated_capsule_under(&output_root);
    let cwd_arg = capsule.to_string_lossy().to_string();

    let output = run_skillrun(&["test", "--cwd", &cwd_arg]);
    let run_id = run_id_from(&output);

    assert!(run_dir(&capsule, &run_id).join("input.json").is_file());

    fs::remove_dir_all(output_root).ok();
}
