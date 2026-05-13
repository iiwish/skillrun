use flate2::read::GzDecoder;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Archive;

#[path = "fixtures/mcp_stdio.rs"]
mod mcp_stdio;

use mcp_stdio::ScriptedMcpClient;

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

fn assert_failure(output: &std::process::Output, label: &str) -> String {
    assert!(
        !output.status.success(),
        "{label} should fail\nstdout: {}\nstderr: {}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
    output_text(&output.stderr)
}

fn success_envelope(output: &std::process::Output, label: &str) -> Value {
    let stdout = assert_success(output, label);
    let envelope: Value = serde_json::from_str(&stdout).expect("stdout should be JSON envelope");
    assert_eq!(envelope["ok"], true);
    assert!(envelope["run_id"].as_str().unwrap().starts_with("run-"));
    envelope
}

fn error_envelope(output: &std::process::Output, code: &str) -> Value {
    assert!(
        !output.status.success(),
        "command should fail with {code}\nstdout: {}\nstderr: {}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
    let stdout = output_text(&output.stdout);
    let envelope: Value = serde_json::from_str(&stdout).expect("stdout should be JSON envelope");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], code);
    envelope
}

fn init_capsule(output_root: &Path, name: &str) -> PathBuf {
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", name, "--python", "--output", &output_arg]);
    assert_success(&init, "init");
    output_root.join(name)
}

fn init_capsule_with_flag(output_root: &Path, name: &str, flag: &str) -> PathBuf {
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", name, flag, "--output", &output_arg]);
    assert_success(&init, "init");
    output_root.join(name)
}

fn manifest(capsule: &Path) {
    let cwd = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd]);
    assert_success(&manifest, "manifest");
}

fn append_to(path: &Path, text: &str) {
    let mut current = fs::read_to_string(path).expect("file should be readable");
    current.push_str(text);
    fs::write(path, current).expect("file should be writable");
}

fn unpack_archive(path: &Path, target: &Path) {
    let file = fs::File::open(path).expect("archive should be readable");
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(target).expect("archive should unpack");
}

fn replace_in_file(path: &Path, from: &str, to: &str) {
    let current = fs::read_to_string(path).expect("file should be readable");
    let updated = current.replace(from, to);
    assert_ne!(
        current,
        updated,
        "replacement should change {}",
        path.display()
    );
    fs::write(path, updated).expect("file should be writable");
}

#[test]
fn a001_to_a013_release_matrix_has_fresh_command_evidence() {
    let output_root = temp_dir("e2e-matrix");
    let capsule = init_capsule(&output_root, "refund");
    let cwd = capsule.to_string_lossy().to_string();

    for required in [
        "SKILL.md",
        "action.py",
        "examples/default.input.json",
        "skillrun.config.json",
    ] {
        assert!(capsule.join(required).is_file(), "A001 missing {required}");
    }

    let duplicate = run_skillrun(&[
        "init",
        "refund",
        "--python",
        "--output",
        &output_root.to_string_lossy(),
    ]);
    assert_failure(&duplicate, "duplicate init");

    manifest(&capsule);
    let manifest_path = capsule.join(".skillrun").join("manifest.generated.yaml");
    let manifest_text = fs::read_to_string(&manifest_path).expect("manifest should be readable");
    for expected in [
        "sources:",
        "schemas:",
        "permissions:",
        "adapter: python",
        "description:",
    ] {
        assert!(manifest_text.contains(expected), "A002 missing {expected}");
    }

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd]);
    let inspect_stdout = assert_success(&inspect, "inspect");
    for expected in [
        "status: runnable",
        "SOP hash",
        "input schema: present",
        "MCP tool",
    ] {
        assert!(inspect_stdout.contains(expected), "A003 missing {expected}");
    }

    let test = run_skillrun(&["test", "--cwd", &cwd]);
    let test_envelope = success_envelope(&test, "test");
    let test_run_dir = PathBuf::from(test_envelope["run_dir"].as_str().unwrap());
    assert!(test_run_dir.join("record.json").is_file(), "A004 record");
    assert!(test_run_dir.join("stdout.log").is_file(), "A004 stdout log");
    assert!(test_run_dir.join("stderr.log").is_file(), "A004 stderr log");

    let run = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        "examples/default.input.json",
    ]);
    let run_envelope = success_envelope(&run, "run");
    assert!(run_envelope["display"]["markdown"].as_str().unwrap().len() > 3);
    let record_path = PathBuf::from(run_envelope["record"].as_str().unwrap());
    let record: Value =
        serde_json::from_str(&fs::read_to_string(record_path).expect("record should be readable"))
            .expect("record should parse");
    assert!(record["manifest_sha256"].as_str().unwrap().len() > 10);

    let invalid_input = capsule.join("examples").join("invalid.input.json");
    fs::write(
        &invalid_input,
        r#"{"order_id":"","amount":0,"reason":"unknown","customer_tier":"standard"}"#,
    )
    .expect("invalid input should be written");
    let invalid = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        &invalid_input.to_string_lossy(),
    ]);
    let invalid_envelope = error_envelope(&invalid, "ValidationError");
    assert_eq!(invalid_envelope["error"]["recoverable"], true);

    let policy_input = capsule.join("examples").join("policy.input.json");
    fs::write(
        &policy_input,
        r#"{"order_id":"order_9001","amount":900,"reason":"damaged","customer_tier":"standard"}"#,
    )
    .expect("policy input should be written");
    let policy = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        &policy_input.to_string_lossy(),
    ]);
    let policy_envelope = error_envelope(&policy, "PolicyViolation");
    assert_eq!(policy_envelope["error"]["recoverable"], true);
    assert!(policy_envelope["error"]["llm_hint"]
        .as_str()
        .unwrap()
        .contains("approval"));

    let protocol_root = output_root.join("protocol");
    let protocol_capsule = init_capsule(&protocol_root, "refund");
    replace_in_file(
        &protocol_capsule.join("action.py"),
        "from typing import Literal\n",
        "from typing import Literal\nimport os\n",
    );
    replace_in_file(
        &protocol_capsule.join("action.py"),
        "def run(input: Input, ctx) -> Output:\n",
        "def run(input: Input, ctx) -> Output:\n    print(\"FAKE_OK_FROM_STDOUT\", flush=True)\n    os._exit(0)\n",
    );
    manifest(&protocol_capsule);
    let protocol = run_skillrun(&["test", "--cwd", &protocol_capsule.to_string_lossy()]);
    let protocol_envelope = error_envelope(&protocol, "ProtocolViolation");
    let protocol_run_dir = PathBuf::from(protocol_envelope["run_dir"].as_str().unwrap());
    let stdout_log =
        fs::read_to_string(protocol_run_dir.join("stdout.log")).expect("stdout log readable");
    assert!(stdout_log.contains("FAKE_OK_FROM_STDOUT"));

    let artifact_root = output_root.join("artifact");
    let artifact_capsule = init_capsule(&artifact_root, "refund");
    replace_in_file(
        &artifact_capsule.join("action.py"),
        "def run(input: Input, ctx) -> Output:\n",
        "def run(input: Input, ctx) -> Output:\n    return {\"output\": Output(decision=\"approved\", amount=input.amount, reasoning_summary=\"artifact escapes\", audit_note=\"blocked\"), \"artifacts\": [{\"name\":\"bad\", \"kind\":\"text\", \"path\":\"../outside.txt\"}]}\n",
    );
    manifest(&artifact_capsule);
    let artifact = run_skillrun(&["test", "--cwd", &artifact_capsule.to_string_lossy()]);
    error_envelope(&artifact, "PermissionDenied");

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd, "--dry-run"]);
    let serve_stdout = assert_success(&serve, "serve dry-run");
    let contract: Value = serde_json::from_str(&serve_stdout).expect("MCP dry-run should be JSON");
    assert_eq!(contract["tools"][0]["name"], "refund");
    assert_eq!(contract["resources"][0]["path"], "SKILL.md");

    let pack = run_skillrun(&["pack", "--cwd", &cwd]);
    assert_success(&pack, "pack");
    let archive_path = capsule.join("dist").join("refund-0.3.0.skr");
    assert!(archive_path.is_file(), "A012 archive should exist");
    let unpacked = output_root.join("unpacked");
    fs::create_dir_all(&unpacked).expect("unpack target should exist");
    unpack_archive(&archive_path, &unpacked);
    let unpacked_inspect = run_skillrun(&["inspect", "--cwd", &unpacked.to_string_lossy()]);
    assert_success(&unpacked_inspect, "unpacked inspect");

    let mut action = fs::read_to_string(capsule.join("action.py")).expect("action readable");
    action.push_str("\n# stale after release matrix pack\n");
    fs::write(capsule.join("action.py"), action).expect("action writable");
    let stale = run_skillrun(&["serve", "--mcp", "--cwd", &cwd, "--dry-run"]);
    let stale_stderr = assert_failure(&stale, "stale serve");
    assert!(stale_stderr.contains("stale Manifest"));

    let instruction_only = output_root.join("instruction-only");
    fs::create_dir_all(instruction_only.join("scripts")).expect("instruction dir created");
    fs::write(
        instruction_only.join("SKILL.md"),
        "# Instruction Only\n\nThis is documentation, not a SkillRun capsule.\n",
    )
    .expect("instruction skill written");
    fs::write(
        instruction_only.join("scripts").join("helper.py"),
        "print('no implicit action')\n",
    )
    .expect("script written");
    let instruction_cwd = instruction_only.to_string_lossy().to_string();
    let instruction_inspect = run_skillrun(&["inspect", "--cwd", &instruction_cwd]);
    let instruction_stdout = assert_success(&instruction_inspect, "instruction inspect");
    assert!(instruction_stdout.contains("status: instruction-only"));
    for command in [
        vec!["manifest", "--cwd", &instruction_cwd],
        vec![
            "run",
            "--cwd",
            &instruction_cwd,
            "--input",
            "examples/default.input.json",
        ],
        vec!["serve", "--mcp", "--cwd", &instruction_cwd, "--dry-run"],
        vec!["pack", "--cwd", &instruction_cwd],
    ] {
        let output = run_skillrun(&command);
        assert!(
            !output.status.success(),
            "instruction-only command should fail: {command:?}"
        );
    }

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn a014_mcp_stdio_release_matrix_exercises_full_client_flow() {
    let output_root = temp_dir("e2e-mcp-stdio");
    let capsule = init_capsule(&output_root, "refund");
    replace_in_file(
        &capsule.join("action.py"),
        "def run(input: Input, ctx) -> Output:\n",
        "def run(input: Input, ctx) -> Output:\n    print(\"MCP_STDOUT_NOISE\", flush=True)\n",
    );
    manifest(&capsule);

    let mut client = ScriptedMcpClient::spawn(&capsule);
    let initialize = client.initialize();
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    assert!(initialize["result"]["capabilities"]["tools"].is_object());
    assert!(initialize["result"]["capabilities"]["resources"].is_object());
    client.initialized();
    client.expect_no_stdout_line("initialized notification");

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }));
    let tools = client.read_response("release tools/list response");
    assert_eq!(tools["result"]["tools"][0]["name"], "refund");
    assert_eq!(
        tools["result"]["tools"][0]["inputSchema"]["properties"]["order_id"]["type"],
        "string"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "refund",
            "arguments": {
                "order_id": "order_1001",
                "amount": 120,
                "reason": "damaged",
                "customer_tier": "standard"
            }
        }
    }));
    let call = client.read_response("release tools/call success response");
    assert_eq!(call["result"]["isError"], false);
    let call_text = call["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result text should be present");
    assert!(call_text.contains("approved"));
    assert!(!call_text.contains("MCP_STDOUT_NOISE"));
    let mcp_run_dir = fs::read_dir(capsule.join(".skillrun").join("runs"))
        .expect("runs directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.join("record.json").is_file())
        .expect("MCP tools/call should create a run directory");
    let stdout_log =
        fs::read_to_string(mcp_run_dir.join("stdout.log")).expect("stdout log should be readable");
    assert!(stdout_log.contains("MCP_STDOUT_NOISE"));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "refund",
            "arguments": {
                "order_id": "order_1002",
                "amount": 1200,
                "reason": "damaged",
                "customer_tier": "standard"
            }
        }
    }));
    let error_call = client.read_response("release tools/call error response");
    assert_eq!(error_call["result"]["isError"], true);
    assert!(error_call["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .contains("PolicyViolation"));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "resources/list",
        "params": {}
    }));
    let resources = client.read_response("release resources/list response");
    let listed = resources["result"]["resources"]
        .as_array()
        .expect("resources should be listed");
    let listed_uris = listed
        .iter()
        .map(|resource| resource["uri"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(listed_uris.iter().all(|uri| !uri.contains("action.py")));
    assert!(listed_uris.iter().all(|uri| !uri.contains(".skillrun")));
    let skill_uri = listed_uris
        .iter()
        .find(|uri| uri.ends_with("SKILL.md"))
        .expect("SKILL.md should be an MCP resource")
        .to_string();
    let example_uri = listed_uris
        .iter()
        .find(|uri| uri.ends_with("examples/default.input.json"))
        .expect("default example should be an MCP resource")
        .to_string();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "resources/read",
        "params": {
            "uri": skill_uri
        }
    }));
    let skill_read = client.read_response("release resources/read skill response");
    assert_eq!(
        skill_read["result"]["contents"][0]["mimeType"],
        "text/markdown"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "resources/read",
        "params": {
            "uri": example_uri
        }
    }));
    let example_read = client.read_response("release resources/read example response");
    assert_eq!(
        example_read["result"]["contents"][0]["mimeType"],
        "application/json"
    );
    assert!(example_read["result"]["contents"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .contains("order_1001"));
    client.expect_no_stdout_line("completed MCP release flow");

    drop(client);
    fs::remove_dir_all(output_root).ok();
}

#[test]
fn js_alpha_local_command_matrix_covers_init_manifest_inspect_test_and_run() {
    let output_root = temp_dir("e2e-js-alpha");
    let capsule = init_capsule_with_flag(&output_root, "refund", "--js");
    let cwd = capsule.to_string_lossy().to_string();

    for required in [
        "SKILL.md",
        "action.mjs",
        "examples/default.input.json",
        "skillrun.config.json",
    ] {
        assert!(
            capsule.join(required).is_file(),
            "JS init missing {required}"
        );
    }
    assert!(!capsule.join("action.py").exists());
    assert!(!capsule.join("package.json").exists());

    manifest(&capsule);
    let manifest_path = capsule.join(".skillrun").join("manifest.generated.yaml");
    let manifest_text = fs::read_to_string(&manifest_path).expect("manifest should be readable");
    for expected in [
        "adapter: node",
        "entrypoint: action.mjs",
        "path: action.mjs",
    ] {
        assert!(
            manifest_text.contains(expected),
            "JS manifest missing {expected}"
        );
    }

    let inspect = run_skillrun(&["inspect", "--cwd", &cwd]);
    let inspect_stdout = assert_success(&inspect, "JS inspect");
    for expected in [
        "status: runnable",
        "runtime contract: Manifest adapter and entrypoint",
        "adapter: node",
        "entrypoint: action.mjs",
        "preflight: present",
    ] {
        assert!(
            inspect_stdout.contains(expected),
            "JS inspect missing {expected}"
        );
    }

    let test = run_skillrun(&["test", "--cwd", &cwd]);
    let test_envelope = success_envelope(&test, "JS test");
    assert_eq!(test_envelope["output"]["decision"], "approved");

    let run = run_skillrun(&[
        "run",
        "--cwd",
        &cwd,
        "--input",
        "examples/default.input.json",
    ]);
    let run_envelope = success_envelope(&run, "JS run");
    assert_eq!(run_envelope["output"]["decision"], "approved");

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd, "--dry-run"]);
    let serve_stdout = assert_success(&serve, "JS serve dry-run");
    let contract: Value =
        serde_json::from_str(&serve_stdout).expect("JS MCP dry-run should be JSON");
    assert_eq!(contract["tools"][0]["name"], "refund");
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["amount"]["type"],
        "integer"
    );
    assert_eq!(
        contract["tools"][0]["output_schema"]["properties"]["decision"]["enum"][1],
        "needs_approval"
    );
    assert_eq!(contract["resources"][0]["path"], "SKILL.md");

    let pack = run_skillrun(&["pack", "--cwd", &cwd]);
    let pack_stdout = assert_success(&pack, "JS pack");
    assert!(pack_stdout.contains("does not vendor dependencies"));
    assert!(
        capsule.join("dist").join("refund-0.3.0.skr").is_file(),
        "JS pack should create .skr archive"
    );

    append_to(
        &capsule.join("action.mjs"),
        "\n// stale after JS command matrix\n",
    );
    let stale = run_skillrun(&["test", "--cwd", &cwd]);
    let stale_stderr = assert_failure(&stale, "stale JS test");
    assert!(stale_stderr.contains("stale Manifest"));
    assert!(stale_stderr.contains("action.mjs"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn py_alias_manifest_smoke_uses_python_adapter_identity() {
    let output_root = temp_dir("e2e-py-alias");
    let capsule = init_capsule_with_flag(&output_root, "refund", "--py");
    assert!(capsule.join("action.py").is_file());
    assert!(!capsule.join("action.mjs").exists());

    manifest(&capsule);
    let manifest_text =
        fs::read_to_string(capsule.join(".skillrun").join("manifest.generated.yaml"))
            .expect("manifest should be readable");
    assert!(manifest_text.contains("adapter: python"));
    assert!(manifest_text.contains("entrypoint: action.py"));

    fs::remove_dir_all(output_root).ok();
}
