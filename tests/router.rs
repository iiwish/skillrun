use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "fixtures/mcp_stdio.rs"]
mod mcp_stdio;

use mcp_stdio::ScriptedMcpClient;

fn run_skillrun(args: &[&str], skillrun_home: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .env("SKILLRUN_HOME", skillrun_home)
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

fn generated_capsule(label: &str) -> (PathBuf, PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let skillrun_home = output_root.join("skillrun-home");
    let capsule = output_root.join("refund");
    fs::create_dir_all(capsule.join("examples")).expect("capsule should be created");
    fs::write(
        capsule.join("SKILL.md"),
        "# Refund\n\nApprove eligible refund requests.\n",
    )
    .expect("SKILL.md should be written");
    fs::write(
        capsule.join("action.sh"),
        r#"cat > "$SKILLRUN_OUTPUT_JSON" <<'JSON'
{"ok":true,"output":{"decision":"approved","message":"refund approved"},"artifacts":[]}
JSON
"#,
    )
    .expect("action should be written");
    fs::write(
        capsule.join("examples").join("default.input.json"),
        r#"{
  "order_id": "order_default",
  "amount": 50,
  "reason": "damaged",
  "customer_tier": "standard"
}"#,
    )
    .expect("default input should be written");
    fs::write(
        capsule.join("skillrun.config.json"),
        r#"{
  "runtime": {
    "adapter": "command",
    "command": ["sh", "action.sh"],
    "timeout": "30s"
  },
  "input_schema": {
    "type": "object",
    "required": ["order_id", "amount", "reason", "customer_tier"],
    "additionalProperties": false,
    "properties": {
      "order_id": { "type": "string" },
      "amount": { "type": "number" },
      "reason": { "type": "string" },
      "customer_tier": { "type": "string" }
    }
  },
  "output_schema": {
    "type": "object",
    "required": ["decision", "message"],
    "additionalProperties": false,
    "properties": {
      "decision": { "type": "string" },
      "message": { "type": "string" }
    }
  }
}"#,
    )
    .expect("config should be written");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );

    (output_root, capsule, skillrun_home)
}

fn generated_package(label: &str) -> (PathBuf, PathBuf) {
    let (output_root, capsule, skillrun_home) = generated_capsule(label);
    let cwd_arg = capsule.to_string_lossy().to_string();

    let test = run_skillrun(&["test", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        test.status.success(),
        "test should create author run history before pack\nstderr: {}",
        String::from_utf8_lossy(&test.stderr)
    );

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        pack.status.success(),
        "pack should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&pack.stdout),
        String::from_utf8_lossy(&pack.stderr)
    );

    let archive = capsule
        .join("dist")
        .join(format!("refund-{}.skr", env!("CARGO_PKG_VERSION")));
    assert!(archive.is_file(), "package should exist at {archive:?}");
    (output_root, archive)
}

fn assert_success_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn assert_failure_json(output: &std::process::Output) -> Value {
    assert!(
        !output.status.success(),
        "command should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

#[test]
fn imported_capsule_stays_hidden_until_enabled_then_routes_via_router() {
    let (output_root, archive) = generated_package("router-imported-contract");
    let skillrun_home = output_root.join("consumer-home");
    let package_arg = archive.to_string_lossy().to_string();

    let imported = assert_success_json(&run_skillrun(
        &["import", &package_arg, "--json"],
        &skillrun_home,
    ));
    assert_eq!(imported["capsule"]["id"], "refund");
    assert_eq!(imported["capsule"]["source_type"], "imported_skr");
    assert_eq!(imported["capsule"]["enabled"], false);
    let imported_path = PathBuf::from(imported["capsule"]["path"].as_str().unwrap());
    assert!(
        !imported_path.join(".skillrun").join("runs").exists(),
        "imported package should not carry author run history"
    );

    let inventory = assert_success_json(&run_skillrun(
        &["consumer", "inventory", "--json"],
        &skillrun_home,
    ));
    assert_eq!(inventory["capsules"][0]["source_type"], "imported_skr");
    assert_eq!(inventory["capsules"][0]["enabled"], false);
    assert_eq!(inventory["capsules"][0]["readiness"]["status"], "ok");

    let exposure_disabled = assert_success_json(&run_skillrun(
        &["consumer", "exposure", "--json"],
        &skillrun_home,
    ));
    assert_eq!(
        exposure_disabled["tools"].as_array().unwrap().len(),
        0,
        "imported capsules must not be exposed before switchboard enable"
    );

    let router_disabled = assert_success_json(&run_skillrun(
        &["router", "serve", "--mcp", "--dry-run"],
        &skillrun_home,
    ));
    assert_eq!(router_disabled["ok"], true);
    assert_eq!(router_disabled["router"]["capsules"], 0);
    assert_eq!(router_disabled["tools"].as_array().unwrap().len(), 0);

    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(
        enable.status.success(),
        "switchboard enable should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&enable.stdout),
        String::from_utf8_lossy(&enable.stderr)
    );

    let exposure_enabled = assert_success_json(&run_skillrun(
        &["consumer", "exposure", "--json"],
        &skillrun_home,
    ));
    assert_eq!(exposure_enabled["tools"].as_array().unwrap().len(), 1);
    assert_eq!(exposure_enabled["tools"][0]["capsule_id"], "refund");
    assert_eq!(exposure_enabled["tools"][0]["enabled"], true);
    assert_eq!(exposure_enabled["tools"][0]["exposed"], true);

    let router_enabled = assert_success_json(&run_skillrun(
        &["router", "serve", "--mcp", "--dry-run"],
        &skillrun_home,
    ));
    assert_eq!(router_enabled["ok"], true);
    assert_eq!(router_enabled["router"]["capsules"], 1);
    assert_eq!(router_enabled["tools"][0]["capsule_id"], "refund");
    assert_eq!(router_enabled["tools"][0]["name"], "refund");
    assert!(router_enabled["tools"][0]["capsule_path"]
        .as_str()
        .unwrap_or_default()
        .replace('\\', "/")
        .contains("/consumer-home/capsules/refund"));
    assert_eq!(router_enabled["resources"][0]["capsule_id"], "refund");
    assert!(router_enabled["resources"][0]["uri"]
        .as_str()
        .unwrap_or_default()
        .starts_with("skillrun://router/refund/"));

    let mut client = ScriptedMcpClient::spawn_router(&skillrun_home);
    let init = client.initialize();
    assert_eq!(init["result"]["serverInfo"]["name"], "skillrun-router");
    client.initialized();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }));
    let tools = client.read_response("imported router tools/list response");
    assert_eq!(tools["result"]["tools"][0]["name"], "refund");

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "refund",
            "arguments": {
                "order_id": "order_imported_router_1001",
                "amount": 80,
                "reason": "damaged",
                "customer_tier": "standard"
            }
        }
    }));
    let call = client.read_response("imported router tools/call response");
    assert_eq!(call["id"], 3);
    assert_eq!(call["result"]["isError"], false);

    let run_record = fs::read_dir(imported_path.join(".skillrun").join("runs"))
        .expect("imported capsule runs directory should be readable after router call")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("record.json"))
        .find(|path| path.is_file())
        .expect("router call should create run evidence in imported capsule");
    let record: Value = serde_json::from_str(
        &fs::read_to_string(run_record).expect("run record should be readable"),
    )
    .expect("run record should parse");
    assert_eq!(record["mode"], "mcp");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn router_dry_run_exposes_only_enabled_ready_capsules() {
    let (output_root, capsule, skillrun_home) = generated_capsule("router-dry-run");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());

    let disabled = assert_success_json(&run_skillrun(
        &["router", "serve", "--mcp", "--dry-run"],
        &skillrun_home,
    ));
    assert_eq!(disabled["command"], "router serve --mcp");
    assert_eq!(disabled["schema_version"], "router.mcp.v1");
    assert_eq!(disabled["ok"], true);
    assert_eq!(disabled["router"]["snapshot"], true);
    assert_eq!(disabled["tools"].as_array().unwrap().len(), 0);
    assert_eq!(disabled["routes"].as_array().unwrap().len(), 0);
    assert_eq!(disabled["issues"].as_array().unwrap().len(), 0);

    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(enable.status.success());

    let enabled = assert_success_json(&run_skillrun(
        &["router", "serve", "--mcp", "--dry-run"],
        &skillrun_home,
    ));
    assert_eq!(enabled["ok"], true);
    assert_eq!(enabled["router"]["capsules"], 1);
    assert_eq!(enabled["tools"][0]["capsule_id"], "refund");
    assert_eq!(enabled["tools"][0]["name"], "refund");
    assert_eq!(
        enabled["tools"][0]["result_contract"],
        "SkillRun output/error envelope"
    );
    assert_eq!(enabled["resources"][0]["capsule_id"], "refund");
    assert!(enabled["resources"][0]["uri"]
        .as_str()
        .unwrap_or_default()
        .starts_with("skillrun://router/refund/"));
    assert!(enabled["resources"]
        .as_array()
        .unwrap()
        .iter()
        .all(|resource| !resource["path"]
            .as_str()
            .unwrap_or_default()
            .contains(".skillrun")));
    assert_eq!(enabled["routes"][0]["capsule_id"], "refund");
    assert_eq!(enabled["routes"][0]["state"], "routable");
    assert_eq!(enabled["routes"][0]["tool_name"], "refund");
    assert_eq!(
        enabled["routes"][0]["uri_prefix"],
        "skillrun://router/refund/"
    );
    assert_eq!(enabled["issues"].as_array().unwrap().len(), 0);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn router_status_reports_machine_readable_route_snapshot() {
    let (output_root, capsule, skillrun_home) = generated_capsule("router-status");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(enable.status.success());

    let status = assert_success_json(&run_skillrun(
        &["router", "status", "--json"],
        &skillrun_home,
    ));
    assert_eq!(status["command"], "router status");
    assert_eq!(status["schema_version"], "router.status.v1");
    assert_eq!(status["ok"], true);
    assert_eq!(status["router"]["snapshot"], true);
    assert_eq!(status["router"]["capsules"], 1);
    assert_eq!(status["tools"][0]["capsule_id"], "refund");
    assert_eq!(status["tools"][0]["name"], "refund");
    assert_eq!(
        status["resources"][0]["uri_prefix"],
        "skillrun://router/refund/"
    );
    assert_eq!(status["routes"][0]["capsule_id"], "refund");
    assert_eq!(status["routes"][0]["state"], "routable");
    assert_eq!(status["routes"][0]["tool_name"], "refund");
    assert_eq!(
        status["routes"][0]["uri_prefix"],
        "skillrun://router/refund/"
    );
    assert_eq!(status["routes"][0]["issue"], Value::Null);
    assert_eq!(status["issues"].as_array().unwrap().len(), 0);
    assert!(status["error"].is_null());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn router_reports_enabled_not_ready_route_with_recovery_hint() {
    let (output_root, capsule, skillrun_home) = generated_capsule("router-not-ready");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(enable.status.success());
    fs::remove_file(capsule.join("action.sh")).expect("action should be removable");

    let status = assert_success_json(&run_skillrun(
        &["router", "status", "--json"],
        &skillrun_home,
    ));
    assert_eq!(status["ok"], true);
    assert_eq!(status["tools"].as_array().unwrap().len(), 0);
    assert_eq!(status["resources"].as_array().unwrap().len(), 0);
    assert_eq!(status["routes"][0]["capsule_id"], "refund");
    assert_eq!(status["routes"][0]["state"], "blocked");
    assert_eq!(status["routes"][0]["issue"]["code"], "capsule-not-ready");
    assert_eq!(status["routes"][0]["issue"]["severity"], "warning");
    assert!(status["routes"][0]["recommended_action"]
        .as_str()
        .unwrap_or_default()
        .contains("skillrun manifest --cwd"));
    assert_eq!(status["issues"][0]["code"], "capsule-not-ready");
    assert_eq!(status["issues"][0]["severity"], "warning");
    assert!(status["error"].is_null());

    let dry_run = assert_success_json(&run_skillrun(
        &["router", "serve", "--mcp", "--dry-run"],
        &skillrun_home,
    ));
    assert_eq!(dry_run["ok"], true);
    assert_eq!(dry_run["routes"][0]["state"], "blocked");
    assert_eq!(dry_run["issues"][0]["code"], "capsule-not-ready");
    assert_eq!(dry_run["issues"][0]["severity"], "warning");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn router_dry_run_failure_uses_structured_error_contract() {
    let (first_root, first_capsule, skillrun_home) = generated_capsule("router-duplicate-first");
    let (second_root, second_capsule, _) = generated_capsule("router-duplicate-second");
    let first_arg = first_capsule.to_string_lossy().to_string();
    let second_arg = second_capsule.to_string_lossy().to_string();

    let add_first = run_skillrun(
        &["registry", "add", "--cwd", &first_arg, "--id", "refund_a"],
        &skillrun_home,
    );
    assert!(add_first.status.success());
    let add_second = run_skillrun(
        &["registry", "add", "--cwd", &second_arg, "--id", "refund_b"],
        &skillrun_home,
    );
    assert!(add_second.status.success());
    let enable_first = run_skillrun(&["switchboard", "enable", "refund_a"], &skillrun_home);
    assert!(enable_first.status.success());
    let enable_second = run_skillrun(&["switchboard", "enable", "refund_b"], &skillrun_home);
    assert!(enable_second.status.success());

    let dry_run = assert_failure_json(&run_skillrun(
        &["router", "serve", "--mcp", "--dry-run"],
        &skillrun_home,
    ));
    assert_eq!(dry_run["command"], "router serve --mcp --dry-run");
    assert_eq!(dry_run["schema_version"], "router.mcp.v1");
    assert_eq!(dry_run["ok"], false);
    assert_eq!(dry_run["error"]["code"], "duplicate-tool-name");
    assert!(dry_run["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("duplicate MCP tool name refund"));
    assert_eq!(dry_run["routes"].as_array().unwrap().len(), 2);
    assert!(dry_run["routes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|route| route["state"] == "blocked"));
    assert_eq!(dry_run["issues"].as_array().unwrap().len(), 2);
    assert!(dry_run["issues"]
        .as_array()
        .unwrap()
        .iter()
        .all(|issue| issue["code"] == "duplicate-tool-name" && issue["severity"] == "error"));

    let status = assert_failure_json(&run_skillrun(
        &["router", "status", "--json"],
        &skillrun_home,
    ));
    assert_eq!(status["command"], "router status");
    assert_eq!(status["schema_version"], "router.status.v1");
    assert_eq!(status["ok"], false);
    assert_eq!(status["error"]["code"], "duplicate-tool-name");
    assert_eq!(status["routes"].as_array().unwrap().len(), 2);
    assert_eq!(status["issues"].as_array().unwrap().len(), 2);

    fs::remove_dir_all(first_root).ok();
    fs::remove_dir_all(second_root).ok();
}

#[test]
fn router_stdio_lists_and_calls_enabled_capsule_tool() {
    let (output_root, capsule, skillrun_home) = generated_capsule("router-stdio");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(enable.status.success());

    let mut client = ScriptedMcpClient::spawn_router(&skillrun_home);
    let init = client.initialize();
    assert_eq!(init["result"]["serverInfo"]["name"], "skillrun-router");
    client.initialized();

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }));
    let tools = client.read_response("router tools/list response");
    assert_eq!(tools["result"]["tools"][0]["name"], "refund");

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "resources/list",
        "params": {}
    }));
    let resources = client.read_response("router resources/list response");
    let skill_uri = resources["result"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| resource["uri"].as_str().unwrap_or_default())
        .find(|uri| uri.ends_with("/SKILL.md"))
        .expect("router should expose SKILL.md resource")
        .to_string();
    assert!(skill_uri.starts_with("skillrun://router/refund/"));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "refund",
            "arguments": {
                "order_id": "order_router_1001",
                "amount": 120,
                "reason": "damaged",
                "customer_tier": "standard"
            }
        }
    }));
    let call = client.read_response("router tools/call response");
    assert_eq!(call["id"], 3);
    assert_eq!(call["result"]["isError"], false);
    assert!(call["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .contains("approved"));

    let run_record = fs::read_dir(capsule.join(".skillrun").join("runs"))
        .expect("runs directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("record.json"))
        .find(|path| path.is_file())
        .expect("router tools/call should create a SkillRun run record");
    let record: Value = serde_json::from_str(
        &fs::read_to_string(run_record).expect("run record should be readable"),
    )
    .expect("run record should parse");
    assert_eq!(record["mode"], "mcp");

    fs::remove_dir_all(output_root).ok();
}
