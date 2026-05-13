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
    let archive_path = capsule.join("dist").join("refund-0.3.0.skr");
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
    assert!(readme.contains("manifest-driven Agent skill capsule"));
    assert!(readme.contains("FastMCP turns functions into MCP tools"));
    assert!(!readme.contains("tested MCP skill package"));
    assert!(!cargo_toml.contains("tested MCP skill package"));
}
