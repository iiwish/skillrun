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
fn pack_refuses_stale_config_before_unimplemented_fallback() {
    let (output_root, capsule) = generated_capsule("guard-stale-config");
    append_to(&capsule.join("skillrun.config.json"), "\n");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert_guard_failure(&pack, "skillrun.config.json");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn valid_capsule_reaches_serve_and_pack_unimplemented_fallbacks() {
    let (output_root, capsule) = generated_capsule("guard-valid-unimplemented");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert!(!serve.status.success());
    let serve_stderr = String::from_utf8(serve.stderr).expect("stderr should be utf-8");
    assert!(serve_stderr.contains("command not implemented yet: serve --mcp"));

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert!(!pack.status.success());
    let pack_stderr = String::from_utf8(pack.stderr).expect("stderr should be utf-8");
    assert!(pack_stderr.contains("command not implemented yet: pack"));

    fs::remove_dir_all(output_root).ok();
}
