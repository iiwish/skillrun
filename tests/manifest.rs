use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_skillrun(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .output()
        .expect("skillrun binary should run")
}

fn run_skillrun_with_env(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_skillrun"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("skillrun binary should run")
}

fn temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("skillrun-{label}-{}-{stamp}", std::process::id()))
}

fn generated_manifest(capsule: &Path) -> PathBuf {
    capsule.join(".skillrun").join("manifest.generated.yaml")
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "manifest should contain {expected:?}\n{text}"
    );
}

fn has_source_sha(manifest: &str, path: &str) -> bool {
    let expected_path = format!("path: {path}");
    let lines: Vec<&str> = manifest.lines().collect();
    lines.windows(2).any(|window| {
        window[0].trim_start().starts_with(&expected_path)
            && window[1]
                .trim_start()
                .strip_prefix("sha256: ")
                .is_some_and(is_64_hex)
    })
}

fn assert_source_hash(manifest: &str, path: &str) {
    assert!(
        has_source_sha(manifest, path),
        "manifest should record a 64-hex source hash for {path}\n{manifest}"
    );
}

fn manifest_yaml(capsule: &Path) -> YamlValue {
    let manifest_text =
        fs::read_to_string(generated_manifest(capsule)).expect("manifest should be readable");
    serde_yaml::from_str(&manifest_text).expect("manifest should parse as YAML")
}

fn yaml_at<'a>(value: &'a YamlValue, path: &[&str]) -> &'a YamlValue {
    let mut current = value;
    for segment in path {
        current = if let Some(sequence) = current.as_sequence() {
            let index = segment
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("YAML path {} needs a sequence index", path.join(".")));
            sequence
                .get(index)
                .unwrap_or_else(|| panic!("missing YAML path {}", path.join(".")))
        } else {
            current
                .get(*segment)
                .unwrap_or_else(|| panic!("missing YAML path {}", path.join(".")))
        };
    }
    current
}

fn yaml_str_at<'a>(value: &'a YamlValue, path: &[&str]) -> &'a str {
    yaml_at(value, path)
        .as_str()
        .unwrap_or_else(|| panic!("YAML path {} should be a string", path.join(".")))
}

fn is_64_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn valid_js_action() -> &'static str {
    r#"
export const inputSchema = {
  type: "object",
  required: ["order_id"],
  additionalProperties: false,
  properties: {
    order_id: { type: "string", minLength: 1 }
  }
};

export const outputSchema = {
  type: "object",
  required: ["decision"],
  additionalProperties: false,
  properties: {
    decision: { type: "string", enum: ["approved", "rejected"] }
  }
};
"#
}

#[test]
fn manifest_generates_yaml_with_hashes_and_pydantic_schemas() {
    let output_root = temp_dir("manifest");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--python", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(manifest.status.success());
    let manifest_path = generated_manifest(&capsule);
    assert!(manifest_path.is_file());
    let manifest_text = fs::read_to_string(&manifest_path).expect("manifest should be readable");

    assert_contains(&manifest_text, "manifest_version: 0.1.0");
    assert_contains(&manifest_text, "generated_by: skillrun@0.4.2");
    assert_contains(&manifest_text, "name: refund");
    assert_contains(&manifest_text, "adapter: python");
    assert_contains(&manifest_text, "entrypoint: action.py");
    assert_contains(&manifest_text, "order_id");
    assert_contains(&manifest_text, "manager_approval_id");
    assert_contains(&manifest_text, "decision");
    assert_source_hash(&manifest_text, "SKILL.md");
    assert_source_hash(&manifest_text, "action.py");
    assert_source_hash(&manifest_text, "skillrun.config.json");
    let manifest_yaml = manifest_yaml(&capsule);
    assert_eq!(
        yaml_str_at(
            &manifest_yaml,
            &["runtime", "requirements", "executable", "name"]
        ),
        "python"
    );
    assert_eq!(
        yaml_str_at(
            &manifest_yaml,
            &["runtime", "requirements", "executable", "version"]
        ),
        ">=3.10"
    );
    assert_eq!(
        yaml_str_at(
            &manifest_yaml,
            &["runtime", "requirements", "packages", "0", "name"]
        ),
        "pydantic"
    );
    assert_eq!(
        yaml_str_at(
            &manifest_yaml,
            &["runtime", "requirements", "packages", "0", "version"]
        ),
        ">=2,<3"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_generates_yaml_for_js_alpha_with_explicit_json_schemas() {
    let output_root = temp_dir("manifest-js");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--js", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );
    let manifest_path = generated_manifest(&capsule);
    assert!(manifest_path.is_file());
    let manifest_text = fs::read_to_string(&manifest_path).expect("manifest should be readable");

    assert_contains(&manifest_text, "adapter: node");
    assert_contains(&manifest_text, "entrypoint: action.mjs");
    assert_contains(&manifest_text, "path: action.mjs");
    assert_contains(&manifest_text, "order_id");
    assert_contains(&manifest_text, "decision");
    assert_source_hash(&manifest_text, "SKILL.md");
    assert_source_hash(&manifest_text, "action.mjs");
    assert_source_hash(&manifest_text, "skillrun.config.json");
    let manifest_yaml = manifest_yaml(&capsule);
    assert_eq!(
        yaml_str_at(
            &manifest_yaml,
            &["runtime", "requirements", "executable", "name"]
        ),
        "node"
    );
    assert_eq!(
        yaml_str_at(
            &manifest_yaml,
            &["runtime", "requirements", "executable", "version"]
        ),
        ">=18"
    );
    assert_eq!(
        yaml_at(&manifest_yaml, &["runtime", "requirements", "packages"])
            .as_sequence()
            .expect("packages should be a sequence")
            .len(),
        0
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_uses_config_entrypoint_before_action_file_convention() {
    let output_root = temp_dir("manifest-js-config-first");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--js", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    fs::write(capsule.join("action.py"), "not valid python\n").expect("extra action should write");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(
        manifest.status.success(),
        "config should select action.mjs before convention\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );
    let manifest_text =
        fs::read_to_string(generated_manifest(&capsule)).expect("manifest should be readable");
    assert_contains(&manifest_text, "path: action.mjs");
    assert!(!manifest_text.contains("path: action.py"));

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_uses_action_mjs_convention_without_config() {
    let output_root = temp_dir("manifest-js-convention");
    let capsule = output_root.join("refund");
    fs::create_dir_all(&capsule).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# refund").expect("skill should be written");
    fs::write(capsule.join("action.mjs"), valid_js_action()).expect("action should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(
        manifest.status.success(),
        "manifest should use action.mjs convention\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );
    let manifest_text =
        fs::read_to_string(generated_manifest(&capsule)).expect("manifest should be readable");
    assert_contains(&manifest_text, "adapter: node");
    assert_contains(&manifest_text, "entrypoint: action.mjs");
    assert_source_hash(&manifest_text, "action.mjs");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_fails_for_ambiguous_action_files_without_config() {
    let output_root = temp_dir("manifest-ambiguous-actions");
    let capsule = output_root.join("refund");
    fs::create_dir_all(&capsule).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# refund").expect("skill should be written");
    fs::write(
        capsule.join("action.py"),
        "from pydantic import BaseModel\n",
    )
    .expect("python action should be written");
    fs::write(capsule.join("action.mjs"), valid_js_action()).expect("js action should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("ambiguous action files"));
    assert!(stderr.contains("skillrun.config.json"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_fails_for_unsupported_typescript_action_without_config() {
    let output_root = temp_dir("manifest-typescript");
    let capsule = output_root.join("refund");
    fs::create_dir_all(&capsule).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# refund").expect("skill should be written");
    fs::write(
        capsule.join("action.ts"),
        "export const inputSchema = {};\n",
    )
    .expect("ts action should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("action.ts is not supported"));
    assert!(stderr.contains("compile to action.mjs"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_fails_when_js_schema_export_is_missing() {
    let output_root = temp_dir("manifest-js-missing-schema");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--js", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    fs::write(
        capsule.join("action.mjs"),
        "export const inputSchema = { type: \"object\" };\n",
    )
    .expect("action should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("missing outputSchema export"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_fails_when_js_schema_export_is_not_object() {
    let output_root = temp_dir("manifest-js-invalid-schema");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--js", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    fs::write(
        capsule.join("action.mjs"),
        "export const inputSchema = [];\nexport const outputSchema = { type: \"object\" };\n",
    )
    .expect("action should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("inputSchema must export a JSON Schema object"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_fails_clearly_when_node_is_missing() {
    let output_root = temp_dir("manifest-js-missing-node");
    let output_arg = output_root.to_string_lossy().to_string();
    let init = run_skillrun(&["init", "refund", "--js", "--output", &output_arg]);
    assert!(init.status.success());

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun_with_env(&["manifest", "--cwd", &cwd_arg], &[("PATH", "")]);

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to spawn Node metadata extractor"));
    assert!(stderr.contains("program not found"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_fails_when_action_is_missing() {
    let output_root = temp_dir("manifest-missing-action");
    let capsule = output_root.join("instruction_only");
    fs::create_dir_all(&capsule).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# instruction only").expect("skill should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg]);

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("missing action.py"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn manifest_metadata_extraction_times_out() {
    let output_root = temp_dir("manifest-timeout");
    let capsule = output_root.join("slow_action");
    fs::create_dir_all(&capsule).expect("capsule should be created");
    fs::write(capsule.join("SKILL.md"), "# slow action").expect("skill should be written");
    fs::write(capsule.join("action.py"), "import time\ntime.sleep(5)\n")
        .expect("action should be written");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let manifest = run_skillrun_with_env(
        &["manifest", "--cwd", &cwd_arg],
        &[("SKILLRUN_METADATA_TIMEOUT_MS", "200")],
    );

    assert!(!manifest.status.success());
    let stderr = String::from_utf8(manifest.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("metadata extraction timed out"));
    assert!(!generated_manifest(&capsule).exists());

    fs::remove_dir_all(output_root).ok();
}
