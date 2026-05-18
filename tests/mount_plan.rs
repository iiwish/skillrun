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

fn json_file(path: &PathBuf) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("json file should be readable"))
        .expect("json file should parse")
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
    let backup_path = output["backup"]["path"]
        .as_str()
        .expect("plan should expose the backup path pattern");
    assert!(
        backup_path.ends_with("claude_desktop_config.json.skillrun.<backup-id>.bak.json"),
        "plan backup path should match the apply backup naming contract: {backup_path}"
    );
    assert_eq!(output["backup"]["required_before_apply"], true);
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

#[test]
fn mount_apply_creates_missing_claude_config_and_rollback_removes_it() {
    let root = temp_dir("mount-apply-missing-config");
    let config = root.join("claude_desktop_config.json");
    let config_arg = config.to_string_lossy().to_string();

    let applied = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "apply",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--json",
    ]));

    assert_eq!(applied["command"], "consumer mount apply");
    assert_eq!(applied["schema_version"], "consumer.mount_apply.v1");
    assert_eq!(applied["client"]["id"], "claude-desktop");
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["config"]["exists_before"], false);
    assert_eq!(applied["config"]["written"], true);
    assert_eq!(applied["backup"]["created"], true);
    let backup_path = applied["backup"]["path"]
        .as_str()
        .expect("backup path should be present")
        .to_string();
    assert!(PathBuf::from(&backup_path).is_file());

    let written = json_file(&config);
    assert_eq!(written["mcpServers"]["skillrun"]["command"], "skillrun");
    assert_eq!(written["mcpServers"]["skillrun"]["args"][0], "router");

    let noop = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "apply",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--json",
    ]));
    assert_eq!(noop["applied"], false);
    assert_eq!(noop["backup"]["created"], false);

    let rolled_back = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "rollback",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--backup",
        &backup_path,
        "--json",
    ]));
    assert_eq!(rolled_back["command"], "consumer mount rollback");
    assert_eq!(rolled_back["schema_version"], "consumer.mount_rollback.v1");
    assert_eq!(rolled_back["rolled_back"], true);
    assert!(
        !config.exists(),
        "rollback should remove a config file created only for SkillRun"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_apply_and_rollback_restore_existing_skillrun_entry_without_leaking_other_config() {
    let root = temp_dir("mount-apply-existing-config");
    fs::create_dir_all(&root).expect("test root should be created");
    let config = root.join("claude_desktop_config.json");
    fs::write(
        &config,
        r#"{
  "mcpServers": {
    "skillrun": {
      "command": "old-skillrun",
      "args": ["serve", "--mcp"],
      "env": {
        "SECRET_TOKEN": "restore-me"
      }
    },
    "other": {
      "command": "other-server",
      "env": {
        "OTHER_SECRET": "preserve-me"
      }
    }
  }
}"#,
    )
    .expect("test config should be written");
    let config_arg = config.to_string_lossy().to_string();

    let applied = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "apply",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--json",
    ]));
    let rendered_apply = serde_json::to_string(&applied).expect("apply output should render");

    assert_eq!(applied["applied"], true);
    assert!(!rendered_apply.contains("SECRET_TOKEN"));
    assert!(!rendered_apply.contains("OTHER_SECRET"));
    let backup_path = applied["backup"]["path"]
        .as_str()
        .expect("backup path should be present")
        .to_string();
    let patched = json_file(&config);
    assert_eq!(patched["mcpServers"]["skillrun"]["command"], "skillrun");
    assert_eq!(patched["mcpServers"]["other"]["command"], "other-server");

    let rolled_back = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "rollback",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--backup",
        &backup_path,
        "--json",
    ]));

    assert_eq!(rolled_back["rolled_back"], true);
    let restored = json_file(&config);
    assert_eq!(
        restored["mcpServers"]["skillrun"]["command"],
        "old-skillrun"
    );
    assert_eq!(
        restored["mcpServers"]["skillrun"]["env"]["SECRET_TOKEN"],
        "restore-me"
    );
    assert_eq!(
        restored["mcpServers"]["other"]["env"]["OTHER_SECRET"],
        "preserve-me"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_rollback_refuses_when_skillrun_entry_changed_after_apply() {
    let root = temp_dir("mount-rollback-conflict");
    let config = root.join("claude_desktop_config.json");
    let config_arg = config.to_string_lossy().to_string();

    let applied = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "apply",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--json",
    ]));
    let backup_path = applied["backup"]["path"]
        .as_str()
        .expect("backup path should be present")
        .to_string();

    let mut changed = json_file(&config);
    changed["mcpServers"]["skillrun"]["env"] =
        serde_json::json!({ "USER_ADDED_TOKEN": "do-not-overwrite" });
    fs::write(
        &config,
        serde_json::to_string_pretty(&changed).expect("changed config should render"),
    )
    .expect("changed config should be written");

    let rolled_back = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "rollback",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--backup",
        &backup_path,
        "--json",
    ]));

    assert_eq!(rolled_back["rolled_back"], false);
    assert_eq!(rolled_back["warnings"][0]["code"], "rollback-conflict");
    assert_eq!(
        json_file(&config)["mcpServers"]["skillrun"]["env"]["USER_ADDED_TOKEN"],
        "do-not-overwrite"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_apply_for_cursor_is_plan_only_and_does_not_write_config() {
    let root = temp_dir("mount-apply-cursor-plan-only");
    let config = root.join("mcp.json");
    let config_arg = config.to_string_lossy().to_string();

    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "apply",
        "--client",
        "cursor",
        "--config",
        &config_arg,
        "--json",
    ]));

    assert_eq!(output["command"], "consumer mount apply");
    assert_eq!(output["applied"], false);
    assert_eq!(output["warnings"][0]["code"], "unsupported-client");
    assert!(
        !config.exists(),
        "cursor apply must not write config in v0.5.9"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_rollback_missing_backup_returns_json_warning_without_writing() {
    let root = temp_dir("mount-rollback-missing-backup");
    let config = root.join("claude_desktop_config.json");
    let backup = root.join("missing.skillrun.bak.json");
    let config_arg = config.to_string_lossy().to_string();
    let backup_arg = backup.to_string_lossy().to_string();

    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "rollback",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--backup",
        &backup_arg,
        "--json",
    ]));

    assert_eq!(output["command"], "consumer mount rollback");
    assert_eq!(output["rolled_back"], false);
    assert_eq!(output["client"]["supported"], true);
    assert_eq!(output["warnings"][0]["code"], "missing-backup");
    assert!(
        !config.exists(),
        "missing backup rollback must not write config"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_rollback_invalid_backup_returns_json_warning_without_writing() {
    let root = temp_dir("mount-rollback-invalid-backup");
    fs::create_dir_all(&root).expect("test root should be created");
    let config = root.join("claude_desktop_config.json");
    let backup = root.join("invalid.skillrun.bak.json");
    fs::write(&backup, "{not-json").expect("invalid backup should be written");
    let config_arg = config.to_string_lossy().to_string();
    let backup_arg = backup.to_string_lossy().to_string();

    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "rollback",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--backup",
        &backup_arg,
        "--json",
    ]));

    assert_eq!(output["command"], "consumer mount rollback");
    assert_eq!(output["rolled_back"], false);
    assert_eq!(output["client"]["supported"], true);
    assert_eq!(output["warnings"][0]["code"], "invalid-backup");
    assert!(
        !config.exists(),
        "invalid backup rollback must not write config"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn mount_rollback_unparseable_current_config_returns_json_warning_without_writing() {
    let root = temp_dir("mount-rollback-unparseable-current");
    let config = root.join("claude_desktop_config.json");
    let config_arg = config.to_string_lossy().to_string();

    let applied = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "apply",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--json",
    ]));
    let backup_path = applied["backup"]["path"]
        .as_str()
        .expect("backup path should be present")
        .to_string();
    fs::write(&config, "{not-json").expect("current config should be corrupted");

    let output = assert_success_json(&run_skillrun(&[
        "consumer",
        "mount",
        "rollback",
        "--client",
        "claude-desktop",
        "--config",
        &config_arg,
        "--backup",
        &backup_path,
        "--json",
    ]));

    assert_eq!(output["command"], "consumer mount rollback");
    assert_eq!(output["rolled_back"], false);
    assert_eq!(output["warnings"][0]["code"], "unparseable-config");
    assert_eq!(
        fs::read_to_string(&config).expect("corrupted config should remain readable"),
        "{not-json"
    );

    fs::remove_dir_all(root).ok();
}
