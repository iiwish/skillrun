# SkillRun v0.5.0 Work Graph: Language-agnostic Adapter Protocol

Version: v0.5.0
Status: Confirmed
Source spec: `.ai-platform/specs/v0.5-adapter-protocol/spec.md`
Last updated: 2026-05-14
Review: User requested review, commit and continuation on 2026-05-14; work graph accepted for packetized execution.

## Work Graph Summary

```text
T050 -> T051 -> T052 -> T053 -> T054 -> T055
```

## Epic E007: Adapter Protocol And Level 0 Command Adapter

Goal:
Define and prove a language-agnostic Adapter Protocol with Level 0 command adapter as the first protocol-native implementation slice.

### T050: Publish Adapter Protocol Contract

Status: Accepted
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

Test targets:
- Documentation consistency review.
- `cargo test --test business_examples`

Deliverables:
- Public `docs/adapter-protocol.md` contract.
- Updated architecture or index docs linking the protocol.
- Updated v0.5 planning/evidence references if needed.

Acceptance criteria:
- Adapter Protocol lifecycle, metadata phase, run phase, envelopes and capability levels are defined.
- Docs distinguish Adapter Protocol, Language Adapter and SDK.
- Docs do not imply sandboxing, dependency installation, shell execution or new blessed language support.
- Existing business examples still pass.

Definition of Done:
- `git diff --check` passes.
- `cargo test --test business_examples` passes.
- Delivery artifact validator reports no blocking errors for T050.

Validation commands:
- `git diff --check`
- `cargo test --test business_examples`

TDD plan:
- RED: Documentation-only task; use protocol consistency review instead of behavior RED.
- GREEN: Add protocol docs and links until acceptance criteria are met.
- REFACTOR: Remove duplicate or overbroad wording after validation.

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T050.yaml`

Evidence required:
- Summary, diff, validation results.

Acceptance:
- Accepted by user's 2026-05-14 review/commit/continue request after T050 evidence review passed.

### T051: Add Adapter Conformance Fixtures For Existing Adapters

Status: Accepted
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

Test targets:
- `tests/adapter_conformance.rs`
- Existing Python stable capsule behavior.
- Existing JS alpha capsule behavior.

Deliverables:
- A focused adapter conformance integration test suite.
- Fixtures or helper data needed to exercise existing Python and JS adapters.
- No user-visible CLI behavior changes.

Acceptance criteria:
- Python stable adapter path is covered by conformance tests.
- JS alpha adapter path is covered by conformance tests.
- Tests describe success envelope behavior, schema/metadata availability and stdout-not-result discipline where currently supported.
- Existing behavior is mapped, not refactored.

Definition of Done:
- `cargo test --test adapter_conformance` passes.
- `cargo test` passes.
- Delivery evidence captures RED/GREEN results and residual risk.

Validation commands:
- `cargo test --test adapter_conformance`
- `cargo test`

TDD plan:
- RED: Add adapter conformance tests and confirm they fail before required fixture/test support exists, or document why existing behavior already satisfies a specific assertion.
- GREEN: Add the smallest fixtures/helpers needed for the tests to pass without runtime behavior changes.
- REFACTOR: Keep helper code local to tests unless an existing public test helper pattern already exists.

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T051.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

Acceptance:
- Accepted by user's 2026-05-14 review/commit/continue request after T051 evidence review passed.

### T052: Support Static Schema And Command Runtime Manifest Generation

Status: Accepted
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

Test targets:
- `tests/manifest.rs`
- `tests/consumer_guards.rs`

Deliverables:
- Manifest generation recognizes `runtime.adapter = "command"` with explicit argv command.
- Manifest generation accepts static JSON Schema from `skillrun.config.json`.
- Consumer/readiness diagnostics can inspect command adapter executable requirements without importing source.
- No runtime execution support for command adapter yet; that remains T053.

Acceptance criteria:
- A command-adapter capsule can generate a Manifest with `runtime.adapter: command`, argv command, protocol version and static schemas.
- Command adapter configuration rejects shell-string commands.
- Consumer Mode checks do not import command action source for metadata.
- Missing command executable is diagnosed as a requirement/readiness issue, not installed.

Definition of Done:
- `cargo test --test manifest --test consumer_guards` passes.
- `cargo test` passes.
- Delivery evidence captures RED/GREEN results and residual risk.

Validation commands:
- `cargo test --test manifest --test consumer_guards`
- `cargo test`

TDD plan:
- RED: Add manifest/consumer guard tests for command adapter static schema and argv-only config.
- GREEN: Add minimal config/manifest/readiness support without runtime dispatch.
- REFACTOR: Keep schema parsing and command requirement logic localized to existing config/manifest/readiness boundaries.

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T052.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

Acceptance:
- Accepted by user's 2026-05-14 review/commit/continue request after T052 evidence review passed.

### T053: Implement Level 0 Command Adapter Runtime

Status: Accepted
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

Test targets:
- `tests/runtime.rs`
- `tests/errors.rs`
- `tests/adapter_conformance.rs`

Deliverables:
- Runtime dispatch supports `runtime.adapter = "command"`.
- Command adapter receives standard SkillRun IPC environment variables.
- Command adapter stdout/stderr are captured as logs, not result.
- Missing output, malformed output and artifact escape remain Core-validated failures.
- Missing executable maps to structured dependency/runtime failure without shell execution.

Acceptance criteria:
- Command adapter can execute an argv command that writes a valid SkillRun output envelope.
- Command adapter does not invoke a shell.
- Command adapter missing executable produces a structured dependency-style failure.
- Existing Python and JS adapter runtime tests remain green.

Definition of Done:
- `cargo test --test runtime --test errors --test adapter_conformance` passes.
- `cargo test` passes.
- Delivery evidence captures RED/GREEN results and residual risk.

Validation commands:
- `cargo test --test runtime --test errors --test adapter_conformance`
- `cargo test`

TDD plan:
- RED: Add runtime tests for command adapter success, stdout logging and missing output/protocol violation.
- GREEN: Add minimal command adapter process runner wired into existing adapter dispatch.
- REFACTOR: Keep common process behavior small and avoid broad Python/Node refactors unless needed.

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T053.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

Review:
- Implementation evidence recorded at `.ai-platform/evidence/T053/summary.md`.
- Accepted by user's 2026-05-14 review/commit/continue request after T053 evidence review passed.

### T054: Add Command Adapter Example Capsule

Status: Needs_Review
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

Test targets:
- `tests/business_examples.rs`

Deliverables:
- A small runnable `examples/command_hello` command-adapter capsule.
- Static input/output schemas in `skillrun.config.json`.
- Business example coverage for manifest, check, test, serve dry-run and pack.
- Docs that frame the example as Level 0 command adapter, not a new blessed language adapter.

Acceptance criteria:
- `command_hello` can generate a Manifest with `runtime.adapter: command`.
- `command_hello` can run through `skillrun test` and return a valid envelope.
- stdout/stderr are logs and not the result channel.
- The example does not require package installation, vendored dependencies, registry behavior or sandbox claims.

Definition of Done:
- `cargo test --test business_examples` passes.
- `cargo test` passes.
- Delivery evidence captures RED/GREEN results and residual risk.

Validation commands:
- `cargo test --test business_examples`
- `cargo test`

TDD plan:
- RED: Add a business example test for the missing `command_hello` capsule.
- GREEN: Add the smallest command adapter capsule and docs references needed for the test to pass.
- REFACTOR: Keep the example SDK-free and avoid runtime changes.

Packet path:
- `.ai-platform/specs/v0.5-adapter-protocol/packets/T054.yaml`

Evidence required:
- RED/GREEN test evidence, diff summary, residual risk.

Readiness:
- Dependencies satisfied by accepted T053 command adapter runtime.
- Execution packet generated at `.ai-platform/specs/v0.5-adapter-protocol/packets/T054.yaml`.

Review:
- Implementation evidence recorded at `.ai-platform/evidence/T054/summary.md`.
- Awaiting user acceptance before marking Accepted.

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

- Approval: accepted by user's 2026-05-14 review/commit/continue request.
- Reviewer notes: T050 is Ready after packet generation. Later tasks remain Draft until their dependencies are completed and packets are generated.
