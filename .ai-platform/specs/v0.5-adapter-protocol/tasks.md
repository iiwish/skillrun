# SkillRun v0.5.0 Work Graph: Language-agnostic Adapter Protocol

Version: v0.5.0
Status: Ready_For_User_Review
Source spec: `.ai-platform/specs/v0.5-adapter-protocol/spec.md`
Last updated: 2026-05-14
Review: Prepared after v0.5 spec rereview passed; implementation requires user acceptance.

## Work Graph Summary

```text
T050 -> T051 -> T052 -> T053 -> T054 -> T055
```

## Epic E007: Adapter Protocol And Level 0 Command Adapter

Goal:
Define and prove a language-agnostic Adapter Protocol with Level 0 command adapter as the first protocol-native implementation slice.

### T050: Publish Adapter Protocol Contract

Status: Draft
Priority: P0
Depends on: v0.4.2
Blocks: T051, T052
Story / Requirement: FR-050-001, FR-050-002, FR-050-003, FR-050-004, NFR-050-003
Parallel: No
Conflicts with: T051

Goal:
Create the public Adapter Protocol contract and update architecture docs.

Allowed files:
- `docs/adapter-protocol.md`
- `docs/v0.5-adapter-protocol.md`
- `docs/README.md`
- `docs/ssot.md`
- `README.md`
- `README.zh-CN.md`

Validation commands:
- `git diff --check`
- `cargo test --test business_examples`

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T050.yaml`

Evidence required:
- Summary, diff, validation results.

### T051: Add Adapter Conformance Fixtures For Existing Adapters

Status: Draft
Priority: P0
Depends on: T050
Blocks: T052, T053
Story / Requirement: FR-050-006, FR-050-007, FR-050-009
Parallel: No
Conflicts with: T053

Goal:
Add conformance tests that map Python stable and JS alpha behavior to the protocol without changing user-visible behavior.

Allowed files:
- `tests/adapter_conformance.rs`
- `tests/fixtures/**`
- `examples/refund/**` only if fixture input is required
- `templates/**` only if test fixture reuse requires it

Validation commands:
- `cargo test --test adapter_conformance`
- `cargo test`

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T051.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

### T052: Support Static Schema And Command Runtime Manifest Generation

Status: Draft
Priority: P0
Depends on: T051
Blocks: T053
Story / Requirement: FR-050-005, FR-050-008, FR-050-010, FR-050-012
Parallel: No
Conflicts with: T053

Goal:
Teach manifest generation and readiness logic to understand Level 0 command adapter config with explicit argv command and static JSON schemas.

Allowed files:
- `src/config.rs`
- `src/manifest.rs`
- `src/readiness.rs`
- `src/check.rs`
- `src/doctor.rs`
- `tests/manifest.rs`
- `tests/consumer_guards.rs`
- `tests/fixtures/**`

Validation commands:
- `cargo test --test manifest --test consumer_guards`
- `cargo test`

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T052.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

### T053: Implement Level 0 Command Adapter Runtime

Status: Draft
Priority: P0
Depends on: T052
Blocks: T054, T055
Story / Requirement: FR-050-003, FR-050-010, FR-050-011, FR-050-012
Parallel: No
Conflicts with: T052, T054

Goal:
Run explicit command adapter argv through standard SkillRun IPC, envelope validation, artifact checks, stdout/stderr logging and structured dependency/protocol errors.

Allowed files:
- `src/adapters/mod.rs`
- `src/adapters/command.rs`
- `src/runtime.rs`
- `src/errors.rs`
- `tests/runtime.rs`
- `tests/errors.rs`
- `tests/adapter_conformance.rs`
- `tests/fixtures/**`

Validation commands:
- `cargo test --test runtime --test errors --test adapter_conformance`
- `cargo test`

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T053.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

### T054: Add Command Adapter Example Capsule

Status: Draft
Priority: P1
Depends on: T053
Blocks: T055
Story / Requirement: US-050-004, FR-050-010, FR-050-011
Parallel: No
Conflicts with: T053

Goal:
Add one small command-adapter example that proves Level 0 without implying a new blessed language adapter.

Allowed files:
- `examples/command_hello/**`
- `docs/business-examples.md`
- `docs/v0.5-adapter-protocol.md`
- `tests/business_examples.rs`

Validation commands:
- `cargo test --test business_examples`
- `cargo test`

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T054.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

### T055: Prepare v0.5.0 Release Readiness

Status: Draft
Priority: P1
Depends on: T054
Blocks: release decision
Story / Requirement: release readiness
Parallel: No
Conflicts with: all v0.5 release docs

Goal:
Update release notes, README status, version metadata and release report for v0.5.0.

Allowed files:
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `README.zh-CN.md`
- `RELEASE_NOTES.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.5-adapter-protocol/tasks.md`

Validation commands:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T055.yaml`

Evidence required:
- Release validation summary, diff, residual risk.

## User Review Gate

- Approval: pending.
- Reviewer notes: Implementation must not start until this work graph is accepted and task packets are generated for the selected Ready task.
