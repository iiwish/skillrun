use flate2::read::GzDecoder;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Archive;

fn run_skillrun(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .env_remove("WECOM_WEBHOOK_URL")
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

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("target directory should be created");
    for entry in fs::read_dir(source).expect("source directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).expect("file should copy");
        }
    }
}

fn output_text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("output should be utf-8")
}

fn assert_success(output: &std::process::Output, label: &str) -> String {
    assert!(
        output.status.success(),
        "{label} should succeed\nstdout: {}\nstderr: {}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
    output_text(&output.stdout)
}

fn error_envelope(output: &std::process::Output, code: &str) -> Value {
    assert!(
        !output.status.success(),
        "command should fail with {code}\nstdout: {}\nstderr: {}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
    let envelope: Value =
        serde_json::from_str(&output_text(&output.stdout)).expect("stdout should be JSON");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], code);
    envelope
}

fn unpack_archive(path: &Path, target: &Path) {
    let file = fs::File::open(path).expect("archive should be readable");
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(target).expect("archive should unpack");
}

#[test]
fn refund_hero_example_proves_business_value_end_to_end() {
    let output_root = temp_dir("business-refund");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/refund");
    let capsule = output_root.join("refund");
    copy_dir(&source, &capsule);
    let cwd = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd]);
    assert_success(&manifest, "manifest");

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd]);
    let inspect_stdout = assert_success(&inspect, "inspect");
    assert!(inspect_stdout.contains("MCP tool: refund"));

    let test = run_skillrun(&["test", "--cwd", &cwd]);
    let test_stdout = assert_success(&test, "test");
    let test_envelope: Value = serde_json::from_str(&test_stdout).expect("test JSON");
    assert_eq!(test_envelope["ok"], true);
    assert_eq!(test_envelope["artifacts"][0]["kind"], "markdown");
    let artifact_path = PathBuf::from(test_envelope["run_dir"].as_str().unwrap())
        .join("artifacts")
        .join(test_envelope["artifacts"][0]["path"].as_str().unwrap());
    assert!(
        artifact_path.is_file(),
        "refund receipt artifact should exist"
    );

    let policy = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        "examples/policy_violation.input.json",
    ]);
    let policy_envelope = error_envelope(&policy, "PolicyViolation");
    assert_eq!(policy_envelope["error"]["recoverable"], true);
    assert!(policy_envelope["error"]["llm_hint"]
        .as_str()
        .unwrap()
        .contains("approval"));

    let invalid = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        "examples/invalid.input.json",
    ]);
    let invalid_envelope = error_envelope(&invalid, "ValidationError");
    assert_eq!(invalid_envelope["error"]["recoverable"], true);

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd, "--dry-run"]);
    let serve_stdout = assert_success(&serve, "serve dry-run");
    let contract: Value = serde_json::from_str(&serve_stdout).expect("MCP JSON");
    assert_eq!(contract["tools"][0]["name"], "refund");
    assert!(contract["resources"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Approval Boundary"));

    let pack = run_skillrun(&["pack", "--cwd", &cwd]);
    assert_success(&pack, "pack");
    let archive_path = capsule.join("dist").join("refund-0.4.2.skr");
    assert!(archive_path.is_file());
    let unpacked = output_root.join("unpacked");
    fs::create_dir_all(&unpacked).expect("unpack dir should be created");
    unpack_archive(&archive_path, &unpacked);
    let unpacked_inspect = run_skillrun(&["inspect", "--cwd", &unpacked.to_string_lossy()]);
    assert_success(&unpacked_inspect, "unpacked inspect");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn docs_explain_b001_to_b004_without_expanding_v0_runtime_scope() {
    let docs = fs::read_to_string("docs/business-examples.md").expect("business docs readable");
    let readme = fs::read_to_string("README.md").expect("README readable");
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml readable");

    for expected in [
        "B001: Refund Decision",
        "B002: Support Triage",
        "B003: Access Request Approval",
        "B004: Vendor Risk Review",
        "docs-level example",
        "does not vendor dependencies",
    ] {
        assert!(
            docs.contains(expected) || readme.contains(expected),
            "business narrative should mention {expected}"
        );
    }

    assert!(
        docs.contains("v0.1 MVP only implements the refund capsule")
            || docs.contains("v0.1 MVP 只要求完整实现 `refund`")
    );
    assert!(readme.contains("Support Triage"));
    assert!(readme.contains("Access Request Approval"));
    assert!(readme.contains("Vendor Risk Review"));
    assert!(readme.contains("portable Agent skill"));
    assert!(readme.contains("FastMCP turns functions into MCP tools"));
    assert!(!readme.contains("tested MCP skill package"));
    assert!(!cargo_toml.contains("tested MCP skill package"));
}

#[test]
fn wecom_team_notice_example_runs_locally_without_real_webhook() {
    let output_root = temp_dir("business-wecom");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/wecom_team_notice");
    let capsule = output_root.join("wecom_team_notice");
    copy_dir(&source, &capsule);
    let cwd = capsule.to_string_lossy().to_string();

    let direct_missing_webhook =
        run_skillrun(&["run", "--cwd", &cwd, "--input", "examples/send.input.json"]);
    let direct_dependency_envelope = error_envelope(&direct_missing_webhook, "DependencyError");
    assert!(direct_dependency_envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("WECOM_WEBHOOK_URL"));

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd]);
    assert_success(&manifest, "wecom manifest");

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd]);
    let inspect_stdout = assert_success(&inspect, "wecom inspect");
    assert!(inspect_stdout.contains("MCP tool: wecom_team_notice"));
    assert!(inspect_stdout.contains("WECOM_WEBHOOK_URL"));

    let check = run_skillrun(&["check", "--cwd", &cwd]);
    assert_success(&check, "wecom check");

    let test = run_skillrun(&["test", "--cwd", &cwd]);
    let test_stdout = assert_success(&test, "wecom test");
    let test_envelope: Value = serde_json::from_str(&test_stdout).expect("test JSON");
    assert_eq!(test_envelope["ok"], true);
    assert_eq!(test_envelope["output"]["decision"], "preview");
    assert_eq!(test_envelope["artifacts"][0]["kind"], "markdown");

    let dry_run = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        "examples/dry_run.input.json",
    ]);
    let dry_run_stdout = assert_success(&dry_run, "wecom dry-run");
    let dry_run_envelope: Value = serde_json::from_str(&dry_run_stdout).expect("dry-run JSON");
    assert_eq!(dry_run_envelope["output"]["decision"], "preview");

    let approval = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        "examples/urgent_requires_approval.input.json",
    ]);
    let approval_envelope = error_envelope(&approval, "PolicyViolation");
    assert_eq!(approval_envelope["error"]["recoverable"], true);

    let secret = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        "examples/invalid_secret.input.json",
    ]);
    let secret_envelope = error_envelope(&secret, "PolicyViolation");
    assert!(secret_envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("secret marker"));

    let missing_webhook =
        run_skillrun(&["run", "--cwd", &cwd, "--input", "examples/send.input.json"]);
    let dependency_envelope = error_envelope(&missing_webhook, "DependencyError");
    assert!(dependency_envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("WECOM_WEBHOOK_URL"));

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd, "--dry-run"]);
    let serve_stdout = assert_success(&serve, "wecom serve dry-run");
    let contract: Value = serde_json::from_str(&serve_stdout).expect("MCP JSON");
    assert_eq!(contract["tools"][0]["name"], "wecom_team_notice");
    assert!(contract["resources"][0]["text"]
        .as_str()
        .unwrap()
        .contains("WeCom Team Notice"));

    let pack = run_skillrun(&["pack", "--cwd", &cwd]);
    assert_success(&pack, "wecom pack");
    let archive_path = capsule.join("dist").join("wecom_team_notice-0.4.2.skr");
    assert!(archive_path.is_file());
    let unpacked = output_root.join("unpacked-wecom");
    fs::create_dir_all(&unpacked).expect("unpack dir should be created");
    unpack_archive(&archive_path, &unpacked);
    let unpacked_inspect = run_skillrun(&["inspect", "--cwd", &unpacked.to_string_lossy()]);
    assert_success(&unpacked_inspect, "unpacked wecom inspect");
    let unpacked_check = run_skillrun(&["check", "--cwd", &unpacked.to_string_lossy()]);
    assert_success(&unpacked_check, "unpacked wecom check");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn v042_official_reference_capsules_run_without_registry_or_sandbox_claims() {
    let output_root = temp_dir("business-v042-capsules");

    for capsule_name in [
        "commit_message_gate",
        "bounded_file_patcher",
        "readonly_diagnostics_runner",
    ] {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(capsule_name);
        let capsule = output_root.join(capsule_name);
        copy_dir(&source, &capsule);
        let cwd = capsule.to_string_lossy().to_string();

        let manifest = run_skillrun(&["manifest", "--cwd", &cwd]);
        assert_success(&manifest, &format!("{capsule_name} manifest"));

        let inspect = run_skillrun(&["inspect", "--cwd", &cwd]);
        let inspect_stdout = assert_success(&inspect, &format!("{capsule_name} inspect"));
        assert!(
            inspect_stdout.contains(&format!("MCP tool: {capsule_name}")),
            "inspect should expose the capsule MCP tool"
        );

        let check = run_skillrun(&["check", "--cwd", &cwd]);
        assert_success(&check, &format!("{capsule_name} check"));

        let test = run_skillrun(&["test", "--cwd", &cwd]);
        let test_stdout = assert_success(&test, &format!("{capsule_name} test"));
        let test_envelope: Value =
            serde_json::from_str(&test_stdout).expect("reference capsule test JSON");
        assert_eq!(test_envelope["ok"], true);
        assert_eq!(test_envelope["artifacts"][0]["kind"], "markdown");

        let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd, "--dry-run"]);
        let serve_stdout = assert_success(&serve, &format!("{capsule_name} serve dry-run"));
        let contract: Value = serde_json::from_str(&serve_stdout).expect("MCP JSON");
        assert_eq!(contract["tools"][0]["name"], capsule_name);

        let pack = run_skillrun(&["pack", "--cwd", &cwd]);
        assert_success(&pack, &format!("{capsule_name} pack"));
        assert!(capsule
            .join("dist")
            .join(format!("{capsule_name}-0.4.2.skr"))
            .is_file());
    }

    let commit_gate = output_root
        .join("commit_message_gate")
        .to_string_lossy()
        .to_string();
    let commit_violation = run_skillrun(&[
        "run",
        "--cwd",
        &commit_gate,
        "--input",
        "examples/invalid.input.json",
    ]);
    let commit_envelope = error_envelope(&commit_violation, "PolicyViolation");
    assert!(commit_envelope["error"]["llm_hint"]
        .as_str()
        .unwrap()
        .contains("policy"));

    let patcher = output_root
        .join("bounded_file_patcher")
        .to_string_lossy()
        .to_string();
    let blocked_path = run_skillrun(&[
        "run",
        "--cwd",
        &patcher,
        "--input",
        "examples/blocked_path.input.json",
    ]);
    let path_envelope = error_envelope(&blocked_path, "PolicyViolation");
    assert!(path_envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("file_path"));

    let diagnostics = output_root
        .join("readonly_diagnostics_runner")
        .to_string_lossy()
        .to_string();
    let list = run_skillrun(&[
        "run",
        "--cwd",
        &diagnostics,
        "--input",
        "examples/list.input.json",
    ]);
    let list_stdout = assert_success(&list, "readonly diagnostics list");
    let list_envelope: Value = serde_json::from_str(&list_stdout).expect("list JSON");
    assert_eq!(list_envelope["output"]["diagnostic"], "list");
    assert!(list_envelope["output"]["stdout"]
        .as_str()
        .unwrap()
        .contains("SKILL.md"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn command_adapter_example_capsule_proves_level_zero_ipc() {
    let output_root = temp_dir("business-command-adapter");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/command_hello");
    let capsule = output_root.join("command_hello");
    copy_dir(&source, &capsule);
    let cwd = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd]);
    assert_success(&manifest, "command adapter manifest");
    let manifest_yaml =
        fs::read_to_string(capsule.join(".skillrun/manifest.generated.yaml")).unwrap();
    assert!(manifest_yaml.contains("adapter: command"));
    assert!(manifest_yaml.contains("protocol_version: adapter.v1"));
    assert!(manifest_yaml.contains("- python"));
    assert!(manifest_yaml.contains("- action.py"));

    let check = run_skillrun(&["check", "--cwd", &cwd]);
    assert_success(&check, "command adapter check");

    let test = run_skillrun(&["test", "--cwd", &cwd]);
    let test_stdout = assert_success(&test, "command adapter test");
    assert!(!test_stdout.contains("command adapter log"));
    let test_envelope: Value = serde_json::from_str(&test_stdout).expect("test JSON");
    assert_eq!(test_envelope["ok"], true);
    assert_eq!(test_envelope["output"]["adapter"], "command");
    assert_eq!(
        test_envelope["output"]["message"],
        "hello Ada from command adapter"
    );
    assert_eq!(test_envelope["artifacts"][0]["kind"], "markdown");
    let run_dir = PathBuf::from(test_envelope["run_dir"].as_str().unwrap());
    let stdout_log = fs::read_to_string(run_dir.join("stdout.log")).expect("stdout log readable");
    assert!(stdout_log.contains("command adapter log"));

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd, "--dry-run"]);
    let serve_stdout = assert_success(&serve, "command adapter serve dry-run");
    let contract: Value = serde_json::from_str(&serve_stdout).expect("MCP JSON");
    assert_eq!(contract["tools"][0]["name"], "command_hello");

    let pack = run_skillrun(&["pack", "--cwd", &cwd]);
    assert_success(&pack, "command adapter pack");
    assert!(capsule
        .join("dist")
        .join("command_hello-0.4.2.skr")
        .is_file());

    fs::remove_dir_all(output_root).ok();
}
