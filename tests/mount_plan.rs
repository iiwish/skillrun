use serde_json::Value;
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

fn assert_success_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

#[test]
fn mount_plan_for_missing_config_is_plan_only_router_upsert() {
    let root = temp_dir("mount-plan-missing-config");
    let config = root.join("claude_desktop_config.json");
    let config_arg = config.to_string_lossy().to_string();

    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "plan",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--json",
    ]));

    assert_eq!(output["command"], "consumer mount plan");
    assert_eq!(output["schema_version"], "consumer.mount_plan.v1");
    assert_eq!(output["client"]["id"], "claude-desktop");
    assert_eq!(output["client"]["supported"], true);
    assert_eq!(output["client"]["detected"], false);
    assert_eq!(output["config"]["path_source"], "override");
    assert_eq!(output["config"]["exists"], false);
    assert_eq!(output["config"]["parseable"], true);
    assert_eq!(output["router"]["command"], "skillrun");
    assert_eq!(output["router"]["args"][0], "router");
    assert_eq!(output["changes"].as_array().unwrap().len(), 1);
    assert_eq!(output["changes"][0]["kind"], "upsert_mcp_server");
    assert!(output["changes"][0]["before"].is_null());
    assert_eq!(output["changes"][0]["after"]["command"], "skillrun");
    assert_eq!(output["changes"][0]["after"]["args"][0], "router");
    assert!(
        !config.exists(),
        "mount plan must not create or modify real config files"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_plan_sanitizes_existing_skillrun_entry_and_does_not_write_config() {
    let root = temp_dir("mount-plan-existing-config");
    fs::create_dir_all(&root).expect("test root should be created");
    let config = root.join("mcp.json");
    fs::write(
        &config,
        r#"{
  "mcpServers": {
    "skillrun": {
      "command": "old-skillrun",
      "args": ["serve", "--mcp"],
      "env": {
        "SECRET_TOKEN": "do-not-leak"
      }
    },
    "other": {
      "command": "other-server",
      "env": {
        "OTHER_SECRET": "also-do-not-leak"
      }
    }
  }
}"#,
    )
    .expect("test config should be written");
    let before = fs::read_to_string(&config).expect("test config should be readable");
    let config_arg = config.to_string_lossy().to_string();

    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "plan",
        "--client",
        "cursor",
        "--config",
        &config_arg,
        "--json",
    ]));
    let rendered = serde_json::to_string(&output).expect("plan should render");

    assert_eq!(output["client"]["id"], "cursor");
    assert_eq!(output["client"]["detected"], true);
    assert_eq!(output["config"]["exists"], true);
    assert_eq!(output["config"]["parseable"], true);
    assert_eq!(output["changes"].as_array().unwrap().len(), 1);
    assert_eq!(output["changes"][0]["before"]["command"], "old-skillrun");
    assert_eq!(output["changes"][0]["before"]["args"][0], "serve");
    assert!(output["changes"][0]["before"].get("env").is_none());
    assert!(!rendered.contains("SECRET_TOKEN"));
    assert!(!rendered.contains("OTHER_SECRET"));
    assert_eq!(
        fs::read_to_string(&config).expect("test config should remain readable"),
        before,
        "mount plan must not modify real config files"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_plan_returns_structured_warning_for_unsupported_clients() {
    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "plan",
        "--client",
        "unknown-client",
        "--json",
    ]));

    assert_eq!(output["client"]["id"], "unknown-client");
    assert_eq!(output["client"]["supported"], false);
    assert!(output["config"].is_null());
    assert_eq!(output["changes"].as_array().unwrap().len(), 0);
    assert_eq!(output["warnings"][0]["code"], "unsupported-client");
}

#[test]
fn mount_plan_refuses_to_patch_unparseable_config() {
    let root = temp_dir("mount-plan-bad-config");
    fs::create_dir_all(&root).expect("test root should be created");
    let config = root.join("bad.json");
    fs::write(&config, "{not-json").expect("test config should be written");
    let config_arg = config.to_string_lossy().to_string();

    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "plan",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--json",
    ]));

    assert_eq!(output["config"]["exists"], true);
    assert_eq!(output["config"]["parseable"], false);
    assert_eq!(output["changes"].as_array().unwrap().len(), 0);
    assert_eq!(output["warnings"][0]["code"], "unparseable-config");

    fs::remove_dir_all(root).ok();
}
