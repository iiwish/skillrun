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

fn append_to(path: &Path, text: &str) {
    let mut current = fs::read_to_string(path).expect("file should be readable");
    current.push_str(text);
    fs::write(path, current).expect("file should be writable");
}

fn assert_guard_failure(output: &std::process::Output, expected_path: &str) -> String {
    assert!(
        !output.status.success(),
        "guarded command should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8");
    assert!(
        stderr.contains("stale Manifest"),
        "stderr should explain stale Manifest\n{stderr}"
    );
    assert!(
        stderr.contains(expected_path),
        "stderr should name stale source {expected_path:?}\n{stderr}"
    );
    assert!(
        stderr.contains("skillrun manifest"),
        "stderr should include the regeneration command\n{stderr}"
    );
    assert!(
        !stderr.contains("command not implemented yet"),
        "guard should run before unimplemented fallback\n{stderr}"
    );
    stderr
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

fn assert_success_stdout(output: &std::process::Output, label: &str) -> String {
    assert!(
        output.status.success(),
        "{label} should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn assert_failure_stdout(output: &std::process::Output, label: &str) -> String {
    assert!(
        !output.status.success(),
        "{label} should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

#[test]
fn run_refuses_stale_skill_hash_before_creating_run() {
    let (output_root, capsule) = generated_capsule("guard-stale-skill");
    append_to(&capsule.join("SKILL.md"), "\n\nAdditional policy text.\n");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let run = run_skillrun(&[
        "run",
        "--cwd",
        &cwd_arg,
        "--input",
        "examples/default.input.json",
    ]);
    assert_guard_failure(&run, "SKILL.md");
    assert!(
        !capsule.join(".skillrun").join("runs").exists(),
        "stale guard should fail before creating run records"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn serve_refuses_stale_action_before_unimplemented_fallback() {
    let (output_root, capsule) = generated_capsule("guard-stale-action");
    append_to(&capsule.join("action.py"), "\n# changed after manifest\n");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert_guard_failure(&serve, "action.py");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn serve_refuses_stale_js_action_before_runtime_dispatch() {
    let (output_root, capsule) = generated_js_capsule("guard-stale-js-action");
    append_to(&capsule.join("action.mjs"), "\n// changed after manifest\n");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert_guard_failure(&serve, "action.mjs");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn js_capsule_without_manifest_is_not_reported_as_missing_python_action() {
    let output_root = temp_dir("guard-js-missing-manifest");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--js", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);

    assert!(!serve.status.success());
    let stderr = String::from_utf8(serve.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("missing Manifest"));
    assert!(stderr.contains("skillrun manifest"));
    assert!(!stderr.contains("missing action.py"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn pack_refuses_stale_config_before_archive_creation() {
    let (output_root, capsule) = generated_capsule("guard-stale-config");
    append_to(&capsule.join("skillrun.config.json"), "\n");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert_guard_failure(&pack, "skillrun.config.json");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn valid_capsule_reaches_serve_dry_run_and_pack_success() {
    let (output_root, capsule) = generated_capsule("guard-valid-unimplemented");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    let contract = assert_success_json(&serve);
    assert_eq!(contract["mcp"]["dry_run"], true);
    assert_eq!(contract["tools"][0]["name"], "refund");

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert!(pack.status.success());
    let pack_stdout = String::from_utf8(pack.stdout).expect("stdout should be utf-8");
    assert!(pack_stdout.contains("refund-0.4.1.skr"));
    assert!(pack_stdout.contains("does not vendor dependencies"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn doctor_reports_valid_python_and_js_capsules_without_language_flags() {
    let (python_root, python_capsule) = generated_capsule("doctor-valid-python");
    let python_cwd = python_capsule.to_string_lossy().to_string();
    let python = run_skillrun(&["doctor", "--cwd", &python_cwd]);
    let python_stdout = assert_success_stdout(&python, "python doctor");

    for expected in [
        "SkillRun Doctor",
        "status: ok",
        "adapter: python",
        "entrypoint: action.py",
        "manifest freshness: fresh",
        "examples/default.input.json: present",
    ] {
        assert!(
            python_stdout.contains(expected),
            "python doctor missing {expected:?}\n{python_stdout}"
        );
    }
    assert!(!python_stdout.contains("--python"));
    assert!(!python_stdout.contains("--js"));

    let (js_root, js_capsule) = generated_js_capsule("doctor-valid-js");
    let js_cwd = js_capsule.to_string_lossy().to_string();
    let js = run_skillrun(&["doctor", "--cwd", &js_cwd]);
    let js_stdout = assert_success_stdout(&js, "JS doctor");

    for expected in [
        "status: ok",
        "adapter: node",
        "entrypoint: action.mjs",
        "manifest freshness: fresh",
        "examples/default.input.json: present",
    ] {
        assert!(
            js_stdout.contains(expected),
            "JS doctor missing {expected:?}\n{js_stdout}"
        );
    }
    assert!(!js_stdout.contains("--python"));
    assert!(!js_stdout.contains("--js"));

    fs::remove_dir_all(python_root).ok();
    fs::remove_dir_all(js_root).ok();
}

#[test]
fn check_reports_valid_python_and_js_capsules_without_language_flags() {
    let (python_root, python_capsule) = generated_capsule("check-valid-python");
    let python_cwd = python_capsule.to_string_lossy().to_string();
    let python = run_skillrun(&["check", "--cwd", &python_cwd]);
    let python_stdout = assert_success_stdout(&python, "python check");

    for expected in [
        "SkillRun Check",
        "status: ok",
        "adapter: python",
        "entrypoint: action.py",
        "manifest freshness: fresh",
        "executable: python >=3.10",
        "package: pydantic >=2,<3",
        "executable: python required: >=3.10 detected:",
        "package: pydantic required: >=2,<3 detected:",
        "status: satisfied",
        "examples/default.input.json: present",
    ] {
        assert!(
            python_stdout.contains(expected),
            "python check missing {expected:?}\n{python_stdout}"
        );
    }
    assert!(!python_stdout.contains("--python"));
    assert!(!python_stdout.contains("--js"));

    let (js_root, js_capsule) = generated_js_capsule("check-valid-js");
    let js_cwd = js_capsule.to_string_lossy().to_string();
    let js = run_skillrun(&["check", "--cwd", &js_cwd]);
    let js_stdout = assert_success_stdout(&js, "JS check");

    for expected in [
        "status: ok",
        "adapter: node",
        "entrypoint: action.mjs",
        "manifest freshness: fresh",
        "executable: node >=18",
        "executable: node required: >=18 detected:",
        "packages: none",
        "examples/default.input.json: present",
    ] {
        assert!(
            js_stdout.contains(expected),
            "JS check missing {expected:?}\n{js_stdout}"
        );
    }
    assert!(!js_stdout.contains("--python"));
    assert!(!js_stdout.contains("--js"));

    fs::remove_dir_all(python_root).ok();
    fs::remove_dir_all(js_root).ok();
}

#[test]
fn check_reports_missing_python_without_raw_program_not_found() {
    let (output_root, capsule) = generated_capsule("check-missing-python");
    let fake_path = output_root.join("empty-path");
    fs::create_dir_all(&fake_path).expect("empty PATH dir should be created");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let check = run_skillrun_with_path(&["check", "--cwd", &cwd_arg], &fake_path);
    let stdout = assert_failure_stdout(&check, "missing python check");

    for expected in [
        "status: dependency-error",
        "executable: python required: >=3.10 detected: missing status: missing",
        "package: pydantic required: >=2,<3 detected: missing status: missing",
    ] {
        assert!(
            stdout.contains(expected),
            "missing python check missing {expected:?}\n{stdout}"
        );
    }
    assert!(
        !stdout.contains("program not found"),
        "check should hide raw spawn wording\n{stdout}"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn check_reports_missing_node_without_package_manager_checks() {
    let (output_root, capsule) = generated_js_capsule("check-missing-node");
    let fake_path = output_root.join("empty-path");
    fs::create_dir_all(&fake_path).expect("empty PATH dir should be created");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let check = run_skillrun_with_path(&["check", "--cwd", &cwd_arg], &fake_path);
    let stdout = assert_failure_stdout(&check, "missing node check");

    for expected in [
        "status: dependency-error",
        "executable: node required: >=18 detected: missing status: missing",
        "packages: none",
    ] {
        assert!(
            stdout.contains(expected),
            "missing node check missing {expected:?}\n{stdout}"
        );
    }
    assert!(!stdout.contains("npm"));
    assert!(!stdout.contains("node_modules"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn check_reports_missing_pydantic_without_importing_action_source() {
    let (output_root, capsule) = generated_capsule("check-missing-pydantic");
    let fake_path = output_root.join("fake-path");
    write_fake_python(&fake_path, None);
    let fake_log = output_root.join("fake-python.log");
    let fake_log_arg = fake_log.to_string_lossy().to_string();

    let cwd_arg = capsule.to_string_lossy().to_string();
    let check = run_skillrun_with_path_and_env(
        ["check", "--cwd", &cwd_arg].as_slice(),
        &fake_path,
        &[("SKILLRUN_FAKE_PYTHON_LOG", &fake_log_arg)],
    );
    let stdout = assert_failure_stdout(&check, "missing pydantic check");

    for expected in [
        "status: dependency-error",
        "executable: python required: >=3.10 detected: Python 3.11.0 status: satisfied",
        "package: pydantic required: >=2,<3 detected: missing status: missing",
    ] {
        assert!(
            stdout.contains(expected),
            "missing pydantic check missing {expected:?}\n{stdout}"
        );
    }

    let fake_log = fs::read_to_string(fake_log).expect("fake python log should be readable");
    assert!(fake_log.contains("pydantic"));
    assert!(
        !fake_log.contains("action.py"),
        "pydantic probe must not receive action source path\n{fake_log}"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn check_reports_incompatible_pydantic_version() {
    let (output_root, capsule) = generated_capsule("check-pydantic-v1");
    let fake_path = output_root.join("fake-path");
    write_fake_python(&fake_path, Some("1.10.0"));

    let cwd_arg = capsule.to_string_lossy().to_string();
    let check = run_skillrun_with_path(&["check", "--cwd", &cwd_arg], &fake_path);
    let stdout = assert_failure_stdout(&check, "pydantic v1 check");

    for expected in [
        "status: dependency-error",
        "package: pydantic required: >=2,<3 detected: 1.10.0 status: unsupported-version",
    ] {
        assert!(
            stdout.contains(expected),
            "pydantic v1 check missing {expected:?}\n{stdout}"
        );
    }

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn doctor_reports_stale_manifest_without_creating_run_records() {
    let (output_root, capsule) = generated_js_capsule("doctor-stale-js");
    append_to(&capsule.join("action.mjs"), "\n// changed before doctor\n");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let doctor = run_skillrun(&["doctor", "--cwd", &cwd_arg]);
    let stdout = assert_failure_stdout(&doctor, "stale JS doctor");

    for expected in [
        "status: stale-manifest",
        "manifest freshness: stale",
        "action.mjs: stale",
        "skillrun manifest",
    ] {
        assert!(
            stdout.contains(expected),
            "stale doctor missing {expected:?}\n{stdout}"
        );
    }
    assert!(
        !capsule.join(".skillrun").join("runs").exists(),
        "doctor must not create run records"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn check_reports_stale_manifest_without_creating_run_records() {
    let (output_root, capsule) = generated_js_capsule("check-stale-js");
    append_to(&capsule.join("action.mjs"), "\n// changed before check\n");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let check = run_skillrun(&["check", "--cwd", &cwd_arg]);
    let stdout = assert_failure_stdout(&check, "stale JS check");

    for expected in [
        "SkillRun Check",
        "status: stale-manifest",
        "manifest freshness: stale",
        "action.mjs: stale",
        "skillrun manifest",
    ] {
        assert!(
            stdout.contains(expected),
            "stale check missing {expected:?}\n{stdout}"
        );
    }
    assert!(
        !capsule.join(".skillrun").join("runs").exists(),
        "check must not create run records"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn doctor_does_not_import_js_action_for_metadata() {
    let (output_root, capsule) = generated_js_capsule("doctor-js-no-import");
    let marker = output_root.join("doctor-import-marker.txt");
    let marker_literal = serde_json::to_string(&marker.to_string_lossy().to_string())
        .expect("marker should serialize");
    let action_path = capsule.join("action.mjs");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = format!(
        "import fs from \"node:fs\";\nfs.writeFileSync({marker_literal}, \"imported\", \"utf8\");\n{action}"
    );
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());
    assert!(
        marker.is_file(),
        "manifest metadata extraction imports action.mjs in Author Mode"
    );
    fs::remove_file(&marker).expect("marker should be removed before doctor");

    let doctor = run_skillrun(&["doctor", "--cwd", &cwd_arg]);
    assert_success_stdout(&doctor, "JS doctor no import");
    assert!(
        !marker.exists(),
        "doctor must not import action.mjs for metadata"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn check_does_not_import_js_action_for_metadata() {
    let (output_root, capsule) = generated_js_capsule("check-js-no-import");
    let marker = output_root.join("check-import-marker.txt");
    let marker_literal = serde_json::to_string(&marker.to_string_lossy().to_string())
        .expect("marker should serialize");
    let action_path = capsule.join("action.mjs");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = format!(
        "import fs from \"node:fs\";\nfs.writeFileSync({marker_literal}, \"imported\", \"utf8\");\n{action}"
    );
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());
    assert!(
        marker.is_file(),
        "manifest metadata extraction imports action.mjs in Author Mode"
    );
    fs::remove_file(&marker).expect("marker should be removed before check");

    let check = run_skillrun(&["check", "--cwd", &cwd_arg]);
    assert_success_stdout(&check, "JS check no import");
    assert!(
        !marker.exists(),
        "check must not import action.mjs for metadata"
    );

    fs::remove_dir_all(output_root).ok();
}
