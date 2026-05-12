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

fn run_skillrun_with_env(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_skillrun"));
    command.args(args);
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

fn generated_manifest(capsule: &Path) -> PathBuf {
    capsule.join(".skillrun").join("manifest.generated.yaml")
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "manifest should contain {expected:?}\n{text}"
    );
}

fn sha_lines_are_64_hex(manifest: &str) -> bool {
    let lines: Vec<&str> = manifest.lines().collect();
    lines.windows(2).any(|window| {
        window[0].trim_start().starts_with("path: SKILL.md")
            && window[1]
                .trim_start()
                .strip_prefix("sha256: ")
                .is_some_and(is_64_hex)
    }) && lines.windows(2).any(|window| {
        window[0].trim_start().starts_with("path: action.py")
            && window[1]
                .trim_start()
                .strip_prefix("sha256: ")
                .is_some_and(is_64_hex)
    }) && lines.windows(2).any(|window| {
        window[0]
            .trim_start()
            .starts_with("path: skillrun.config.json")
            && window[1]
                .trim_start()
                .strip_prefix("sha256: ")
                .is_some_and(is_64_hex)
    })
}

fn is_64_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[test]
fn manifest_generates_yaml_with_hashes_and_pydantic_schemas() {
    let output_root = temp_dir("manifest");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(manifest.status.success());
    let manifest_path = generated_manifest(&capsule);
    assert!(manifest_path.is_file());
    let manifest_text = fs::read_to_string(&manifest_path).expect("manifest should be readable");

    assert_contains(&manifest_text, "manifest_version: 0.1.0");
    assert_contains(&manifest_text, "generated_by: skillrun@0.2.0");
    assert_contains(&manifest_text, "name: refund");
    assert_contains(&manifest_text, "adapter: python");
    assert_contains(&manifest_text, "entrypoint: action.py");
    assert_contains(&manifest_text, "order_id");
    assert_contains(&manifest_text, "manager_approval_id");
    assert_contains(&manifest_text, "decision");
    assert!(sha_lines_are_64_hex(&manifest_text));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_fails_when_action_is_missing() {
    let output_root = temp_dir("manifest-missing-action");
    let capsule = output_root.join("instruction_only");
    fs::create_dir_all(&capsule).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# instruction only").expect("skill should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("missing action.py"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_metadata_extraction_times_out() {
    let output_root = temp_dir("manifest-timeout");
    let capsule = output_root.join("slow_action");
    fs::create_dir_all(&capsule).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# slow action").expect("skill should be written");
    fs::write(capsule.join("action.py"), "import time\ntime.sleep(5)\n")
        .expect("action should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun_with_env(
        &["manifest", "--cwd", &cwd_arg],
        &[("SKILLRUN_METADATA_TIMEOUT_MS", "200")],
    );

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("metadata extraction timed out"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}
