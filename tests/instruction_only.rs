use std::fs;
use std::path::PathBuf;
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

fn instruction_only_skill(label: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let skill = output_root.join("plain-skill");
    fs::create_dir_all(skill.join("scripts")).expect("scripts dir should be created");
    fs::create_dir_all(skill.join("examples")).expect("examples dir should be created");
    fs::create_dir_all(skill.join("references")).expect("references dir should be created");
    fs::write(
        skill.join("SKILL.md"),
        "# Plain Skill\n\n```python\nprint('do not infer me')\n```\n",
    )
    .expect("SKILL.md should be written");
    fs::write(
        skill.join("scripts").join("run.py"),
        "print('not an action')\n",
    )
    .expect("script should be written");
    fs::write(skill.join("examples").join("default.input.json"), "{}\n")
        .expect("example should be written");
    fs::write(skill.join("references").join("policy.md"), "policy text\n")
        .expect("reference should be written");
    (output_root, skill)
}

fn assert_instruction_refusal(output: &std::process::Output, command: &str) -> String {
    assert!(
        !output.status.success(),
        "{command} should refuse instruction-only Skill\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8");
    assert!(
        stderr.contains("instruction-only"),
        "{command} stderr should name instruction-only status\n{stderr}"
    );
    assert!(
        stderr.contains("action.py"),
        "{command} stderr should explain explicit action.py is required\n{stderr}"
    );
    assert!(
        stderr.contains("skillrun manifest"),
        "{command} stderr should include the Manifest generation next step\n{stderr}"
    );
    assert!(
        !stderr.contains("command not implemented yet"),
        "{command} guard should run before unimplemented fallback\n{stderr}"
    );
    stderr
}

#[test]
fn inspect_reports_instruction_only_status_with_scripts_and_examples_present() {
    let (output_root, skill) = instruction_only_skill("instruction-inspect");
    let cwd_arg = skill.to_string_lossy().to_string();

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd_arg]);
    assert!(
        inspect.status.success(),
        "inspect should succeed\nstderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let stdout = String::from_utf8(inspect.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("status: instruction-only"));
    assert!(stdout.contains("missing action.py"));
    assert!(stdout.contains("missing .skillrun/manifest.generated.yaml"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_refuses_to_infer_actions_from_markdown_or_scripts() {
    let (output_root, skill) = instruction_only_skill("instruction-manifest");
    let cwd_arg = skill.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("missing action.py"));
    assert!(stderr.contains("does not infer actions"));
    assert!(stderr.contains("Markdown"));
    assert!(!skill
        .join(".skillrun")
        .join("manifest.generated.yaml")
        .exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn run_serve_and_pack_refuse_instruction_only_skill() {
    let (output_root, skill) = instruction_only_skill("instruction-consumer");
    let cwd_arg = skill.to_string_lossy().to_string();

    let run = run_skillrun(&[
        "run",
        "--cwd",
        &cwd_arg,
        "--input",
        "examples/default.input.json",
    ]);
    assert_instruction_refusal(&run, "run");

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert_instruction_refusal(&serve, "serve");

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert_instruction_refusal(&pack, "pack");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn missing_action_with_manifest_is_not_treated_as_runnable() {
    let output_root = temp_dir("instruction-missing-action");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(manifest.status.success());
    fs::remove_file(capsule.join("action.py")).expect("action should be removed");

    let run = run_skillrun(&[
        "run",
        "--cwd",
        &cwd_arg,
        "--input",
        "examples/default.input.json",
    ]);
    assert!(!run.status.success());
    let stderr = String::from_utf8(run.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("stale Manifest"));
    assert!(stderr.contains("action.py"));

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd_arg]);
    assert!(inspect.status.success());
    let stdout = String::from_utf8(inspect.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("status: invalid-runnable"));
    assert!(stdout.contains("action.py"));

    fs::remove_dir_all(output_root).ok();
}
