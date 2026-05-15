# SkillRun v0.5.4 Work Graph: Core Stabilization Audit

Version: v0.5.4
Status: Ready_For_User_Review
Source analysis: `.ai-platform/specs/v0.5.4-core-stabilization-audit/analysis.md`
Public doc: `docs/v0.5.4-core-stabilization-audit.md`
Last updated: 2026-05-15

## Work Graph Summary

```text
T062 -> T063 -> T064 -> T065 -> T066
```

## Epic E010: Core Stabilization Before Desktop

Goal:
Make the whole `skillrun` project stable enough to be consumed by a separate Desktop project.

### T062: Stop Command Readiness From Executing Arbitrary Commands

Status: Ready_For_User_Review
Priority: P0
Depends on: v0.5.4 audit approval
Blocks: T063, T064, T065, T066
Parallel: No

Goal:
For `runtime.adapter="command"`, readiness must detect executable presence without executing `<command> --version`.

Allowed files:

- `src/readiness.rs`
- `tests/consumer_guards.rs`
- `tests/runtime.rs`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/analysis.md`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/tasks.md`

Acceptance criteria:

- `skillrun check` for command adapter does not execute the configured command.
- `skillrun switchboard enable` does not execute the configured command while checking readiness.
- Missing command executable still reports `dependency-error`.
- Python/Node dependency diagnostics remain unchanged.

Validation commands:

- `cargo test --test consumer_guards --test runtime --test registry`
- `cargo test`

### T063: Enforce Manifest JSON Schemas In Core Runtime

Status: Ready_For_User_Review
Priority: P0
Depends on: T062
Blocks: T064, T065, T066
Parallel: No

Goal:
Make Manifest input/output schemas real runtime contracts for every adapter, especially Level 0 command.

Allowed files:

- `Cargo.toml`
- `Cargo.lock`
- `src/schemas.rs`
- `src/runtime.rs`
- `src/errors.rs`
- `tests/runtime.rs`
- `tests/errors.rs`
- `tests/mcp_server.rs`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/analysis.md`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/tasks.md`

Acceptance criteria:

- Invalid input fails before adapter launch with structured `ValidationError`.
- Invalid successful adapter output fails as `ProtocolViolation`.
- Command adapter static schemas are enforced.
- Existing Python/JS adapter behavior remains compatible.
- MCP `tools/call` surfaces schema failures without killing the server.

Validation commands:

- `cargo test --test runtime --test errors --test mcp_server`
- `cargo test`

### T064: Make Registry And Switchboard List Robust Against Bad Entries

Status: Ready_For_User_Review
Priority: P1
Depends on: T063
Blocks: T065, T066
Parallel: No

Goal:
Inventory list commands should not fail entirely because one registered capsule has a malformed Manifest or unreadable metadata.

Allowed files:

- `src/registry.rs`
- `tests/registry.rs`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/analysis.md`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/tasks.md`

Acceptance criteria:

- Corrupt Manifest is represented as per-capsule readiness state.
- Missing path behavior from v0.5.3 remains intact.
- `switchboard enable` continues fail-closed.

Validation commands:

- `cargo test --test registry`
- `cargo test`

### T065: Freeze Consumer JSON Fixtures

Status: Ready_For_User_Review
Priority: P1
Depends on: T064
Blocks: T066
Parallel: No

Goal:
Add Desktop-facing JSON contract fixtures and document compatibility rules.

Allowed files:

- `tests/consumer_json_contracts.rs`
- `tests/fixtures/contracts/**`
- `docs/v0.5.2-consumer-json-surface.md`
- `docs/v0.5.3-capsule-registry-switchboard.md`
- `docs/v0.5.4-core-stabilization-audit.md`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/analysis.md`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/tasks.md`

Acceptance criteria:

- Fixtures cover inspect/check/doctor JSON states.
- Fixtures cover registry/switchboard JSON states.
- Contract compatibility rule is documented.

Validation commands:

- `cargo test --test consumer_json_contracts`
- `cargo test`

### T066: Document Version Semantics And Desktop Boundary

Status: Ready_For_User_Review
Priority: P1
Depends on: T065
Blocks: Desktop project start
Parallel: No

Goal:
Make version semantics and Desktop/Core dependency boundaries explicit.

Allowed files:

- `README.md`
- `README.zh-CN.md`
- `docs/release-policy.md`
- `docs/v0.5.4-core-stabilization-audit.md`
- `RELEASE_NOTES.md`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/analysis.md`
- `.ai-platform/specs/v0.5.4-core-stabilization-audit/tasks.md`

Acceptance criteria:

- Binary/artifact version, release line, Manifest IR version, and Adapter Protocol version are distinct.
- Desktop is documented as a separate project consuming Core contracts.
- Release validation includes `cargo clippy --all-targets -- -D warnings`.

Validation commands:

- `cargo fmt --check`
- `git diff --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
