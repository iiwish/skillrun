use std::process::Command;

fn run_skillrun(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .output()
        .expect("skillrun binary should run")
}

#[test]
fn help_lists_core_commands() {
    let output = run_skillrun(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output should be utf-8");

    assert!(stdout.contains("SkillRun"));
    assert!(stdout.contains("skillrun"));

    for command in [
        "init", "manifest", "inspect", "check", "doctor", "test", "run", "serve", "pack",
    ] {
        assert!(
            stdout.contains(command),
            "help output should list planned command: {command}"
        );
    }

    assert!(stdout.contains("init --python"));
    assert!(stdout.contains("init --py"));
    assert!(stdout.contains("init --js (alpha)"));
}

#[test]
fn version_uses_approved_project_name() {
    let output = run_skillrun(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("version output should be utf-8"),
        "skillrun 0.5.4\n"
    );
}

#[test]
fn serve_requires_explicit_mcp_mode() {
    let output = run_skillrun(&["serve"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("error output should be utf-8");

    assert!(stderr.contains("serve currently requires --mcp"));
}
