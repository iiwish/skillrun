use std::process::Command;

fn run_skillrun(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .output()
        .expect("skillrun binary should run")
}

#[test]
fn help_lists_planned_mvp_commands() {
    let output = run_skillrun(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output should be utf-8");

    assert!(stdout.contains("SkillRun"));
    assert!(stdout.contains("skillrun"));

    for command in [
        "init", "manifest", "inspect", "test", "run", "serve", "pack",
    ] {
        assert!(
            stdout.contains(command),
            "help output should list planned command: {command}"
        );
    }
}

#[test]
fn version_uses_approved_project_name() {
    let output = run_skillrun(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("version output should be utf-8"),
        "skillrun 0.1.0\n"
    );
}

#[test]
fn planned_commands_fail_until_implemented() {
    let output = run_skillrun(&["init"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("error output should be utf-8");

    assert!(stderr.contains("command not implemented yet: init"));
}
