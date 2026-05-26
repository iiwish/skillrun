use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn generated_capsule(label: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();
    let skillrun_home = output_root.join("skillrun-home");

    let init = run_skillrun(
        &["init", "refund", "--python", "--output", &output_arg],
        &skillrun_home,
    );
    assert!(
        init.status.success(),
        "init should succeed\nstderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );

    (output_root, capsule)
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

fn assert_missing_capsule(entry: &Value) {
    assert_eq!(entry["id"], "refund");
    assert_eq!(entry["enabled"], false);
    assert_eq!(entry["manifest"]["present"], false);
    assert_eq!(entry["manifest"]["freshness"], "missing");
    assert!(entry.get("skill").is_none());
    assert!(entry.get("runtime").is_none());
    assert!(entry.get("tool").is_none());
    assert_eq!(entry["readiness"]["ok"], false);
    assert_eq!(entry["readiness"]["status"], "missing-path");
    assert!(entry["readiness"]["reason"]
        .as_str()
        .expect("missing path should include reason")
        .contains("cwd does not exist"));
    assert!(entry["readiness"]["next_step"]
        .as_str()
        .expect("missing path should include next step")
        .contains("registry remove"));
}

fn assert_invalid_manifest_capsule(entry: &Value) {
    assert_eq!(entry["id"], "refund");
    assert_eq!(entry["enabled"], false);
    assert_eq!(entry["manifest"]["present"], true);
    assert_eq!(entry["manifest"]["freshness"], "invalid");
    assert!(entry.get("skill").is_none());
    assert!(entry.get("runtime").is_none());
    assert!(entry.get("tool").is_none());
    assert_eq!(entry["readiness"]["ok"], false);
    assert_eq!(entry["readiness"]["status"], "invalid-manifest");
    assert!(entry["readiness"]["reason"]
        .as_str()
        .expect("invalid manifest should include reason")
        .contains("failed to parse"));
    assert!(entry["readiness"]["next_step"]
        .as_str()
        .expect("invalid manifest should include next step")
        .contains("skillrun manifest"));
}

#[test]
fn registry_list_json_treats_missing_registry_as_empty() {
    let skillrun_home = temp_dir("registry-empty-home");

    let list = run_skillrun(&["registry", "list", "--json"], &skillrun_home);
    let report = assert_success_json(&list);

    assert_eq!(report["command"], "registry list");
    assert_eq!(report["version"], 1);
    assert_eq!(report["capsules"].as_array().unwrap().len(), 0);
    assert!(report["registry_path"]
        .as_str()
        .expect("registry path should be a string")
        .contains("registry.json"));

    fs::remove_dir_all(skillrun_home).ok();
}

#[test]
fn registry_add_list_inspect_and_remove_local_capsule() {
    let (output_root, capsule) = generated_capsule("registry-add-list");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        add.status.success(),
        "registry add should succeed\nstderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let add_stdout = String::from_utf8(add.stdout).expect("stdout should be utf-8");
    assert!(add_stdout.contains("registered refund"));
    assert!(add_stdout.contains("enabled: false"));

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    let capsules = list["capsules"]
        .as_array()
        .expect("capsules should be array");
    assert_eq!(capsules.len(), 1);
    let entry = &capsules[0];
    assert_eq!(entry["id"], "refund");
    assert_eq!(entry["source_type"], "local_path");
    assert_eq!(entry["enabled"], false);
    assert_eq!(entry["manifest"]["present"], true);
    assert_eq!(entry["manifest"]["freshness"], "fresh");
    assert_eq!(entry["skill"]["name"], "refund");
    assert_eq!(entry["runtime"]["adapter"], "python");
    assert_eq!(entry["runtime"]["entrypoint"], "action.py");
    assert_eq!(entry["tool"]["name"], "refund");
    assert_eq!(entry["readiness"]["ok"], true);
    assert_eq!(entry["readiness"]["status"], "ok");

    let inspect = assert_success_json(&run_skillrun(
        &["registry", "inspect", "refund", "--json"],
        &skillrun_home,
    ));
    assert_eq!(inspect["command"], "registry inspect");
    assert_eq!(inspect["capsule"]["id"], "refund");
    assert_eq!(inspect["capsule"]["enabled"], false);

    let remove = run_skillrun(&["registry", "remove", "refund"], &skillrun_home);
    assert!(
        remove.status.success(),
        "registry remove should succeed\nstderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    assert!(
        capsule.join("SKILL.md").is_file(),
        "registry remove must not delete capsule files"
    );

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(list["capsules"].as_array().unwrap().len(), 0);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn registry_remove_delete_files_refuses_local_path_capsule() {
    let (output_root, capsule) = generated_capsule("registry-remove-local-files");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        add.status.success(),
        "registry add should succeed\nstderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let remove = run_skillrun(
        &["registry", "remove", "refund", "--delete-files"],
        &skillrun_home,
    );
    assert!(
        !remove.status.success(),
        "delete-files should refuse local_path entries"
    );
    assert!(
        String::from_utf8_lossy(&remove.stderr).contains("requires imported_skr"),
        "stderr should explain source_type boundary\nstderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    assert!(
        capsule.join("SKILL.md").is_file(),
        "delete-files refusal must preserve local capsule files"
    );

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(list["capsules"].as_array().unwrap().len(), 1);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn switchboard_enable_disable_and_list_json_updates_registry_state() {
    let (output_root, capsule) = generated_capsule("switchboard-happy");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());

    let before = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(before["command"], "switchboard list");
    assert_eq!(before["capsules"][0]["id"], "refund");
    assert_eq!(before["capsules"][0]["enabled"], false);
    assert_eq!(before["capsules"][0]["readiness"]["ok"], true);

    let exposure_before = assert_success_json(&run_skillrun(
        &["consumer", "exposure", "--json"],
        &skillrun_home,
    ));
    assert_eq!(exposure_before["command"], "consumer exposure");
    assert_eq!(exposure_before["schema_version"], "consumer.exposure.v1");
    assert_eq!(exposure_before["tools"].as_array().unwrap().len(), 0);

    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(
        enable.status.success(),
        "switchboard enable should succeed\nstderr: {}",
        String::from_utf8_lossy(&enable.stderr)
    );
    let enable_stdout = String::from_utf8(enable.stdout).expect("stdout should be utf-8");
    assert!(enable_stdout.contains("enabled refund"));

    let enabled = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(enabled["capsules"][0]["enabled"], true);

    let exposure_enabled = assert_success_json(&run_skillrun(
        &["consumer", "exposure", "--json"],
        &skillrun_home,
    ));
    assert_eq!(exposure_enabled["tools"].as_array().unwrap().len(), 1);
    assert_eq!(exposure_enabled["tools"][0]["capsule_id"], "refund");
    assert_eq!(exposure_enabled["tools"][0]["tool_name"], "refund");
    assert_eq!(exposure_enabled["tools"][0]["enabled"], true);
    assert_eq!(exposure_enabled["tools"][0]["exposed"], true);
    assert_eq!(exposure_enabled["tools"][0]["readiness_status"], "ok");

    let disable = run_skillrun(&["switchboard", "disable", "refund"], &skillrun_home);
    assert!(
        disable.status.success(),
        "switchboard disable should succeed\nstderr: {}",
        String::from_utf8_lossy(&disable.stderr)
    );
    let disabled = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(disabled["capsules"][0]["enabled"], false);

    let exposure_disabled = assert_success_json(&run_skillrun(
        &["consumer", "exposure", "--json"],
        &skillrun_home,
    ));
    assert_eq!(exposure_disabled["tools"].as_array().unwrap().len(), 0);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn consumer_runs_list_summarizes_registered_capsule_runs_without_inputs() {
    let (output_root, capsule) = generated_capsule("consumer-runs-list");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());

    let run = assert_success_json(&run_skillrun(&["test", "--cwd", &cwd_arg], &skillrun_home));
    let run_id = run["run_id"]
        .as_str()
        .expect("run envelope should include run_id");

    let list = assert_success_json(&run_skillrun(
        &["consumer", "runs", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(list["command"], "consumer runs list");
    assert_eq!(list["schema_version"], "consumer.runs.list.v1");
    assert_eq!(list["scope"]["kind"], "registry");
    assert!(list["scope"]["capsule_id"].is_null());
    assert!(list["scope"]["status"].is_null());
    assert!(list["scope"]["mode"].is_null());
    assert!(list["scope"]["ok"].is_null());
    assert!(list["scope"]["error_code"].is_null());
    assert!(list["scope"]["since"].is_null());
    assert!(list["scope"]["until"].is_null());

    let runs = list["runs"].as_array().expect("runs should be an array");
    assert_eq!(runs.len(), 1);
    let summary = &runs[0];
    assert_eq!(summary["run_id"], run_id);
    assert_eq!(summary["run_ref"]["kind"], "local_run");
    assert_eq!(summary["run_ref"]["capsule_id"], "refund");
    assert_eq!(summary["run_ref"]["run_id"], run_id);
    assert_eq!(summary["capsule_id"], "refund");
    assert_eq!(summary["mode"], "test");
    assert_eq!(summary["status"], "succeeded");
    assert_eq!(summary["ok"], true);
    assert!(summary["error_code"].is_null());
    let started_at = DateTime::parse_from_rfc3339(
        summary["started_at"]
            .as_str()
            .expect("summary should include started_at"),
    )
    .expect("started_at should be RFC3339")
    .with_timezone(&Utc);
    assert_eq!(summary["artifact_count"], 0);
    assert_eq!(summary["input_included"], false);
    assert!(summary.get("input").is_none());
    assert!(summary.get("envelope").is_none());
    assert!(summary.get("stdout").is_none());
    assert!(summary.get("stderr").is_none());

    let inspected = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "inspect",
            run_id,
            "--json",
            "--capsule",
            "refund",
        ],
        &skillrun_home,
    ));
    assert_eq!(inspected["command"], "consumer runs inspect");
    assert_eq!(inspected["schema_version"], "consumer.runs.inspect.v1");
    assert_eq!(inspected["ok"], true);
    assert_eq!(inspected["run_ref"]["kind"], "local_run");
    assert_eq!(inspected["run_ref"]["capsule_id"], "refund");
    assert_eq!(inspected["run_ref"]["run_id"], run_id);
    assert_eq!(inspected["capsule"]["id"], "refund");
    assert_eq!(inspected["record"]["run_id"], run_id);
    assert_eq!(inspected["record"]["mode"], "test");
    assert_eq!(inspected["record"]["status"], "succeeded");
    assert_eq!(inspected["input"]["included"], false);
    assert_eq!(inspected["input"]["available"], true);
    assert_eq!(inspected["envelope"]["included"], true);
    assert_eq!(inspected["envelope"]["status"], "ok");
    assert_eq!(inspected["envelope"]["value"]["ok"], true);
    assert_eq!(inspected["logs"]["stdout_available"], true);
    assert_eq!(inspected["logs"]["stderr_available"], true);
    assert_eq!(inspected["logs"]["stdout_included"], false);
    assert_eq!(inspected["logs"]["stderr_included"], false);
    assert_eq!(inspected["warnings"].as_array().unwrap().len(), 0);
    let rendered_inspect = serde_json::to_string(&inspected).expect("inspect output should render");
    assert!(
        !rendered_inspect.contains("refund_amount"),
        "inspect must not include full input content by default"
    );
    assert!(inspected.get("stdout").is_none());
    assert!(inspected.get("stderr").is_none());

    let scoped = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "list",
            "--json",
            "--capsule",
            "refund",
            "--limit",
            "1",
        ],
        &skillrun_home,
    ));
    assert_eq!(scoped["scope"]["capsule_id"], "refund");
    assert_eq!(scoped["runs"].as_array().unwrap().len(), 1);

    let filtered = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "list",
            "--json",
            "--status",
            "succeeded",
            "--mode",
            "test",
        ],
        &skillrun_home,
    ));
    assert_eq!(filtered["scope"]["status"], "succeeded");
    assert_eq!(filtered["scope"]["mode"], "test");
    assert!(filtered["scope"]["ok"].is_null());
    assert!(filtered["scope"]["error_code"].is_null());
    assert!(filtered["scope"]["since"].is_null());
    assert!(filtered["scope"]["until"].is_null());
    assert_eq!(filtered["runs"].as_array().unwrap().len(), 1);

    let since = (started_at - Duration::seconds(1)).to_rfc3339();
    let until = (started_at + Duration::seconds(1)).to_rfc3339();
    let time_filtered = assert_success_json(&run_skillrun(
        &[
            "consumer", "runs", "list", "--json", "--since", &since, "--until", &until,
        ],
        &skillrun_home,
    ));
    assert_eq!(time_filtered["scope"]["since"], since);
    assert_eq!(time_filtered["scope"]["until"], until);
    assert_eq!(time_filtered["runs"].as_array().unwrap().len(), 1);

    let after_run = (started_at + Duration::seconds(1)).to_rfc3339();
    let after_run_no_match = assert_success_json(&run_skillrun(
        &["consumer", "runs", "list", "--json", "--since", &after_run],
        &skillrun_home,
    ));
    assert_eq!(after_run_no_match["scope"]["since"], after_run);
    assert!(after_run_no_match["scope"]["until"].is_null());
    assert_eq!(after_run_no_match["runs"].as_array().unwrap().len(), 0);

    let reversed_range = run_skillrun(
        &[
            "consumer", "runs", "list", "--json", "--since", &until, "--until", &since,
        ],
        &skillrun_home,
    );
    assert!(!reversed_range.status.success());
    assert!(String::from_utf8_lossy(&reversed_range.stderr)
        .contains("--since must be earlier than or equal to --until"));

    let no_match = assert_success_json(&run_skillrun(
        &["consumer", "runs", "list", "--json", "--status", "failed"],
        &skillrun_home,
    ));
    assert_eq!(no_match["scope"]["status"], "failed");
    assert_eq!(no_match["runs"].as_array().unwrap().len(), 0);

    fs::write(
        capsule
            .join(".skillrun")
            .join("runs")
            .join(run_id)
            .join("output.json"),
        r#"{"ok":false,"error":{"code":"policy_violation"},"artifacts":[]}"#,
    )
    .expect("test should rewrite run envelope");

    let failed_by_code = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "list",
            "--json",
            "--ok",
            "false",
            "--error-code",
            "policy_violation",
        ],
        &skillrun_home,
    ));
    assert_eq!(failed_by_code["scope"]["ok"], false);
    assert_eq!(failed_by_code["scope"]["error_code"], "policy_violation");
    assert_eq!(failed_by_code["runs"].as_array().unwrap().len(), 1);
    assert_eq!(failed_by_code["runs"][0]["ok"], false);
    assert_eq!(failed_by_code["runs"][0]["error_code"], "policy_violation");
    assert!(failed_by_code["runs"][0].get("input").is_none());
    assert!(failed_by_code["runs"][0].get("envelope").is_none());

    let ok_true_no_match = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "list",
            "--json",
            "--ok",
            "true",
            "--error-code",
            "policy_violation",
        ],
        &skillrun_home,
    ));
    assert_eq!(ok_true_no_match["scope"]["ok"], true);
    assert_eq!(ok_true_no_match["scope"]["error_code"], "policy_violation");
    assert_eq!(ok_true_no_match["runs"].as_array().unwrap().len(), 0);

    let rebuilt = assert_success_json(&run_skillrun(
        &["consumer", "runs", "index", "rebuild", "--json"],
        &skillrun_home,
    ));
    assert_eq!(rebuilt["command"], "consumer runs index rebuild");
    assert_eq!(rebuilt["schema_version"], "consumer.runs.index.v1");
    assert_eq!(rebuilt["ok"], true);
    assert_eq!(rebuilt["capsules_scanned"], 1);
    assert_eq!(rebuilt["runs_indexed"], 1);

    let index_path = skillrun_home.join("runs-index.json");
    assert!(index_path.is_file(), "runs index file should be written");
    let index_text = fs::read_to_string(&index_path).expect("runs index should be readable");
    let index: Value = serde_json::from_str(&index_text).expect("runs index should be JSON");
    assert_eq!(index["schema_version"], "consumer.runs.index.v1");
    assert_eq!(index["runs"].as_array().unwrap().len(), 1);
    assert_eq!(index["runs"][0]["run_ref"]["kind"], "local_run");
    assert_eq!(index["runs"][0]["run_ref"]["capsule_id"], "refund");
    assert_eq!(index["runs"][0]["run_ref"]["run_id"], run_id);
    assert_eq!(index["runs"][0]["input_included"], false);
    assert!(index["runs"][0].get("input").is_none());
    assert!(index["runs"][0].get("envelope").is_none());
    assert!(index["runs"][0].get("stdout").is_none());
    assert!(index["runs"][0].get("stderr").is_none());
    assert!(
        !index_text.contains("refund_amount"),
        "runs index must not include full input content"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn consumer_runs_inspect_reports_ambiguous_run_ids_without_reading_internal_files() {
    let (output_root, capsule) = generated_capsule("consumer-runs-inspect-ambiguous");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add_a = run_skillrun(
        &["registry", "add", "--cwd", &cwd_arg, "--id", "refund-a"],
        &skillrun_home,
    );
    assert!(add_a.status.success());
    let add_b = run_skillrun(
        &["registry", "add", "--cwd", &cwd_arg, "--id", "refund-b"],
        &skillrun_home,
    );
    assert!(add_b.status.success());

    let run = assert_success_json(&run_skillrun(&["test", "--cwd", &cwd_arg], &skillrun_home));
    let run_id = run["run_id"]
        .as_str()
        .expect("run envelope should include run_id");

    let inspected = assert_success_json(&run_skillrun(
        &["consumer", "runs", "inspect", run_id, "--json"],
        &skillrun_home,
    ));

    assert_eq!(inspected["command"], "consumer runs inspect");
    assert_eq!(inspected["schema_version"], "consumer.runs.inspect.v1");
    assert_eq!(inspected["ok"], false);
    assert_eq!(inspected["error"]["code"], "AmbiguousRunId");
    assert!(inspected["error"]["message"]
        .as_str()
        .expect("ambiguous error should include message")
        .contains("--capsule"));
    let matches = inspected["matches"]
        .as_array()
        .expect("ambiguous response should include matches");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0]["run_id"], run_id);
    assert_eq!(matches[1]["run_id"], run_id);
    assert!(inspected.get("record").is_none());
    assert!(inspected.get("envelope").is_none());

    let scoped = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "inspect",
            run_id,
            "--json",
            "--capsule",
            "refund-a",
        ],
        &skillrun_home,
    ));
    assert_eq!(scoped["ok"], true);
    assert_eq!(scoped["run_ref"]["capsule_id"], "refund-a");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn consumer_runs_list_degrades_invalid_run_records_without_failing() {
    let (output_root, capsule) = generated_capsule("consumer-runs-invalid-record");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());

    let run = assert_success_json(&run_skillrun(&["test", "--cwd", &cwd_arg], &skillrun_home));
    let run_id = run["run_id"]
        .as_str()
        .expect("run envelope should include run_id");
    fs::write(
        capsule
            .join(".skillrun")
            .join("runs")
            .join(run_id)
            .join("record.json"),
        "{not-json",
    )
    .expect("test should corrupt run record");

    let list = assert_success_json(&run_skillrun(
        &["consumer", "runs", "list", "--json"],
        &skillrun_home,
    ));
    let runs = list["runs"].as_array().expect("runs should be an array");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["run_id"], run_id);
    assert_eq!(runs[0]["status"], "invalid-record");
    assert!(runs[0]["ok"].is_null());
    assert_eq!(runs[0]["input_included"], false);

    let inspected = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "inspect",
            run_id,
            "--json",
            "--capsule",
            "refund",
        ],
        &skillrun_home,
    ));
    assert_eq!(inspected["ok"], false);
    assert!(inspected["record"].is_null());
    assert_eq!(inspected["envelope"]["included"], true);
    assert_eq!(inspected["warnings"][0]["code"], "invalid-record");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn registry_and_switchboard_lists_tolerate_missing_capsule_paths() {
    let (output_root, capsule) = generated_capsule("registry-missing-path");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    fs::remove_dir_all(&capsule).expect("test should remove registered capsule path");

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(list["capsules"].as_array().unwrap().len(), 1);
    assert_missing_capsule(&list["capsules"][0]);

    let inspect = assert_success_json(&run_skillrun(
        &["registry", "inspect", "refund", "--json"],
        &skillrun_home,
    ));
    assert_missing_capsule(&inspect["capsule"]);

    let switchboard = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(switchboard["capsules"].as_array().unwrap().len(), 1);
    assert_missing_capsule(&switchboard["capsules"][0]);

    let consumer_inventory = assert_success_json(&run_skillrun(
        &["consumer", "inventory", "--json"],
        &skillrun_home,
    ));
    assert_eq!(
        consumer_inventory["schema_version"],
        "consumer.inventory.v1"
    );
    assert_eq!(consumer_inventory["capsules"].as_array().unwrap().len(), 1);
    assert_missing_capsule(&consumer_inventory["capsules"][0]);

    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(!enable.status.success());
    let stderr = String::from_utf8(enable.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("cannot enable refund"));
    assert!(stderr.contains("missing-path"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn registry_and_switchboard_lists_tolerate_invalid_manifest_entries() {
    let (output_root, capsule) = generated_capsule("registry-invalid-manifest");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    fs::write(
        capsule.join(".skillrun").join("manifest.generated.yaml"),
        "skill: [unterminated",
    )
    .expect("test should corrupt manifest");

    let list = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(list["capsules"].as_array().unwrap().len(), 1);
    assert_invalid_manifest_capsule(&list["capsules"][0]);

    let inspect = assert_success_json(&run_skillrun(
        &["registry", "inspect", "refund", "--json"],
        &skillrun_home,
    ));
    assert_invalid_manifest_capsule(&inspect["capsule"]);

    let switchboard = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_eq!(switchboard["capsules"].as_array().unwrap().len(), 1);
    assert_invalid_manifest_capsule(&switchboard["capsules"][0]);

    let consumer_inventory = assert_success_json(&run_skillrun(
        &["consumer", "inventory", "--json"],
        &skillrun_home,
    ));
    assert_eq!(consumer_inventory["command"], "consumer inventory");
    assert_eq!(
        consumer_inventory["schema_version"],
        "consumer.inventory.v1"
    );
    assert_eq!(consumer_inventory["capsules"].as_array().unwrap().len(), 1);
    assert_invalid_manifest_capsule(&consumer_inventory["capsules"][0]);

    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(!enable.status.success());
    let stderr = String::from_utf8(enable.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("cannot enable refund"));
    assert!(stderr.contains("invalid-manifest"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn consumer_exposure_hides_enabled_capsules_that_are_no_longer_ready() {
    let (output_root, capsule) = generated_capsule("consumer-exposure-not-ready");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(enable.status.success());

    fs::write(
        capsule.join(".skillrun").join("manifest.generated.yaml"),
        "skill: [unterminated",
    )
    .expect("test should corrupt manifest after enablement");

    let inventory = assert_success_json(&run_skillrun(
        &["consumer", "inventory", "--json"],
        &skillrun_home,
    ));
    assert_eq!(inventory["capsules"][0]["enabled"], true);
    assert_eq!(inventory["capsules"][0]["manifest"]["present"], true);
    assert_eq!(inventory["capsules"][0]["manifest"]["freshness"], "invalid");
    assert_eq!(inventory["capsules"][0]["readiness"]["ok"], false);
    assert_eq!(
        inventory["capsules"][0]["readiness"]["status"],
        "invalid-manifest"
    );

    let exposure = assert_success_json(&run_skillrun(
        &["consumer", "exposure", "--json"],
        &skillrun_home,
    ));
    assert_eq!(exposure["tools"].as_array().unwrap().len(), 0);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn registry_add_rejects_duplicate_ids() {
    let (output_root, capsule) = generated_capsule("registry-duplicate-id");
    let skillrun_home = output_root.join("skillrun-home");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let first = run_skillrun(
        &["registry", "add", "--cwd", &cwd_arg, "--id", "refund-prod"],
        &skillrun_home,
    );
    assert!(first.status.success());

    let duplicate = run_skillrun(
        &["registry", "add", "--cwd", &cwd_arg, "--id", "refund-prod"],
        &skillrun_home,
    );
    assert!(!duplicate.status.success());
    let stderr = String::from_utf8(duplicate.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("registry id already exists"));
    assert!(stderr.contains("refund-prod"));

    fs::remove_dir_all(output_root).ok();
}
