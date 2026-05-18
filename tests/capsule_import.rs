use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use std::fs;
use std::io::Write;
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

fn archive_name(stem: &str) -> String {
    format!("{stem}-{}.skr", env!("CARGO_PKG_VERSION"))
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

fn generated_package(label: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();
    let skillrun_home = output_root.join("author-home");

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

    let test = run_skillrun(&["test", "--cwd", &cwd_arg], &skillrun_home);
    assert!(
        test.status.success(),
        "test should create run history before pack\nstderr: {}",
        String::from_utf8_lossy(&test.stderr)
    );

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg], &skillrun_home);
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

#[test]
fn import_skr_unpacks_valid_package_and_registers_disabled_capsule() {
    let (output_root, archive_path) = generated_package("import-success");
    let skillrun_home = output_root.join("consumer-home");
    let package_arg = archive_path.to_string_lossy().to_string();

    let imported = assert_success_json(&run_skillrun(
        &["import", &package_arg, "--json"],
        &skillrun_home,
    ));

    assert_eq!(imported["command"], "import");
    assert_eq!(imported["schema_version"], "import.v1");
    assert_eq!(imported["ok"], true);
    assert_eq!(imported["capsule"]["id"], "refund");
    assert_eq!(imported["capsule"]["source_type"], "imported_skr");
    assert_eq!(imported["capsule"]["enabled"], false);
    assert!(imported["registry_path"]
        .as_str()
        .expect("registry path should be present")
        .contains("registry.json"));
    assert_eq!(imported["warnings"].as_array().unwrap().len(), 2);

    let imported_path = PathBuf::from(
        imported["capsule"]["path"]
            .as_str()
            .expect("imported capsule path should be present"),
    );
    assert!(imported_path.join("SKILL.md").is_file());
    assert!(imported_path.join("action.py").is_file());
    assert!(imported_path
        .join(".skillrun")
        .join("manifest.generated.yaml")
        .is_file());
    assert!(
        !imported_path.join(".skillrun").join("runs").exists(),
        "import must not recreate author run history"
    );
    assert!(
        !imported_path.join("dist").exists(),
        "import must not recreate author dist output"
    );

    let inventory = assert_success_json(&run_skillrun(
        &["consumer", "inventory", "--json"],
        &skillrun_home,
    ));
    assert_eq!(inventory["capsules"].as_array().unwrap().len(), 1);
    assert_eq!(inventory["capsules"][0]["id"], "refund");
    assert_eq!(inventory["capsules"][0]["source_type"], "imported_skr");
    assert_eq!(inventory["capsules"][0]["enabled"], false);
    assert_eq!(inventory["capsules"][0]["readiness"]["status"], "ok");

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn import_rejects_duplicate_registry_ids_without_overwriting_existing_capsule() {
    let (output_root, archive_path) = generated_package("import-duplicate");
    let skillrun_home = output_root.join("consumer-home");
    let package_arg = archive_path.to_string_lossy().to_string();

    let first = run_skillrun(&["import", &package_arg, "--json"], &skillrun_home);
    assert!(first.status.success());
    let imported = serde_json::from_slice::<Value>(&first.stdout).unwrap();
    let imported_path = PathBuf::from(imported["capsule"]["path"].as_str().unwrap());
    let marker = imported_path.join("KEEP.txt");
    fs::write(&marker, "do not overwrite").expect("marker should be writable");

    let duplicate = run_skillrun(&["import", &package_arg, "--json"], &skillrun_home);
    assert!(
        !duplicate.status.success(),
        "duplicate import should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&duplicate.stdout),
        String::from_utf8_lossy(&duplicate.stderr)
    );
    let stderr = String::from_utf8(duplicate.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("registry id already exists"));
    assert_eq!(
        fs::read_to_string(&marker).expect("marker should survive failed duplicate import"),
        "do not overwrite"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn import_rejects_archive_entries_that_escape_target_directory() {
    let output_root = temp_dir("import-path-traversal");
    let skillrun_home = output_root.join("consumer-home");
    let package_path = output_root.join("evil.skr");
    fs::create_dir_all(&output_root).expect("output root should be created");

    write_malicious_package(&package_path);

    let package_arg = package_path.to_string_lossy().to_string();
    let imported = run_skillrun(&["import", &package_arg, "--json"], &skillrun_home);
    assert!(
        !imported.status.success(),
        "path traversal package should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&imported.stdout),
        String::from_utf8_lossy(&imported.stderr)
    );
    let stderr = String::from_utf8(imported.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("package path escapes import target"));
    assert!(
        !output_root.join("escape.txt").exists(),
        "import must not write escaped archive entries"
    );

    fs::remove_dir_all(output_root).ok();
}

fn write_malicious_package(path: &Path) {
    let file = fs::File::create(path).expect("malicious package should be created");
    let mut encoder = GzEncoder::new(file, Compression::default());
    let content = b"escape";
    let mut header = [0u8; 512];
    write_bytes(&mut header[0..100], b"../escape.txt");
    write_octal(&mut header[100..108], 0o644);
    write_octal(&mut header[108..116], 0);
    write_octal(&mut header[116..124], 0);
    write_octal(&mut header[124..136], content.len() as u64);
    write_octal(&mut header[136..148], 0);
    for byte in &mut header[148..156] {
        *byte = b' ';
    }
    header[156] = b'0';
    write_bytes(&mut header[257..263], b"ustar\0");
    write_bytes(&mut header[263..265], b"00");
    let checksum = header.iter().map(|byte| *byte as u32).sum::<u32>();
    write_checksum(&mut header[148..156], checksum);

    encoder.write_all(&header).expect("header should write");
    encoder.write_all(content).expect("content should write");
    let padding = 512 - content.len();
    encoder
        .write_all(&vec![0u8; padding])
        .expect("padding should write");
    encoder
        .write_all(&[0u8; 1024])
        .expect("trailer should write");
    encoder.finish().expect("gzip should finish");
}

fn write_bytes(target: &mut [u8], value: &[u8]) {
    target[..value.len()].copy_from_slice(value);
}

fn write_octal(target: &mut [u8], value: u64) {
    let text = format!("{value:0width$o}\0", width = target.len() - 1);
    write_bytes(target, text.as_bytes());
}

fn write_checksum(target: &mut [u8], value: u32) {
    let text = format!("{value:06o}\0 ",);
    write_bytes(target, text.as_bytes());
}
