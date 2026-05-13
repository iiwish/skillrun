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

fn manifest_path(capsule: &Path) -> PathBuf {
    capsule.join(".skillrun").join("manifest.generated.yaml")
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "output should contain {expected:?}\n{text}"
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

#[test]
fn inspect_runnable_capsule_summarizes_manifest_contract() {
    let (output_root, capsule) = generated_capsule("inspect-runnable");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd_arg]);

    assert!(
        inspect.status.success(),
        "inspect should succeed\nstderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let stdout = String::from_utf8(inspect.stdout).expect("inspect output should be utf-8");

    for expected in [
        "SkillRun Inspect",
        "status: runnable",
        "name: refund",
        "SOP hash:",
        "action hash:",
        "input schema: present",
        "output schema: present",
        "runtime contract: Manifest adapter and entrypoint",
        "adapter: python",
        "entrypoint: action.py",
        ".skillrun/runs/**",
        "examples/default.input.json",
        "preflight: present",
        "MCP tool: refund",
    ] {
        assert_contains(&stdout, expected);
    }

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn inspect_js_capsule_summarizes_manifest_adapter_and_entrypoint() {
    let (output_root, capsule) = generated_js_capsule("inspect-js-runnable");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd_arg]);

    assert!(
        inspect.status.success(),
        "inspect should succeed\nstderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let stdout = String::from_utf8(inspect.stdout).expect("inspect output should be utf-8");

    for expected in [
        "SkillRun Inspect",
        "status: runnable",
        "runtime contract: Manifest adapter and entrypoint",
        "adapter: node",
        "entrypoint: action.mjs",
        "preflight: present",
        "MCP tool: refund",
    ] {
        assert_contains(&stdout, expected);
    }

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn inspect_instruction_only_skill_stays_non_runnable() {
    let output_root = temp_dir("inspect-instruction-only");
    let skill_dir = output_root.join("instruction_only");
    fs::create_dir_all(skill_dir.join("scripts")).expect("scripts dir should be created");
    fs::create_dir_all(skill_dir.join("assets")).expect("assets dir should be created");
    fs::create_dir_all(skill_dir.join("references")).expect("references dir should be created");
    fs::write(skill_dir.join("SKILL.md"), "# Instruction-only Skill\n")
        .expect("SKILL.md should be written");
    fs::write(
        skill_dir.join("scripts").join("helper.py"),
        "print('helper')\n",
    )
    .expect("helper should be written");

    let cwd_arg = skill_dir.to_string_lossy().to_string();
    let inspect = run_skillrun(&["inspect", "--cwd", &cwd_arg]);

    assert!(
        inspect.status.success(),
        "instruction-only inspect should succeed\nstderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let stdout = String::from_utf8(inspect.stdout).expect("inspect output should be utf-8");

    for expected in [
        "SkillRun Inspect",
        "status: instruction-only",
        "not a runnable capsule",
        "missing action.py",
        "missing .skillrun/manifest.generated.yaml",
    ] {
        assert_contains(&stdout, expected);
    }
    assert!(!manifest_path(&skill_dir).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn inspect_does_not_import_or_execute_action_source() {
    let (output_root, capsule) = generated_capsule("inspect-no-import");
    fs::write(
        capsule.join("action.py"),
        "raise RuntimeError('inspect imported action source')\n",
    )
    .expect("action should be replaced after manifest generation");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd_arg]);

    assert!(
        inspect.status.success(),
        "inspect should read the Manifest without importing action.py\nstderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let stdout = String::from_utf8(inspect.stdout).expect("inspect output should be utf-8");
    assert_contains(&stdout, "status: invalid-runnable");
    assert_contains(&stdout, "stale Manifest");
    assert_contains(&stdout, "action.py");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn inspect_js_capsule_does_not_import_or_execute_action_source() {
    let (output_root, capsule) = generated_js_capsule("inspect-js-no-import");
    fs::write(
        capsule.join("action.mjs"),
        "throw new Error('inspect imported action source');\n",
    )
    .expect("action should be replaced after manifest generation");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd_arg]);

    assert!(
        inspect.status.success(),
        "inspect should read the Manifest without importing action.mjs\nstderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let stdout = String::from_utf8(inspect.stdout).expect("inspect output should be utf-8");
    assert_contains(&stdout, "status: invalid-runnable");
    assert_contains(&stdout, "stale Manifest");
    assert_contains(&stdout, "action.mjs");

    fs::remove_dir_all(output_root).ok();
}
