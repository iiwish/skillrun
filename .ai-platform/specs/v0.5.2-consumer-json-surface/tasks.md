# SkillRun v0.5.2 Work Graph: Consumer JSON Surface

Version: v0.5.2
Status: Confirmed
Source spec: `.ai-platform/specs/v0.5.2-consumer-json-surface/spec.md`
Last updated: 2026-05-15
Review: User requested review, local commit, and continuation on 2026-05-15; work graph approved for sequenced execution.

## Work Graph Summary

```text
T056 -> T057 -> T058
```

## Epic E008: Consumer JSON Surface

Goal:
Provide stable machine-readable JSON output for Consumer Mode report commands without changing default human output or runtime behavior.

### T056: Add `inspect --json`

Status: Completed
Priority: P0
Depends on: v0.5.2 spec approval
Blocks: T057, T058
Story / Requirement: US-001, US-003, FR-001, FR-002, FR-003, FR-007, FR-008, NFR-002, NFR-003, NFR-005
Parallel: No
Conflicts with: T057, T058

Goal:
Add additive JSON output mode for `skillrun inspect` covering runnable, invalid-runnable, and instruction-only states without importing action source.

Allowed files:
- `src/cli.rs`
- `src/inspect.rs`
- `tests/inspect.rs`

Test targets:
- `tests/inspect.rs`
- Existing human `inspect` tests.

Deliverables:
- `skillrun inspect --json --cwd path/to/capsule`
- Structured inspect JSON report.
- Tests that parse JSON output.
- Tests that confirm JSON inspect does not import modified action source.

Acceptance criteria:
- Runnable capsule JSON includes `command`, `cwd`, `status`, `manifest`, `skill`, `sources`, `schemas`, `runtime`, `permissions`, `examples`, `preflight`, and `tool`.
- Stale Manifest JSON returns `status="invalid-runnable"` and includes `reason`.
- Instruction-only JSON returns `status="instruction-only"` and missing file details.
- Default human inspect output remains compatible with existing tests.

Definition of Done:
- RED/GREEN evidence exists for JSON inspect tests.
- `cargo test --test inspect` passes.
- `cargo test` passes or broader validation failure is documented as unrelated.

Validation commands:
- `cargo test --test inspect`
- `cargo test`

TDD plan:
- RED: Add failing JSON inspect contract tests for runnable, stale, and instruction-only states.
- GREEN: Add CLI `--json` parsing and structured inspect report serialization.
- REFACTOR: Keep human rendering stable and avoid broad CLI parser refactor.

Packet path:
- `.ai-platform/specs/v0.5.2-consumer-json-surface/packets/T056.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risk.

Evidence:
- Changed files: `src/cli.rs`, `src/inspect.rs`, `tests/inspect.rs`.
- RED: `cargo test --test inspect` failed because `--json` was rejected by `parse_inspect`.
- GREEN: `cargo test --test inspect` passed, 8 passed.
- Full validation: `cargo test` passed.
- Residual risk: JSON invalid-runnable currently reports structured Manifest presence and reason, but does not attempt to parse partial stale Manifest details.

### T057: Add `check --json` And `doctor --json`

Status: Completed
Priority: P0
Depends on: T056
Blocks: T058
Story / Requirement: US-002, US-003, US-004, FR-004, FR-005, FR-006, FR-007, FR-008, FR-010, NFR-001, NFR-002, NFR-003, NFR-004
Parallel: No
Conflicts with: T056, T058

Goal:
Expose readiness data as JSON for `check` and `doctor`, sharing one readiness schema and preserving existing exit code behavior.

Allowed files:
- `src/cli.rs`
- `src/check.rs`
- `src/doctor.rs`
- `src/readiness.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`
- `tests/cli.rs`

Test targets:
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`
- `tests/cli.rs`

Deliverables:
- `skillrun check --json --cwd path/to/capsule`
- `skillrun doctor --json --cwd path/to/capsule`
- Shared readiness JSON report schema.
- Tests for ok, stale/missing, dependency, and instruction-only states.

Acceptance criteria:
- `check --json` emits one valid JSON object with `command="check"` and `ok`.
- `doctor --json` emits the same readiness shape with `command="doctor"`.
- Exit code matches existing check/doctor semantics.
- Existing human check/doctor tests remain compatible.
- JSON mode does not import action source.

Definition of Done:
- RED/GREEN evidence exists for readiness JSON tests.
- Focused readiness-related tests pass.
- `cargo test` passes or broader validation failure is documented as unrelated.

Validation commands:
- `cargo test --test consumer_guards --test instruction_only --test cli`
- `cargo test`

TDD plan:
- RED: Add failing JSON readiness contract tests for check and doctor.
- GREEN: Add serializable readiness report and CLI `--json` routing.
- REFACTOR: Reuse readiness data instead of duplicating check/doctor report logic.

Packet path:
- `.ai-platform/specs/v0.5.2-consumer-json-surface/packets/T057.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risk.

Evidence:
- Changed files: `src/cli.rs`, `src/check.rs`, `src/doctor.rs`, `src/readiness.rs`, `tests/consumer_guards.rs`, `tests/instruction_only.rs`.
- RED: `cargo test --test consumer_guards --test instruction_only --test cli` failed because `--json` was rejected by `check` and `doctor`.
- GREEN: focused readiness tests passed.
- Full validation: `cargo test` passed.
- Residual risk: JSON mode intentionally keeps parser/filesystem errors on stderr instead of introducing a global CLI error envelope.

### T058: Finalize v0.5.2 Docs And Release Validation

Status: Completed
Priority: P1
Depends on: T056, T057
Blocks: None
Story / Requirement: US-003, FR-009, FR-010, NFR-002
Parallel: No
Conflicts with: T056, T057

Goal:
Update public docs and release notes for the implemented v0.5.2 JSON surface, then run final validation.

Allowed files:
- `README.md`
- `README.zh-CN.md`
- `docs/README.md`
- `docs/v0.5.2-consumer-json-surface.md`
- `RELEASE_NOTES.md`
- `.ai-platform/specs/v0.5.2-consumer-json-surface/analysis.md`
- `.ai-platform/specs/v0.5.2-consumer-json-surface/tasks.md`

Test targets:
- Documentation consistency.
- Full cargo test suite.

Deliverables:
- README references to `inspect/check/doctor --json`.
- Release notes entry for v0.5.2.
- Final analysis or task status updates.

Acceptance criteria:
- Docs clearly state `run` and `test` are not wrapped by v0.5.2.
- Docs do not imply registry, router, daemon, UI, sandbox, or signed package support.
- `cargo test` passes.
- `git diff --check` passes.

Definition of Done:
- Documentation reflects implemented behavior.
- Full validation evidence exists.
- No scope creep into v0.5.3+ features.

Validation commands:
- `git diff --check`
- `cargo test`

TDD plan:
- RED: Documentation-only finalization; use consistency review and release validation instead of behavior RED.
- GREEN: Update docs and release notes to match implemented behavior.
- REFACTOR: Remove redundant or overbroad wording after validation.

Packet path:
- `.ai-platform/specs/v0.5.2-consumer-json-surface/packets/T058.yaml`

Evidence required:
- Changed files.
- Validation results.
- Diff summary.
- Residual risk.

Evidence:
- Changed files: `README.md`, `README.zh-CN.md`, `docs/v0.5.2-consumer-json-surface.md`, `RELEASE_NOTES.md`, `.ai-platform/specs/v0.5.2-consumer-json-surface/analysis.md`, `.ai-platform/specs/v0.5.2-consumer-json-surface/tasks.md`.
- Validation: `git diff --check` passed; `cargo test` passed.
- Residual risk: release notes mark v0.5.2 as `Ready_For_Release_Decision`; no tag, push, or package publication has been performed.
