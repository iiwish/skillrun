use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn init_capsule_with_flag(label: &str, flag: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();

    let init = run_skillrun(&["init", "refund", flag, "--output", &output_arg]);
    assert!(
        init.status.success(),
        "init should succeed\nstderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let capsule = output_root.join("refund");
    (output_root, capsule)
}

fn init_capsule(label: &str) -> (PathBuf, PathBuf) {
    init_capsule_with_flag(label, "--python")
}

fn write_manifest(capsule: &Path) {
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );
}

fn generated_capsule(label: &str) -> (PathBuf, PathBuf) {
    let (output_root, capsule) = init_capsule(label);
    write_manifest(&capsule);
    (output_root, capsule)
}

fn generated_capsule_with_flag(label: &str, flag: &str) -> (PathBuf, PathBuf) {
    let (output_root, capsule) = init_capsule_with_flag(label, flag);
    write_manifest(&capsule);
    (output_root, capsule)
}

fn assert_success_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    serde_json::from_str(&stdout).expect("stdout should be JSON")
}

fn is_64_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[test]
fn mcp_dry_run_maps_manifest_tool_schema_and_skill_resource() {
    let (output_root, capsule) = generated_capsule("mcp-dry-run");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    let contract = assert_success_json(&serve);

    assert_eq!(contract["mcp"]["dry_run"], true);
    assert_eq!(contract["mcp"]["transport"], "stdio");
    assert_eq!(contract["tools"][0]["name"], "refund");
    assert!(contract["tools"][0]["description"]
        .as_str()
        .unwrap()
        .contains("refund"));
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["order_id"]["type"],
        "string"
    );
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["amount"]["type"],
        "integer"
    );
    assert_eq!(
        contract["tools"][0]["result_contract"],
        "SkillRun output/error envelope"
    );
    assert_eq!(contract["resources"][0]["path"], "SKILL.md");
    assert_eq!(contract["resources"][0]["mime_type"], "text/markdown");
    let resource_text = contract["resources"][0]["text"]
        .as_str()
        .expect("resource text should be a string");
    assert!(resource_text.to_ascii_lowercase().contains("refund"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_dry_run_maps_js_manifest_tool_schema_and_skill_resource() {
    let (output_root, capsule) = generated_capsule_with_flag("mcp-js-dry-run", "--js");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    let contract = assert_success_json(&serve);

    assert_eq!(contract["mcp"]["dry_run"], true);
    assert_eq!(contract["mcp"]["transport"], "stdio");
    assert_eq!(contract["tools"][0]["name"], "refund");
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["order_id"]["type"],
        "string"
    );
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["amount"]["type"],
        "integer"
    );
    assert_eq!(
        contract["tools"][0]["input_schema"]["properties"]["reason"]["enum"][2],
        "wrong_item"
    );
    assert_eq!(
        contract["tools"][0]["output_schema"]["properties"]["decision"]["enum"][1],
        "needs_approval"
    );
    assert_eq!(
        contract["tools"][0]["result_contract"],
        "SkillRun output/error envelope"
    );
    assert_eq!(contract["resources"][0]["path"], "SKILL.md");
    assert_eq!(contract["resources"][0]["mime_type"], "text/markdown");
    let resource_text = contract["resources"][0]["text"]
        .as_str()
        .expect("resource text should be a string");
    assert!(resource_text.to_ascii_lowercase().contains("refund"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_stdio_initializes_with_2025_11_25_protocol() {
    let (output_root, capsule) = generated_capsule("mcp-stdio-init");
    let mut client = ScriptedMcpClient::spawn(&capsule);

    let response = client.initialize();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
    assert!(response["result"]["serverInfo"]["name"]
        .as_str()
        .unwrap_or_default()
        .contains("skillrun"));
    assert!(response["result"]["capabilities"]["tools"].is_object());
    assert!(response["result"]["capabilities"]["resources"].is_object());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_stdio_returns_jsonrpc_error_for_unrecognized_method() {
    let (output_root, capsule) = generated_capsule("mcp-stdio-unrecognized");
    let mut client = ScriptedMcpClient::spawn(&capsule);
    client.initialize();
    client.initialized();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 99,
        "method": "skillrun/not-a-method",
        "params": {}
    }));
    let response = client.read_response("unrecognized method response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 99);
    assert_eq!(response["error"]["code"], -32601);
    assert!(response["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("Method not found"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_stdio_lists_and_calls_manifest_tool() {
    let (output_root, capsule) = generated_capsule("mcp-stdio-tools");
    let mut client = ScriptedMcpClient::spawn(&capsule);
    client.initialize();
    client.initialized();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }));
    let tools = client.read_response("tools/list response");
    assert_eq!(tools["id"], 2);
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
    let call = client.read_response("tools/call response");
    assert_eq!(call["id"], 3);
    assert_eq!(call["result"]["isError"], false);
    assert_eq!(call["result"]["content"][0]["type"], "text");
    assert!(call["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .contains("approved"));
    let run_record = fs::read_dir(capsule.join(".skillrun").join("runs"))
        .expect("runs directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("record.json"))
        .find(|path| path.is_file())
        .expect("tools/call should create a SkillRun run record");
    let record: Value = serde_json::from_str(
        &fs::read_to_string(run_record).expect("run record should be readable"),
    )
    .expect("run record should parse");
    assert_eq!(record["mode"], "mcp");

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
    let error_call = client.read_response("tools/call error response");
    assert_eq!(error_call["id"], 4);
    assert_eq!(error_call["result"]["isError"], true);
    assert!(error_call["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .contains("PolicyViolation"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_stdio_lists_and_calls_js_manifest_tool() {
    let (output_root, capsule) = generated_capsule_with_flag("mcp-js-stdio-tools", "--js");
    let mut client = ScriptedMcpClient::spawn(&capsule);
    client.initialize();
    client.initialized();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }));
    let tools = client.read_response("JS tools/list response");
    assert_eq!(tools["id"], 2);
    assert_eq!(tools["result"]["tools"][0]["name"], "refund");
    assert_eq!(
        tools["result"]["tools"][0]["inputSchema"]["properties"]["amount"]["type"],
        "integer"
    );
    assert_eq!(
        tools["result"]["tools"][0]["outputSchema"]["properties"]["decision"]["enum"][1],
        "needs_approval"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "refund",
            "arguments": {
                "order_id": "order_js_1001",
                "amount": 120,
                "reason": "damaged",
                "customer_tier": "standard"
            }
        }
    }));
    let call = client.read_response("JS tools/call response");
    assert_eq!(call["id"], 3);
    assert_eq!(call["result"]["isError"], false);
    assert_eq!(call["result"]["content"][0]["type"], "text");
    assert!(call["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .contains("approved"));
    let run_record = fs::read_dir(capsule.join(".skillrun").join("runs"))
        .expect("runs directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("record.json"))
        .find(|path| path.is_file())
        .expect("tools/call should create a SkillRun run record");
    let record: Value = serde_json::from_str(
        &fs::read_to_string(run_record).expect("run record should be readable"),
    )
    .expect("run record should parse");
    assert_eq!(record["mode"], "mcp");
    assert_eq!(record["status"], "succeeded");
    assert!(is_64_hex(record["action_sha256"].as_str().unwrap()));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "refund",
            "arguments": {
                "order_id": "order_js_1002",
                "amount": 1200,
                "reason": "damaged",
                "customer_tier": "standard"
            }
        }
    }));
    let error_call = client.read_response("JS tools/call error response");
    assert_eq!(error_call["id"], 4);
    assert_eq!(error_call["result"]["isError"], true);
    let error_text = error_call["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(error_text.contains("PolicyViolation"));
    assert!(error_text.contains("manager approval"));

    drop(client);
    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_stdio_dependency_error_keeps_server_alive() {
    let (output_root, capsule) =
        generated_capsule_with_flag("mcp-js-dependency-error-survival", "--js");
    let empty_path = output_root.join("empty-path");
    fs::create_dir_all(&empty_path).expect("empty PATH dir should be created");
    let mut client = ScriptedMcpClient::spawn_with_path(&capsule, &empty_path);
    client.initialize();
    client.initialized();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "refund",
            "arguments": {
                "order_id": "order_js_missing_node",
                "amount": 120,
                "reason": "damaged",
                "customer_tier": "standard"
            }
        }
    }));
    let call = client.read_response("DependencyError tools/call response");
    assert_eq!(call["id"], 10);
    assert_eq!(call["result"]["isError"], true);
    let error_text = call["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(error_text.contains("DependencyError"));
    assert!(error_text.contains("node"));
    assert!(error_text.contains("missing"));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "tools/list",
        "params": {}
    }));
    let tools = client.read_response("tools/list after DependencyError response");
    assert_eq!(tools["id"], 11);
    assert_eq!(tools["result"]["tools"][0]["name"], "refund");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_stdio_lists_and_reads_manifest_resources() {
    let (output_root, capsule) = generated_capsule("mcp-stdio-resources");
    let mut client = ScriptedMcpClient::spawn(&capsule);
    client.initialize();
    client.initialized();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "resources/list",
        "params": {}
    }));
    let resources = client.read_response("resources/list response");
    assert_eq!(resources["id"], 5);
    let listed = resources["result"]["resources"]
        .as_array()
        .expect("resources should be an array");
    let listed_uris = listed
        .iter()
        .map(|resource| {
            resource["uri"]
                .as_str()
                .expect("resource URI should be a string")
                .to_string()
        })
        .collect::<Vec<_>>();
    assert!(listed_uris
        .iter()
        .all(|uri| uri.starts_with("skillrun://refund/")));
    assert!(listed_uris.iter().all(|uri| !uri.contains("action.py")));
    assert!(listed_uris.iter().all(|uri| !uri.contains(".skillrun")));
    let skill_uri = listed_uris
        .iter()
        .find(|uri| uri.ends_with("SKILL.md"))
        .expect("SKILL.md resource should be listed")
        .to_string();
    let example_uri = listed_uris
        .iter()
        .find(|uri| uri.ends_with("examples/default.input.json"))
        .expect("default example resource should be listed")
        .to_string();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "resources/read",
        "params": {
            "uri": skill_uri
        }
    }));
    let read = client.read_response("resources/read response");
    assert_eq!(read["id"], 6);
    assert_eq!(read["result"]["contents"][0]["mimeType"], "text/markdown");
    assert!(read["result"]["contents"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains("refund"));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "resources/read",
        "params": {
            "uri": example_uri
        }
    }));
    let example_read = client.read_response("example resource read response");
    assert_eq!(example_read["id"], 8);
    assert_eq!(
        example_read["result"]["contents"][0]["mimeType"],
        "application/json"
    );
    assert!(example_read["result"]["contents"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .contains("order_1001"));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "resources/read",
        "params": {
            "uri": "skillrun://refund/../action.py"
        }
    }));
    let rejected = client.read_response("resources/read traversal response");
    assert_eq!(rejected["id"], 7);
    assert!(rejected.get("error").is_some());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_dry_run_does_not_import_action_for_metadata() {
    let (output_root, capsule) = init_capsule("mcp-no-import");
    let marker = output_root.join("import-marker.txt");
    let marker_literal = serde_json::to_string(&marker.to_string_lossy().to_string())
        .expect("marker should serialize");
    let action_path = capsule.join("action.py");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = action.replace(
        "from typing import Literal\n",
        &format!(
            "from typing import Literal\nfrom pathlib import Path\nPath({marker_literal}).write_text(\"imported\", encoding=\"utf-8\")\n"
        ),
    );
    fs::write(&action_path, action).expect("action should be updated");

    write_manifest(&capsule);
    assert!(
        marker.is_file(),
        "manifest metadata extraction imports action.py in Author Mode"
    );
    fs::remove_file(&marker).expect("marker should be removed before serve");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert_success_json(&serve);
    assert!(
        !marker.exists(),
        "serve --mcp --dry-run must not import action.py for metadata"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_dry_run_does_not_import_js_action_for_metadata() {
    let (output_root, capsule) = init_capsule_with_flag("mcp-js-no-import", "--js");
    let marker = output_root.join("js-import-marker.txt");
    let marker_literal = serde_json::to_string(&marker.to_string_lossy().to_string())
        .expect("marker should serialize");
    let action_path = capsule.join("action.mjs");
    let action = fs::read_to_string(&action_path).expect("action should be readable");
    let action = format!(
        "import fs from \"node:fs\";\nfs.writeFileSync({marker_literal}, \"imported\", \"utf8\");\n{action}"
    );
    fs::write(&action_path, action).expect("action should be updated");

    write_manifest(&capsule);
    assert!(
        marker.is_file(),
        "manifest metadata extraction imports action.mjs in Author Mode"
    );
    fs::remove_file(&marker).expect("marker should be removed before serve");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert_success_json(&serve);
    assert!(
        !marker.exists(),
        "serve --mcp --dry-run must not import action.mjs for metadata"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn mcp_dry_run_fails_closed_when_manifest_is_stale() {
    let (output_root, capsule) = generated_capsule("mcp-stale");
    let action_path = capsule.join("action.py");
    let mut action = fs::read_to_string(&action_path).expect("action should be readable");
    action.push_str("\n# stale after manifest\n");
    fs::write(&action_path, action).expect("action should be updated");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let serve = run_skillrun(&["serve", "--mcp", "--cwd", &cwd_arg, "--dry-run"]);
    assert!(!serve.status.success());
    let stderr = String::from_utf8(serve.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("stale Manifest"));
    assert!(stderr.contains("action.py"));
    assert!(!stderr.contains("command not implemented yet"));

    fs::remove_dir_all(output_root).ok();
}
