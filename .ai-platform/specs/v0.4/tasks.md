# SkillRun v0.4 Work Graph And Tasks

Version: v0.4
Status: Ready_For_User_Review
Source spec: `.ai-platform/specs/v0.4/spec.md`
Source plan: `.ai-platform/specs/v0.4/plan.md`
Last updated: 2026-05-13
Review: Drafted after assistant review; waiting for user confirmation before any task moves to Ready.

## Work Graph Summary

v0.4 work graph starts after accepted v0.3 tasks `T019` through `T028`. New task IDs begin at `T029`.

```text
T029 -> T030 -> T031 -> T032 -> T033 -> T034 -> T036
                  |              ^
                  v              |
                 T035 ------------+
```

## Epic E004: Portable Consumer Checks

Goal:
Make distributed Skill Capsules inspectable and dependency-checkable even when the current host cannot run them.

### Story S004-1: Establish Readiness Contract

User value:
Maintainers get a stable contract for dependency-aware Consumer Mode before implementation changes runtime behavior.

### T029: Add v0.4 Contract Tests And DependencyError Skeleton

Status: Accepted
Priority: P0
Depends on: T028
Blocks: T030
Story / Requirement: FR-001, FR-005, NFR-004
Parallel: No
Conflicts with: T030, T033

Goal:
Introduce failing tests and minimal error-code contract for v0.4 dependency readiness behavior.

Allowed files:
- `src/errors.rs`
- `tests/errors.rs`
- `tests/cli.rs`
- `tests/consumer_guards.rs`
- `.ai-platform/specs/v0.4/tasks.md`

Test targets:
- `cargo test --test errors --test cli --test consumer_guards`

Deliverables:
- `DependencyError` accepted by error envelope validation.
- Tests that define expected CLI/check behavior before implementation.
- No runtime dependency probing yet.

Acceptance criteria:
- `DependencyError` is a recognized structured error code.
- Existing error envelope behavior remains unchanged.
- Tests clearly separate dependency failure from `RuntimeError`.

Definition of Done:
- Target tests are added.
- Any expected RED failures are recorded before implementation proceeds.

Validation commands:
- `cargo test --test errors --test cli --test consumer_guards`
- `cargo test`

TDD plan:
- RED: Add failing dependency readiness/error tests.
- GREEN: Add minimal `DependencyError` support.
- REFACTOR: None unless duplicate error helpers appear.

Packet path:
- `.ai-platform/specs/v0.4/packets/T029.yaml`

Evidence required:
- `.ai-platform/evidence/T029/summary.md`
- `.ai-platform/evidence/T029/test-results.md`
- `.ai-platform/evidence/T029/diff.patch`

### T030: Add Manifest Runtime Requirements Contract

Status: Accepted
Priority: P0
Depends on: T029
Blocks: T031
Story / Requirement: FR-002, FR-007, NFR-003
Parallel: No
Conflicts with: T029, T031

Goal:
Generate or resolve minimal runtime requirements for Python stable and JS Alpha capsules.

Allowed files:
- `src/manifest.rs`
- `src/config.rs`
- `src/adapters/mod.rs`
- `tests/manifest.rs`
- `tests/pack.rs`
- `docs/ssot.md`

Test targets:
- `cargo test --test manifest --test pack`

Deliverables:
- Manifest includes runtime requirement data or a documented adapter-default fallback.
- Python requirements include Python version and Pydantic v2.
- Node requirements include Node version and no package-manager dependency.
- `.skr` includes the generated Manifest requirements.

Acceptance criteria:
- Python and JS alpha Manifest tests pass.
- Existing v0.3 Manifest fields remain compatible.
- Legacy Manifest behavior is documented for `check`.

Definition of Done:
- Requirements shape is tested.
- Pack tests prove requirements travel with `.skr`.

Validation commands:
- `cargo test --test manifest --test pack`
- `cargo test`

TDD plan:
- RED: Add Manifest assertions for runtime requirements.
- GREEN: Generate adapter defaults.
- REFACTOR: Keep requirements shape minimal.

Packet path:
- `.ai-platform/specs/v0.4/packets/T030.yaml`

Evidence required:
- `.ai-platform/evidence/T030/summary.md`
- `.ai-platform/evidence/T030/test-results.md`
- `.ai-platform/evidence/T030/diff.patch`

### Story S004-2: Diagnose Host Readiness Without Source Import

User value:
Consumers can understand whether a capsule is runnable on their machine without executing untrusted action source.

### T031: Implement Readiness Engine And `skillrun check`

Status: Accepted
Priority: P0
Depends on: T030
Blocks: T032, T035
Story / Requirement: FR-001, FR-008, NFR-003
Parallel: No
Conflicts with: T030, T032, T035

Goal:
Add a Consumer Mode readiness engine and expose it through `skillrun check --cwd <capsule>`.

Allowed files:
- `src/main.rs`
- `src/cli.rs`
- `src/doctor.rs`
- `src/consumer.rs`
- `src/check.rs`
- `src/readiness.rs`
- `tests/cli.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`

Test targets:
- `cargo test --test cli --test consumer_guards --test instruction_only`

Deliverables:
- `skillrun check --cwd <capsule>` command.
- Static checks for SKILL.md, action entrypoint, Manifest presence/freshness, examples and requirements.
- `doctor` alignment or reuse of readiness findings.

Acceptance criteria:
- `check` does not import `action.py` or `action.mjs`.
- `check` does not create run records.
- instruction-only and unsupported TypeScript cases remain clear.
- CLI help lists `check` separately from `doctor`.

Definition of Done:
- Consumer no-import tests pass.
- `check` has deterministic success/failure status.

Validation commands:
- `cargo test --test cli --test consumer_guards --test instruction_only`
- `cargo test`

TDD plan:
- RED: Add failing tests for `check`.
- GREEN: Implement readiness engine and command parser.
- REFACTOR: Share rendering with `doctor` only after behavior is green.

Packet path:
- `.ai-platform/specs/v0.4/packets/T031.yaml`

Evidence required:
- `.ai-platform/evidence/T031/summary.md`
- `.ai-platform/evidence/T031/test-results.md`
- `.ai-platform/evidence/T031/diff.patch`

### T032: Add Python And Node Runtime Discovery

Status: Accepted
Priority: P0
Depends on: T031
Blocks: T033
Story / Requirement: FR-003, FR-004, NFR-004
Parallel: No
Conflicts with: T031, T033

Goal:
Probe host Python/Node executables and required adapter packages without importing action source.

Allowed files:
- `src/adapters/python.rs`
- `src/adapters/node.rs`
- `src/adapters/mod.rs`
- `src/readiness.rs`
- `tests/consumer_guards.rs`
- `tests/manifest.rs`

Test targets:
- `cargo test --test consumer_guards --test manifest`

Deliverables:
- Python executable version detection.
- Node executable version detection.
- Pydantic v2 package detection.
- Clear readiness findings for missing executable, unsupported version and missing package.

Acceptance criteria:
- Missing executable is not reported as raw `program not found`.
- Detected and required versions are present in check output.
- Pydantic check imports only Pydantic, not action source.
- JS Alpha does not require npm package-manager checks.

Definition of Done:
- Hostile environment tests pass or have deterministic simulation.
- Existing valid Python/JS capsules still check successfully.

Validation commands:
- `cargo test --test consumer_guards --test manifest`
- `cargo test`

TDD plan:
- RED: Add failing missing-runtime and missing-package tests.
- GREEN: Implement probes.
- REFACTOR: Keep probe APIs adapter-neutral.

Packet path:
- `.ai-platform/specs/v0.4/packets/T032.yaml`

Evidence required:
- `.ai-platform/evidence/T032/summary.md`
- `.ai-platform/evidence/T032/test-results.md`
- `.ai-platform/evidence/T032/diff.patch`

### Story S004-3: Make Runtime Failures Agent-safe

User value:
Agents receive recoverable structured dependency failures instead of brittle internal runtime errors.

### T033: Convert Runtime Dependency Failures To DependencyError

Status: Accepted
Priority: P0
Depends on: T032
Blocks: T034, T036
Story / Requirement: FR-005, NFR-004
Parallel: No
Conflicts with: T029, T032, T034

Goal:
Ensure `skillrun test` and `skillrun run` return structured `DependencyError` envelopes for dependency failures.

Allowed files:
- `src/runtime.rs`
- `src/errors.rs`
- `src/adapters/python.rs`
- `src/adapters/node.rs`
- `src/readiness.rs`
- `tests/runtime.rs`
- `tests/errors.rs`

Test targets:
- `cargo test --test runtime --test errors`

Deliverables:
- Runtime readiness precheck or equivalent dependency failure mapping.
- `DependencyError` envelope with clear message, recoverable flag and optional `llm_hint`.
- Run logs preserve low-level details without stack traces in display markdown.

Acceptance criteria:
- Missing Python, missing Node and missing Pydantic produce `DependencyError`.
- Stale Manifest still fails before dependency probing.
- Existing `ValidationError`、`PolicyViolation`、`ProtocolViolation` and `RuntimeError` behavior remains stable.

Definition of Done:
- Runtime negative tests pass.
- Full `cargo test` remains green.

Validation commands:
- `cargo test --test runtime --test errors`
- `cargo test`

TDD plan:
- RED: Add failing runtime dependency error tests.
- GREEN: Map dependency failures to envelope.
- REFACTOR: Centralize dependency error construction if helpful.

Packet path:
- `.ai-platform/specs/v0.4/packets/T033.yaml`

Evidence required:
- `.ai-platform/evidence/T033/summary.md`
- `.ai-platform/evidence/T033/test-results.md`
- `.ai-platform/evidence/T033/diff.patch`

### T034: Preserve MCP Server On DependencyError

Status: Accepted
Priority: P0
Depends on: T033
Blocks: T036
Story / Requirement: FR-006, NFR-004
Parallel: No
Conflicts with: T033

Goal:
Map `DependencyError` into MCP tool error results while keeping the stdio server alive.

Allowed files:
- `src/mcp.rs`
- `tests/mcp_server.rs`
- `tests/fixtures/mcp_stdio.rs`

Test targets:
- `cargo test --test mcp_server`

Deliverables:
- Scripted MCP client test for dependency failure.
- Follow-up request after dependency failure proves server survival.
- Tool result contains structured error information.

Acceptance criteria:
- `tools/call` dependency failure returns `isError: true`.
- Server responds to later `tools/list` or `resources/list`.
- stdout discipline remains intact.

Definition of Done:
- MCP tests pass.
- Runtime and MCP error semantics are consistent.

Validation commands:
- `cargo test --test mcp_server`
- `cargo test`

TDD plan:
- RED: Add failing MCP dependency failure survival test.
- GREEN: Adjust MCP error mapping if needed.
- REFACTOR: Avoid language-specific MCP branches.

Packet path:
- `.ai-platform/specs/v0.4/packets/T034.yaml`

Evidence required:
- `.ai-platform/evidence/T034/summary.md`
- `.ai-platform/evidence/T034/test-results.md`
- `.ai-platform/evidence/T034/diff.patch`

### Story S004-4: Prove `.skr` Is Diagnosable, Not A Runtime Image

User value:
Distributed capsules remain portable enough to understand, without pretending to carry their runtime.

### T035: Add Portable `.skr` Check Matrix

Status: Accepted
Priority: P1
Depends on: T031
Blocks: T036
Story / Requirement: FR-007, NFR-001, NFR-002
Parallel: Yes
Conflicts with: T030, T031 if requirements format is still changing

Goal:
Prove unpacked `.skr` archives can be inspected and checked without vendored dependencies or git context.

Allowed files:
- `src/pack.rs`
- `tests/pack.rs`
- `tests/e2e_matrix.rs`
- `docs/v0.4-portable-consumer-checks.md`

Test targets:
- `cargo test --test pack --test e2e_matrix`

Deliverables:
- Pack/unpack tests for `inspect` and `check`.
- Explicit assertions that package-manager artifacts and run history remain excluded.
- Documentation note that `.skr` is diagnosable but not executable everywhere.

Acceptance criteria:
- Unpacked Python and JS capsules can run `inspect`.
- Unpacked Python and JS capsules can run `check`.
- Dependency failure does not affect source hash freshness.

Definition of Done:
- Pack tests pass.
- Docs retain no-vendoring boundary.

Validation commands:
- `cargo test --test pack --test e2e_matrix`
- `cargo test`

TDD plan:
- RED: Add failing unpacked `.skr` check tests.
- GREEN: Adjust pack/check behavior.
- REFACTOR: Keep archive content rules narrow.

Packet path:
- `.ai-platform/specs/v0.4/packets/T035.yaml`

Evidence required:
- `.ai-platform/evidence/T035/summary.md`
- `.ai-platform/evidence/T035/test-results.md`
- `.ai-platform/evidence/T035/diff.patch`

### T036: Prepare v0.4 Release Docs And Validation Matrix

Status: Draft
Priority: P1
Depends on: T033, T034, T035
Blocks: None
Story / Requirement: FR-009, NFR-001, NFR-004
Parallel: No
Conflicts with: All v0.4 implementation tasks

Goal:
Update release-facing documentation and validation evidence for Portable Consumer Checks.

Allowed files:
- `README.md`
- `README.zh-CN.md`
- `RELEASE_NOTES.md`
- `docs/ssot.md`
- `docs/testing.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.4/tasks.md`
- `.ai-platform/specs/v0.4/analysis.md`

Test targets:
- `git diff --check`
- `cargo test`

Deliverables:
- README explains `inspect` / `check` / `doctor`.
- Release notes draft for v0.4.
- Release matrix mentions hostile host environment coverage.
- Task evidence summary paths ready for review.

Acceptance criteria:
- Docs do not claim dependency installation, vendoring, sandboxing or runtime image behavior.
- v0.4 narrative stays focused on portable checks.
- Full validation is green or known limitations are documented.

Definition of Done:
- Release report is ready for user review.
- No task is marked `Accepted` without user acceptance.

Validation commands:
- `git diff --check`
- `cargo test`

TDD plan:
- RED/GREEN applies only if release matrix tests need code changes.
- Documentation/reporting changes use validation and review evidence.

Packet path:
- `.ai-platform/specs/v0.4/packets/T036.yaml`

Evidence required:
- `.ai-platform/evidence/T036/summary.md`
- `.ai-platform/evidence/T036/test-results.md`
- `.ai-platform/evidence/T036/diff.patch`

## User Review Gate

- Approval: Pending.
- Reviewer notes: T029, T030, T031, T032, T033, T034 and T035 were reviewed and accepted on 2026-05-13. T036 remains `Draft`.
