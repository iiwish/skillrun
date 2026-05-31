use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_skillrun(args: &[&str], skillrun_home: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skillrun"))
        .args(args)
        .env("SKILLRUN_HOME", skillrun_home)
        .output()
        .expect("skillrun should run")
}

fn output_root(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("skillrun-team-catalog-{name}-{nonce}"))
}

fn write_catalog(path: &Path) {
    fs::write(
        path,
        r#"{
  "schema_version": "team.catalog.v1",
  "catalog_id": "acme.internal",
  "name": "Acme AI Capabilities",
  "description": "Internal team catalog.",
  "updated_at": "2026-05-26T10:00:00Z",
  "items": [
    {
      "id": "refund",
      "kind": "skillrun.skr",
      "name": "Refund Decision",
      "description": "Evaluate refund requests.",
      "version": "0.1.0",
      "publisher": {
        "name": "Acme Operations"
      },
      "source": {
        "type": "https",
        "url": "https://example.com/refund-0.1.0.skr",
        "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
      },
      "requirements": [
        {
          "kind": "python",
          "summary": "Python 3.11+"
        }
      ],
      "permissions_summary": [
        "reads provided refund input"
      ],
      "trust_note": "Review before enabling.",
      "tags": ["ops", "refund"]
    },
    {
      "id": "plain-skill",
      "kind": "agent.skill",
      "name": "Plain Skill",
      "description": "Display-only Agent Skill.",
      "version": "0.1.0",
      "source": {
        "type": "file",
        "url": "./skills/plain"
      }
    }
  ]
}
"#,
    )
    .expect("catalog should be written");
}

fn archive_name(stem: &str) -> String {
    format!("{stem}-{}.skr", env!("CARGO_PKG_VERSION"))
}

fn generated_package(label: &str) -> (PathBuf, PathBuf) {
    let output_root = output_root(label);
    let output_arg = output_root.to_string_lossy().to_string();
    let author_home = output_root.join("author-home");

    let init = run_skillrun(
        &["init", "refund", "--python", "--output", &output_arg],
        &author_home,
    );
    assert!(
        init.status.success(),
        "init should succeed\nstderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let capsule = output_root.join("refund");
    let cwd_arg = capsule.to_string_lossy().to_string();
    let manifest = run_skillrun(&["manifest", "--cwd", &cwd_arg], &author_home);
    assert!(
        manifest.status.success(),
        "manifest should succeed\nstderr: {}",
        String::from_utf8_lossy(&manifest.stderr)
    );

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg], &author_home);
    assert!(
        pack.status.success(),
        "pack should succeed\nstderr: {}",
        String::from_utf8_lossy(&pack.stderr)
    );

    (
        output_root,
        capsule.join("dist").join(archive_name("refund")),
    )
}

fn sha256_file(path: &Path) -> String {
    let mut file = fs::File::open(path).expect("package should be readable");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .expect("package should be read");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_file_catalog(path: &Path, source_url: &str, sha256: &str) {
    fs::write(
        path,
        format!(
            r#"{{
  "schema_version": "team.catalog.v1",
  "catalog_id": "acme.internal",
  "name": "Acme AI Capabilities",
  "updated_at": "2026-05-26T10:00:00Z",
  "items": [
    {{
      "id": "refund",
      "kind": "skillrun.skr",
      "name": "Refund Decision",
      "description": "Evaluate refund requests.",
      "version": "0.1.0",
      "source": {{
        "type": "file",
        "url": "{source_url}",
        "sha256": "{sha256}"
      }}
    }}
  ]
}}
"#
        ),
    )
    .expect("catalog should be written");
}

fn assert_success_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "expected success\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn assert_failure_json(output: &std::process::Output) -> Value {
    assert!(
        !output.status.success(),
        "expected failure\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

#[test]
fn team_catalog_inspect_reports_items_without_downloading_sources() {
    let root = output_root("inspect");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");

    let output = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "inspect",
            catalog.to_str().unwrap(),
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["schema_version"], "team.catalog.inspect.v1");
    assert_eq!(output["ok"], true);
    assert_eq!(output["catalog"]["catalog_id"], "acme.internal");
    assert_eq!(output["catalog"]["items"], 2);
    assert_eq!(output["items"][0]["id"], "refund");
    assert_eq!(output["items"][0]["installable"], true);
    assert_eq!(output["items"][0]["installed"], false);
    assert_eq!(output["items"][1]["id"], "plain-skill");
    assert_eq!(output["items"][1]["installable"], false);
    assert_eq!(
        output["items"][1]["warnings"][0]["code"],
        "catalog.item.display_only"
    );
}

#[test]
fn team_catalog_status_reports_missing_and_display_only_items() {
    let root = output_root("status-missing");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");

    let output = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "status",
            catalog.to_str().unwrap(),
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["command"], "team catalog status");
    assert_eq!(output["schema_version"], "team.catalog.status.v1");
    assert_eq!(output["ok"], true);
    assert_eq!(output["summary"]["items"], 2);
    assert_eq!(output["summary"]["missing"], 1);
    assert_eq!(output["summary"]["blocked"], 1);
    assert_eq!(output["items"][0]["id"], "refund");
    assert_eq!(output["items"][0]["status"], "missing");
    assert_eq!(output["items"][0]["recommended_action"], "install");
    assert_eq!(output["items"][0]["install_plan_available"], true);
    assert_eq!(output["items"][0]["registry"]["installed"], false);
    assert_eq!(output["items"][1]["id"], "plain-skill");
    assert_eq!(output["items"][1]["status"], "blocked");
    assert_eq!(output["items"][1]["recommended_action"], "none");
    assert_eq!(output["items"][1]["install_plan_available"], false);
    assert_eq!(
        output["items"][1]["warnings"][0]["code"],
        "catalog.item.display_only"
    );
}

#[test]
fn team_catalog_status_reports_replace_available_for_imported_skr_entry() {
    let root = output_root("status-replace");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    fs::write(
        home.join("registry.json"),
        r#"{
  "version": 1,
  "capsules": [
    {
      "id": "refund",
      "path": "/tmp/refund-imported",
      "source_type": "imported_skr",
      "enabled": true,
      "registered_at": "2026-05-26T10:00:00Z"
    }
  ]
}
"#,
    )
    .unwrap();

    let output = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "status",
            catalog.to_str().unwrap(),
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["summary"]["replace_available"], 1);
    assert_eq!(output["items"][0]["status"], "replace_available");
    assert_eq!(output["items"][0]["recommended_action"], "replace");
    assert_eq!(output["items"][0]["install_plan_available"], true);
    assert_eq!(output["items"][0]["registry"]["installed"], true);
    assert_eq!(
        output["items"][0]["registry"]["source_type"],
        "imported_skr"
    );
    assert_eq!(output["items"][0]["registry"]["enabled"], true);
}

#[test]
fn team_catalog_status_reports_blocked_for_local_path_registry_conflict() {
    let root = output_root("status-conflict");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    fs::write(
        home.join("registry.json"),
        r#"{
  "version": 1,
  "capsules": [
    {
      "id": "refund",
      "path": "/tmp/refund-local",
      "source_type": "local_path",
      "enabled": false,
      "registered_at": "2026-05-26T10:00:00Z"
    }
  ]
}
"#,
    )
    .unwrap();

    let output = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "status",
            catalog.to_str().unwrap(),
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["summary"]["blocked"], 2);
    assert_eq!(output["items"][0]["status"], "blocked");
    assert_eq!(output["items"][0]["recommended_action"], "resolve_conflict");
    assert_eq!(output["items"][0]["install_plan_available"], false);
    assert_eq!(output["items"][0]["registry"]["source_type"], "local_path");
    assert_eq!(
        output["items"][0]["warnings"][0]["code"],
        "catalog.registry_conflict"
    );
}

#[test]
fn team_catalog_install_plan_reports_import_without_downloading_package() {
    let root = output_root("plan-import");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");

    let output = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "plan",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["schema_version"], "team.catalog.install_plan.v1");
    assert_eq!(output["ok"], true);
    assert_eq!(output["catalog_id"], "acme.internal");
    assert_eq!(output["item"]["id"], "refund");
    assert_eq!(output["item"]["source_type"], "https");
    assert_eq!(output["registry"]["installed"], false);
    assert_eq!(output["actions"][0]["type"], "import");
    assert_eq!(output["actions"][0]["replace"], false);
    assert_eq!(output["warnings"][0]["code"], "trust.not_proven");
}

#[test]
fn team_catalog_install_plan_rejects_display_only_items() {
    let root = output_root("plan-display-only");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");

    let output = assert_failure_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "plan",
            catalog.to_str().unwrap(),
            "plain-skill",
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["schema_version"], "team.catalog.install_plan.v1");
    assert_eq!(output["ok"], false);
    assert_eq!(output["error"]["code"], "catalog.item_not_installable");
}

#[test]
fn team_catalog_install_plan_fails_closed_on_local_path_registry_conflict() {
    let root = output_root("plan-conflict");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    fs::write(
        home.join("registry.json"),
        r#"{
  "version": 1,
  "capsules": [
    {
      "id": "refund",
      "path": "/tmp/refund-local",
      "source_type": "local_path",
      "enabled": false,
      "registered_at": "2026-05-26T10:00:00Z"
    }
  ]
}
"#,
    )
    .unwrap();

    let output = assert_failure_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "plan",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["schema_version"], "team.catalog.install_plan.v1");
    assert_eq!(output["ok"], false);
    assert_eq!(output["error"]["code"], "catalog.registry_conflict");
}

#[test]
fn team_catalog_install_plan_reports_replace_for_imported_skr_entry() {
    let root = output_root("plan-replace");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    fs::write(
        home.join("registry.json"),
        r#"{
  "version": 1,
  "capsules": [
    {
      "id": "refund",
      "path": "/tmp/refund-imported",
      "source_type": "imported_skr",
      "enabled": true,
      "registered_at": "2026-05-26T10:00:00Z"
    }
  ]
}
"#,
    )
    .unwrap();

    let output = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "plan",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["ok"], true);
    assert_eq!(output["registry"]["installed"], true);
    assert_eq!(output["registry"]["source_type"], "imported_skr");
    assert_eq!(output["registry"]["enabled"], true);
    assert_eq!(output["actions"][0]["replace"], true);
    assert_eq!(output["actions"][0]["requires_confirmation"], true);
}

#[test]
fn team_catalog_install_apply_imports_file_source_after_checksum_verification() {
    let (root, archive_path) = generated_package("apply-import");
    let home = root.join("consumer-home");
    let catalog = root.join("team-catalog.json");
    let relative_package = archive_path
        .strip_prefix(&root)
        .expect("archive should be under root")
        .to_string_lossy()
        .replace('\\', "/");
    write_file_catalog(&catalog, &relative_package, &sha256_file(&archive_path));

    let output = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "apply",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["schema_version"], "team.catalog.install_apply.v1");
    assert_eq!(output["ok"], true);
    assert_eq!(output["catalog_id"], "acme.internal");
    assert_eq!(output["item_id"], "refund");
    assert_eq!(output["download"]["source_type"], "file");
    assert_eq!(output["download"]["sha256_verified"], true);
    assert_eq!(output["import"]["schema_version"], "import.v1");
    assert_eq!(output["import"]["id"], "refund");
    assert_eq!(output["import"]["source_type"], "imported_skr");
    assert_eq!(output["import"]["enabled"], false);
    assert_eq!(output["import"]["replaced"], false);

    let inventory = assert_success_json(&run_skillrun(&["consumer", "inventory", "--json"], &home));
    assert_eq!(inventory["capsules"][0]["id"], "refund");
    assert_eq!(inventory["capsules"][0]["enabled"], false);
    assert_eq!(inventory["capsules"][0]["source_type"], "imported_skr");

    fs::remove_dir_all(root).ok();
}

#[test]
fn team_catalog_install_apply_replace_preserves_enabled_state() {
    let (root, archive_path) = generated_package("apply-replace");
    let home = root.join("consumer-home");
    let catalog = root.join("team-catalog.json");
    let relative_package = archive_path
        .strip_prefix(&root)
        .expect("archive should be under root")
        .to_string_lossy()
        .replace('\\', "/");
    write_file_catalog(&catalog, &relative_package, &sha256_file(&archive_path));

    let first = run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "apply",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    );
    assert!(first.status.success());
    let enable = run_skillrun(&["switchboard", "enable", "refund"], &home);
    assert!(enable.status.success());

    let replaced = assert_success_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "apply",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    ));

    assert_eq!(replaced["import"]["replaced"], true);
    assert_eq!(replaced["import"]["enabled"], true);

    fs::remove_dir_all(root).ok();
}

#[test]
fn team_catalog_install_apply_rejects_checksum_mismatch_without_importing() {
    let (root, archive_path) = generated_package("apply-checksum");
    let home = root.join("consumer-home");
    let catalog = root.join("team-catalog.json");
    let relative_package = archive_path
        .strip_prefix(&root)
        .expect("archive should be under root")
        .to_string_lossy()
        .replace('\\', "/");
    write_file_catalog(
        &catalog,
        &relative_package,
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    );

    let output = assert_failure_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "apply",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["schema_version"], "team.catalog.install_apply.v1");
    assert_eq!(output["ok"], false);
    assert_eq!(output["error"]["code"], "catalog.package_checksum_mismatch");
    let inventory = assert_success_json(&run_skillrun(&["consumer", "inventory", "--json"], &home));
    assert!(inventory["capsules"].as_array().unwrap().is_empty());

    fs::remove_dir_all(root).ok();
}

#[test]
fn team_catalog_install_apply_fails_closed_for_https_until_downloader_exists() {
    let root = output_root("apply-https");
    fs::create_dir_all(&root).unwrap();
    let catalog = root.join("team-catalog.json");
    write_catalog(&catalog);
    let home = root.join("home");

    let output = assert_failure_json(&run_skillrun(
        &[
            "team",
            "catalog",
            "install",
            "apply",
            catalog.to_str().unwrap(),
            "refund",
            "--json",
        ],
        &home,
    ));

    assert_eq!(output["schema_version"], "team.catalog.install_apply.v1");
    assert_eq!(output["ok"], false);
    assert_eq!(
        output["error"]["code"],
        "catalog.source_download_unsupported"
    );
}
