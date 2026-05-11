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

fn assert_file_contains(path: &Path, expected: &str) {
    let content = fs::read_to_string(path).expect("file should be readable");
    assert!(
        content.contains(expected),
        "{} should contain {expected:?}",
        path.display()
    );
}

#[test]
fn init_python_creates_standard_capsule() {
    let output_root = temp_dir("init-python");
    let output_arg = output_root.to_string_lossy().to_string();

    let output = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);

    assert!(output.status.success());
    let capsule = output_root.join("refund");

    assert!(capsule.join("SKILL.md").is_file());
    assert!(capsule.join("action.py").is_file());
    assert!(capsule
        .join("examples")
        .join("default.input.json")
        .is_file());
    assert!(capsule.join("skillrun.config.json").is_file());

    assert_file_contains(&capsule.join("SKILL.md"), "# refund");
    assert_file_contains(&capsule.join("SKILL.md"), "SOP");
    assert_file_contains(&capsule.join("action.py"), "class Input(BaseModel)");
    assert_file_contains(&capsule.join("action.py"), "class Output(BaseModel)");
    assert_file_contains(&capsule.join("action.py"), "def preflight");
    assert_file_contains(&capsule.join("action.py"), "def run");
    assert_file_contains(
        &capsule.join("examples").join("default.input.json"),
        "\"order_id\"",
    );
    assert_file_contains(&capsule.join("skillrun.config.json"), "\"timeout\"");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn init_refuses_non_empty_target() {
    let output_root = temp_dir("init-non-empty");
    let capsule = output_root.join("refund");
    fs::create_dir_all(&capsule).expect("test target should be created");
    fs::write(capsule.join("existing.txt"), "keep me").expect("marker should be written");
    let output_arg = output_root.to_string_lossy().to_string();

    let output = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("target directory is not empty"));
    assert_eq!(
        fs::read_to_string(capsule.join("existing.txt")).expect("marker should remain"),
        "keep me"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn init_requires_python_flag() {
    let output = run_skillrun(&["init", "refund"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("init currently requires --python"));
}

#[test]
fn init_rejects_path_like_capsule_names() {
    let output = run_skillrun(&["init", "../refund", "--python"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("invalid capsule name"));
}
