# SkillRun v0.5.3 Work Graph: Capsule Registry + Switchboard

Version: v0.5.3
Status: Ready_For_User_Review
Source spec: `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/spec.md`
Last updated: 2026-05-15
Review: Requires user approval before task status can move to `Ready`.

## Work Graph Summary

```text
T059 -> T060 -> T061
```

## Epic E009: Capsule Registry + Switchboard

Goal:
Create a local inventory and exposure-intent layer for future Router/Desktop consumers.

### T059: Add Local Capsule Registry

Status: Draft
Priority: P0
Depends on: v0.5.3 spec approval
Blocks: T060, T061
Parallel: No
Conflicts with: T060, T061

Goal:
Implement registry storage and `skillrun registry add/list/inspect/remove` with JSON output.

Allowed files:
- `src/cli.rs`
- `src/registry.rs`
- `src/main.rs`
- `tests/registry.rs`

Acceptance criteria:
- Empty registry is valid.
- `SKILLRUN_HOME` controls registry location.
- `registry add --cwd <capsule>` creates a disabled local_path entry.
- `registry list --json` emits parseable JSON.
- `registry inspect <id> --json` emits parseable JSON and includes readiness summary.
- `registry remove <id>` removes state without deleting capsule files.
- Duplicate ids are rejected.

Validation commands:
- `cargo test --test registry`
- `cargo test`

Packet path:
- `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/packets/T059.yaml`

### T060: Add Switchboard Enable Disable Gates

Status: Draft
Priority: P0
Depends on: T059
Blocks: T061
Parallel: No
Conflicts with: T059, T061

Goal:
Implement `skillrun switchboard list/enable/disable` over registry state with fail-closed enable behavior.

Allowed files:
- `src/cli.rs`
- `src/registry.rs`
- `src/switchboard.rs`
- `src/main.rs`
- `tests/registry.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`

Acceptance criteria:
- `switchboard list --json` emits registered capsules and enabled state.
- `switchboard enable <id>` succeeds only when readiness is ok.
- stale Manifest cannot be enabled.
- instruction-only capsule cannot be enabled.
- dependency-error capsule cannot be enabled.
- `switchboard disable <id>` succeeds without executing action source.

Validation commands:
- `cargo test --test registry --test consumer_guards --test instruction_only`
- `cargo test`

Packet path:
- `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/packets/T060.yaml`

### T061: Finalize v0.5.3 Docs And Release Evidence

Status: Draft
Priority: P1
Depends on: T059, T060
Blocks: None
Parallel: No
Conflicts with: T059, T060

Goal:
Update README, docs, and release notes for the implemented v0.5.3 local registry/switchboard surface.

Allowed files:
- `README.md`
- `README.zh-CN.md`
- `docs/README.md`
- `docs/v0.5.3-capsule-registry-switchboard.md`
- `RELEASE_NOTES.md`
- `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/analysis.md`
- `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/tasks.md`

Acceptance criteria:
- Docs state registry is local inventory, not marketplace or trust.
- Docs state enabled means future exposure intent, not sandbox/trust.
- Docs state v0.5.3 does not include Router or MCP client mount profiles.
- `cargo test` and `git diff --check` pass.

Validation commands:
- `git diff --check`
- `cargo test`

Packet path:
- `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/packets/T061.yaml`
