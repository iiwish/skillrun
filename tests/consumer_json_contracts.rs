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

fn generated_capsule(label: &str) -> (PathBuf, PathBuf, PathBuf) {
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

    (output_root, skillrun_home, capsule)
}

fn command_capsule(label: &str) -> (PathBuf, PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let skillrun_home = output_root.join("skillrun-home");
    let capsule = output_root.join("runs_contract");
    fs::create_dir_all(capsule.join("examples")).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# Runs Contract\n").expect("SKILL.md should be written");
    fs::write(
        capsule.join("action.sh"),
        r#"cat > "$SKILLRUN_OUTPUT_JSON" <<'JSON'
{"ok":true,"output":{"ok":true,"message":"fixture complete"},"artifacts":[]}
JSON
"#,
    )
    .expect("action should be written");
    fs::write(
        capsule.join("examples").join("default.input.json"),
        r#"{"task":"validate-runs"}"#,
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
    "required": ["task"],
    "additionalProperties": false,
    "properties": {
      "task": { "type": "string" }
    }
  },
  "output_schema": {
    "type": "object",
    "required": ["ok", "message"],
    "additionalProperties": false,
    "properties": {
      "ok": { "type": "boolean" },
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

    (output_root, skillrun_home, capsule)
}

fn instruction_only_capsule(label: &str) -> (PathBuf, PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let skillrun_home = output_root.join("skillrun-home");
    let capsule = output_root.join("instruction");
    fs::create_dir_all(&capsule).expect("instruction capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# instruction only\n")
        .expect("instruction SKILL.md should be written");
    (output_root, skillrun_home, capsule)
}

fn assert_success_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn assert_failure_json(output: &std::process::Output) -> Value {
    assert!(
        !output.status.success(),
        "command should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn assert_contract(actual: Value, fixture: &str, paths: &[(&Path, &str)]) {
    let mut normalized = actual;
    normalize_json(&mut normalized, paths, None);
    let expected: Value = serde_json::from_str(fixture).expect("fixture should be JSON");
    assert_eq!(
        normalized,
        expected,
        "normalized JSON did not match fixture\nactual:\n{}",
        serde_json::to_string_pretty(&normalized).expect("actual JSON should render")
    );
}

fn normalize_json(value: &mut Value, paths: &[(&Path, &str)], key: Option<&str>) {
    match value {
        Value::Object(object) => {
            for (child_key, child_value) in object {
                normalize_json(child_value, paths, Some(child_key));
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize_json(item, paths, key);
            }
        }
        Value::String(text) => {
            if key == Some("registered_at") {
                *text = "<timestamp>".to_string();
                return;
            }
            if matches!(key, Some("started_at" | "finished_at")) {
                *text = "<timestamp>".to_string();
                return;
            }
            if key == Some("run_id") && text.starts_with("run-") {
                *text = "<run_id>".to_string();
                return;
            }
            if key == Some("version") && text == env!("CARGO_PKG_VERSION") {
                *text = "<binary_version>".to_string();
                return;
            }
            if matches!(key, Some("os" | "arch" | "family")) {
                *text = format!("<{}>", key.expect("platform key should exist"));
                return;
            }
            if key == Some("detected") {
                *text = "<detected>".to_string();
                return;
            }
            if is_sha256(text) {
                *text = "<sha256>".to_string();
                return;
            }
            for (path, placeholder) in paths {
                replace_path_variants(text, path, placeholder);
            }
            replace_run_id_segments(text);
        }
        Value::Number(_) if key == Some("duration_ms") => {
            *value = Value::String("<duration_ms>".to_string());
        }
        _ => {}
    }
}

fn replace_path_variants(text: &mut String, path: &Path, placeholder: &str) {
    let mut variants = vec![path.to_string_lossy().to_string()];
    if let Ok(canonical) = fs::canonicalize(path) {
        variants.push(canonical.to_string_lossy().to_string());
    }
    variants.sort_by_key(|variant| std::cmp::Reverse(variant.len()));
    variants.dedup();

    for variant in variants {
        let forward = variant.replace('\\', "/");
        *text = text.replace(&variant, placeholder);
        *text = text.replace(&forward, placeholder);
    }
}

fn replace_run_id_segments(text: &mut String) {
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while let Some(relative_start) = text[index..].find("run-") {
        let start = index + relative_start;
        if !text[start + "run-".len()..]
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_digit())
        {
            output.push_str(&text[index..start + "run-".len()]);
            index = start + "run-".len();
            continue;
        }
        output.push_str(&text[index..start]);
        let mut end = start + "run-".len();
        for (offset, ch) in text[end..].char_indices() {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '.' {
                end = start + "run-".len() + offset + ch.len_utf8();
            } else {
                break;
            }
        }
        output.push_str("<run_id>");
        index = end;
    }

    output.push_str(&text[index..]);
    *text = output;
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[test]
fn host_status_json_matches_contract_fixture() {
    let output_root = temp_dir("json-contract-host-status");
    let skillrun_home = output_root.join("skillrun-home");
    let paths = [
        (&skillrun_home as &Path, "<skillrun_home>"),
        (Path::new(env!("CARGO_BIN_EXE_skillrun")), "<skillrun_exe>"),
    ];

    let host = assert_success_json(&run_skillrun(&["host", "status", "--json"], &skillrun_home));
    assert_contract(
        host,
        include_str!("fixtures/contracts/host_status.json"),
        &paths,
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn inspect_check_and_doctor_json_match_contract_fixtures() {
    let (output_root, skillrun_home, capsule) = generated_capsule("json-contract-runnable");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let paths = [(&capsule as &Path, "<capsule>")];

    let inspect = assert_success_json(&run_skillrun(
        &["inspect", "--json", "--cwd", &cwd_arg],
        &skillrun_home,
    ));
    assert_contract(
        inspect,
        include_str!("fixtures/contracts/inspect_runnable_python.json"),
        &paths,
    );

    let check = assert_success_json(&run_skillrun(
        &["check", "--json", "--cwd", &cwd_arg],
        &skillrun_home,
    ));
    assert_contract(
        check,
        include_str!("fixtures/contracts/check_runnable_python.json"),
        &paths,
    );

    let doctor = assert_success_json(&run_skillrun(
        &["doctor", "--json", "--cwd", &cwd_arg],
        &skillrun_home,
    ));
    assert_contract(
        doctor,
        include_str!("fixtures/contracts/doctor_runnable_python.json"),
        &paths,
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn instruction_only_json_matches_contract_fixtures() {
    let (output_root, skillrun_home, capsule) =
        instruction_only_capsule("json-contract-instruction");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let paths = [(&capsule as &Path, "<capsule>")];

    let inspect = assert_success_json(&run_skillrun(
        &["inspect", "--json", "--cwd", &cwd_arg],
        &skillrun_home,
    ));
    assert_contract(
        inspect,
        include_str!("fixtures/contracts/inspect_instruction_only.json"),
        &paths,
    );

    let check = assert_failure_json(&run_skillrun(
        &["check", "--json", "--cwd", &cwd_arg],
        &skillrun_home,
    ));
    assert_contract(
        check,
        include_str!("fixtures/contracts/check_instruction_only.json"),
        &paths,
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn registry_and_switchboard_json_match_contract_fixtures() {
    let (output_root, skillrun_home, capsule) = generated_capsule("json-contract-registry");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let paths = [
        (&capsule as &Path, "<capsule>"),
        (&skillrun_home as &Path, "<skillrun_home>"),
    ];

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(add.status.success());
    let enable = run_skillrun(&["switchboard", "enable", "refund"], &skillrun_home);
    assert!(enable.status.success());

    let registry = assert_success_json(&run_skillrun(
        &["registry", "list", "--json"],
        &skillrun_home,
    ));
    assert_contract(
        registry,
        include_str!("fixtures/contracts/registry_list_enabled.json"),
        &paths,
    );

    let switchboard = assert_success_json(&run_skillrun(
        &["switchboard", "list", "--json"],
        &skillrun_home,
    ));
    assert_contract(
        switchboard,
        include_str!("fixtures/contracts/switchboard_list_enabled.json"),
        &paths,
    );

    let consumer_inventory = assert_success_json(&run_skillrun(
        &["consumer", "inventory", "--json"],
        &skillrun_home,
    ));
    assert_contract(
        consumer_inventory,
        include_str!("fixtures/contracts/consumer_inventory_enabled.json"),
        &paths,
    );

    let consumer_exposure = assert_success_json(&run_skillrun(
        &["consumer", "exposure", "--json"],
        &skillrun_home,
    ));
    assert_contract(
        consumer_exposure,
        include_str!("fixtures/contracts/consumer_exposure_enabled.json"),
        &paths,
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn consumer_runs_json_match_contract_fixtures() {
    let (output_root, skillrun_home, capsule) = command_capsule("json-contract-runs");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let paths = [
        (&capsule as &Path, "<capsule>"),
        (&skillrun_home as &Path, "<skillrun_home>"),
    ];

    let add = run_skillrun(&["registry", "add", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        add.status.success(),
        "registry add should succeed\nstderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let run = run_skillrun(&["test", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        run.status.success(),
        "test run should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json: Value = serde_json::from_slice(&run.stdout).expect("run should emit JSON");
    let run_id = run_json["run_id"]
        .as_str()
        .expect("run should emit run_id")
        .to_string();

    let runs = assert_success_json(&run_skillrun(
        &["consumer", "runs", "list", "--json", "--limit", "1"],
        &skillrun_home,
    ));
    assert_contract(
        runs,
        include_str!("fixtures/contracts/consumer_runs_list_enabled.json"),
        &paths,
    );

    let inspect = assert_success_json(&run_skillrun(
        &[
            "consumer",
            "runs",
            "inspect",
            &run_id,
            "--json",
            "--capsule",
            "runs_contract",
        ],
        &skillrun_home,
    ));
    assert_contract(
        inspect,
        include_str!("fixtures/contracts/consumer_runs_inspect_enabled.json"),
        &paths,
    );

    fs::remove_dir_all(output_root).ok();
}
