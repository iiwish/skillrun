use flate2::read::GzDecoder;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Archive;

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

fn init_capsule(label: &str, name: &str) -> (PathBuf, PathBuf) {
    let output_root = temp_dir(label);
    let output_arg = output_root.to_string_lossy().to_string();

    let init = run_skillrun(&["init", name, "--python", "--output", &output_arg]);
    assert!(
        init.status.success(),
        "init should succeed\nstderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let capsule = output_root.join(name);
    (output_root, capsule)
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

fn generated_capsule(label: &str, name: &str) -> (PathBuf, PathBuf) {
    let (output_root, capsule) = init_capsule(label, name);
    write_manifest(&capsule);
    (output_root, capsule)
}

fn archive_entries(path: &Path) -> Vec<String> {
    let file = fs::File::open(path).expect("archive should be readable");
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let mut entries = archive
        .entries()
        .expect("archive entries should be readable")
        .map(|entry| {
            let entry = entry.expect("archive entry should be readable");
            entry
                .path()
                .expect("archive path should be readable")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn unpack_archive(path: &Path, target: &Path) {
    let file = fs::File::open(path).expect("archive should be readable");
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(target).expect("archive should unpack");
}

#[test]
fn pack_creates_skr_with_sources_manifest_examples_and_no_run_history() {
    let (output_root, capsule) = generated_capsule("pack-content", "refund");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let run = run_skillrun(&["test", "--cwd", &cwd_arg]);
    assert!(
        run.status.success(),
        "test should create run history\nstderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert!(
        pack.status.success(),
        "pack should succeed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&pack.stdout),
        String::from_utf8_lossy(&pack.stderr)
    );
    let stdout = String::from_utf8(pack.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("created"));
    assert!(stdout.contains("refund-0.2.0.skr"));
    assert!(stdout.contains("does not vendor dependencies"));

    let archive_path = capsule.join("dist").join("refund-0.2.0.skr");
    assert!(archive_path.is_file(), "archive should exist");

    let entries = archive_entries(&archive_path);
    for required in [
        ".skillrun/manifest.generated.yaml",
        "SKILL.md",
        "action.py",
        "examples/default.input.json",
        "skillrun.config.json",
    ] {
        assert!(
            entries.iter().any(|entry| entry == required),
            "archive should include {required:?}; got {entries:?}"
        );
    }
    assert!(
        entries
            .iter()
            .all(|entry| !entry.starts_with(".skillrun/runs/")),
        "archive must exclude run history: {entries:?}"
    );
    assert!(
        entries.iter().all(|entry| !entry.starts_with("dist/")),
        "archive must exclude generated dist output: {entries:?}"
    );

    let unpacked_capsule = output_root.join("unpacked-refund");
    fs::create_dir_all(&unpacked_capsule).expect("unpack target should be created");
    unpack_archive(&archive_path, &unpacked_capsule);
    let inspect_cwd = unpacked_capsule.to_string_lossy().to_string();
    let inspect = run_skillrun(&["inspect", "--cwd", &inspect_cwd]);
    assert!(
        inspect.status.success(),
        "unpacked capsule should inspect\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&inspect.stdout),
        String::from_utf8_lossy(&inspect.stderr)
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn pack_uses_capsule_name_for_archive_filename() {
    let (output_root, capsule) = generated_capsule("pack-name", "triage");
    let cwd_arg = capsule.to_string_lossy().to_string();

    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert!(
        pack.status.success(),
        "pack should succeed\nstderr: {}",
        String::from_utf8_lossy(&pack.stderr)
    );

    assert!(capsule.join("dist").join("triage-0.2.0.skr").is_file());

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn pack_refuses_stale_manifest_before_archive_creation() {
    let (output_root, capsule) = generated_capsule("pack-stale", "refund");
    let action_path = capsule.join("action.py");
    let mut action = fs::read_to_string(&action_path).expect("action should be readable");
    action.push_str("\n# stale before pack\n");
    fs::write(&action_path, action).expect("action should be writable");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert!(
        !pack.status.success(),
        "pack should fail when Manifest is stale"
    );
    let stderr = String::from_utf8(pack.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("stale Manifest"));
    assert!(stderr.contains("action.py"));
    assert!(!stderr.contains("command not implemented yet"));
    assert!(
        !capsule.join("dist").join("refund-0.2.0.skr").exists(),
        "stale pack must not create an archive"
    );

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn pack_rejects_manifest_name_that_would_escape_dist() {
    let (output_root, capsule) = generated_capsule("pack-bad-name", "refund");
    let manifest_path = capsule.join(".skillrun").join("manifest.generated.yaml");
    let manifest = fs::read_to_string(&manifest_path).expect("manifest should be readable");
    let manifest = manifest.replace("name: refund", "name: ../escape");
    fs::write(&manifest_path, manifest).expect("manifest should be writable");

    let cwd_arg = capsule.to_string_lossy().to_string();
    let pack = run_skillrun(&["pack", "--cwd", &cwd_arg]);
    assert!(
        !pack.status.success(),
        "pack should fail on invalid archive name"
    );
    let stderr = String::from_utf8(pack.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("invalid package name from Manifest"));
    assert!(
        !capsule.join("escape-0.2.0.skr").exists(),
        "invalid Manifest name must not escape dist"
    );
    assert!(
        !capsule.join("dist").join("escape-0.2.0.skr").exists(),
        "invalid Manifest name must not create a package"
    );

    fs::remove_dir_all(output_root).ok();
}
