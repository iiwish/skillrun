# SkillRun v0.3 Work Graph And Tasks

Version: v0.3
Status: Confirmed
Source spec: `.ai-platform/specs/v0.3/spec.md`
Source plan: `.ai-platform/specs/v0.3/plan.md`
Last updated: 2026-05-13
Review: Approved by user on 2026-05-13 after plan/tasks review request

## Work Graph Summary

v0.3 work graph starts after accepted v0.2 tasks `T012` through `T018`. New task IDs begin at `T019`.

```text
T019 -> T020 -> T022 -> T023 -> T024 -> T025 -> T028
          |        ^
          v        |
         T021 -----+
          |
          v
         T027

T020 -> T026 -> T028
T027 -> T028
```

## Epic E003: Adapter Boundary And JS Action Alpha

Goal:
Prove SkillRun is a Manifest-driven multi-adapter runtime while keeping Python stable and JS alpha narrow.

### Story S003-1: Preserve Python While Extracting Adapter Boundary

User value:
Existing SkillRun users keep the Python path, while maintainers gain a real adapter seam for JS alpha.

### T019: Add Python Path Characterization Tests

Status: Accepted
Priority: P0
Depends on: T018
Blocks: T020
Story / Requirement: FR-001, FR-002, NFR-003, NFR-004
Parallel: No
Conflicts with: T020, T024

Goal:
Capture the current Python capsule behavior before adapter refactor.

Allowed files:
- `tests/manifest.rs`
- `tests/runtime.rs`
- `tests/e2e_matrix.rs`
- `tests/consumer_guards.rs`

Test targets:
- `cargo test --test manifest --test runtime --test e2e_matrix --test consumer_guards`

Deliverables:
- Regression tests proving Python `init --python -> manifest -> inspect/test/run` behavior remains stable.
- Tests that document current unsupported-adapter behavior before replacing it with adapter dispatch.

Acceptance criteria:
- Tests pass against current Python behavior.
- Tests do not introduce JS implementation.
- Tests avoid weakening existing assertions.

Definition of Done:
- Relevant tests are committed in the task diff.
- Evidence records command output and any expected behavior locked by the characterization.

Validation commands:
- `cargo test --test manifest --test runtime --test e2e_matrix --test consumer_guards`

TDD plan:
- RED is not required because this is characterization coverage.
- GREEN is demonstrated by passing tests that protect current behavior.
- REFACTOR is not allowed in this task.

Packet path:
- `.ai-platform/specs/v0.3/packets/T019.yaml`

Evidence required:
- `.ai-platform/evidence/T019/summary.md`
- `.ai-platform/evidence/T019/test-results.md`
- `.ai-platform/evidence/T019/diff.patch`

### T020: Introduce Adapter Dispatch Boundary

Status: Accepted
Priority: P0
Depends on: T019
Blocks: T022, T023, T026
Story / Requirement: FR-001, FR-002, FR-006, NFR-003, NFR-004
Parallel: No
Conflicts with: T021, T022, T023, T024, T026

Goal:
Move metadata extraction and runtime execution behind an adapter dispatch boundary while preserving Python behavior.

Allowed files:
- `src/main.rs`
- `src/adapters/python.rs`
- `src/adapters/mod.rs`
- `src/manifest.rs`
- `src/runtime.rs`
- `src/config.rs`
- `tests/manifest.rs`
- `tests/runtime.rs`
- `tests/e2e_matrix.rs`

Test targets:
- `cargo test --test manifest --test runtime --test e2e_matrix`
- `cargo test`

Deliverables:
- Shared adapter request/output types or equivalent dispatch module.
- Python adapter wired through dispatch for metadata and runtime.
- No user-visible behavior change for Python.

Acceptance criteria:
- Existing Python release matrix passes.
- Unsupported adapter errors remain clear and deterministic.
- Manifest and run record shape remain compatible with v0.2 unless explicitly versioned.

Definition of Done:
- Python path passes full `cargo test`.
- Diff shows Core no longer directly assumes Python in both manifest generation and runtime execution.

Validation commands:
- `cargo test --test manifest --test runtime --test e2e_matrix`
- `cargo test`

TDD plan:
- RED: Add or use tests from T019 that fail if adapter dispatch regresses Python behavior.
- GREEN: Refactor to adapter dispatch with Python passing.
- REFACTOR: Clean duplicated request structs only after tests pass.

Packet path:
- `.ai-platform/specs/v0.3/packets/T020.yaml`

Evidence required:
- `.ai-platform/evidence/T020/summary.md`
- `.ai-platform/evidence/T020/test-results.md`
- `.ai-platform/evidence/T020/diff.patch`

### Story S003-2: Create JS Alpha Author Path

User value:
An author can start a JS alpha capsule from one command without learning SkillRun internals.

### T021: Add Explicit Init Language Flags, `--py` Alias, And JS Templates

Status: Accepted
Priority: P0
Depends on: T020
Blocks: T022, T024, T027
Story / Requirement: FR-003, FR-004, FR-009, FR-010, NFR-001, NFR-002
Parallel: No
Conflicts with: T020, T024

Goal:
Add explicit init language selection, the `--py` alias, and the JS alpha capsule skeleton without implementing Node runtime execution yet.

Allowed files:
- `src/cli.rs`
- `src/init.rs`
- `templates/js/SKILL.md`
- `templates/js/action.mjs`
- `templates/js/examples/default.input.json`
- `templates/js/skillrun.config.json`
- `tests/init.rs`
- `tests/cli.rs`

Test targets:
- `cargo test --test init --test cli`

Deliverables:
- `skillrun init refund --js` creates the JS alpha capsule.
- `skillrun init refund --py` creates the same capsule as `skillrun init refund --python`.
- Generated config uses `runtime.adapter: "node"` and `runtime.entrypoint: "action.mjs"`.
- CLI help reflects Python stable path, `--py` alias, and JS alpha path.

Acceptance criteria:
- `--python` behavior remains unchanged.
- `--py` behavior matches `--python`.
- `--js` and Python flags are mutually exclusive.
- `skillrun init refund` without a language flag fails with a clear recovery hint.
- Generated `action.mjs` exports `inputSchema`, `outputSchema`, optional `preflight`, and `run`.
- No package manager files are generated.

Definition of Done:
- Init tests cover generated JS files and config.
- CLI errors are clear for missing language flag or conflicting language flags.
- No language flags are added to `test`、`run`、`serve --mcp` or `pack`.

Validation commands:
- `cargo test --test init --test cli`
- `cargo test`

TDD plan:
- RED: Add failing tests for `init --py`, missing language flag, conflicting flags, and `init --js` output.
- GREEN: Implement parser alias and JS templates.
- REFACTOR: Keep language selection simple and explicit.

Packet path:
- `.ai-platform/specs/v0.3/packets/T021.yaml`

Evidence required:
- `.ai-platform/evidence/T021/summary.md`
- `.ai-platform/evidence/T021/test-results.md`
- `.ai-platform/evidence/T021/diff.patch`

### T022: Implement Node Metadata Extraction From Explicit JSON Schema

Status: Accepted
Priority: P0
Depends on: T020, T021
Blocks: T023, T024
Story / Requirement: FR-001, FR-004, FR-006, NFR-002, NFR-003, NFR-004
Parallel: No
Conflicts with: T020, T023, T024

Goal:
Allow `skillrun manifest` to resolve adapter by config-first deterministic convention and generate Manifest schemas from `action.mjs` explicit JSON Schema exports.

Allowed files:
- `src/adapters/mod.rs`
- `src/adapters/node.rs`
- `src/manifest.rs`
- `src/consumer.rs`
- `tests/manifest.rs`
- `tests/consumer_guards.rs`

Test targets:
- `cargo test --test manifest --test consumer_guards`

Deliverables:
- Node metadata adapter.
- Manifest generation for JS alpha capsules.
- Config-first adapter resolution, with deterministic `action.py` / `action.mjs` convention fallback.
- Clear errors for missing `node`, missing `inputSchema`, missing `outputSchema`, invalid JSON Schema object, and unsupported `action.ts`.

Acceptance criteria:
- JS Manifest records `sources.action.path: action.mjs`.
- JS Manifest records `runtime.adapter: node`.
- Config runtime settings override action-file convention.
- Multiple known action files fail as ambiguous.
- Metadata phase has timeout behavior equivalent to Python.
- Consumer Mode validation remains static and hash-based.
- No TypeScript type inference is introduced.

Definition of Done:
- JS metadata tests pass.
- Python metadata tests still pass.
- Error messages tell the author the next recovery command or boundary.

Validation commands:
- `cargo test --test manifest --test consumer_guards`
- `cargo test`

TDD plan:
- RED: Add failing JS Manifest generation tests, config-first convention tests, ambiguous-action tests, and unsupported TS tests.
- GREEN: Implement Node metadata extraction.
- REFACTOR: Move shared timeout/error helpers only if duplication becomes meaningful.

Packet path:
- `.ai-platform/specs/v0.3/packets/T022.yaml`

Evidence required:
- `.ai-platform/evidence/T022/summary.md`
- `.ai-platform/evidence/T022/test-results.md`
- `.ai-platform/evidence/T022/diff.patch`

### Story S003-3: Execute JS Alpha Through Shared Runtime

User value:
The same Skill Capsule commands work for JS alpha without creating a separate product surface.

### T023: Implement Node Runtime Adapter

Status: Accepted
Priority: P0
Depends on: T020, T022
Blocks: T024, T025
Story / Requirement: FR-001, FR-005, FR-006, NFR-001, NFR-003, NFR-004
Parallel: No
Conflicts with: T020, T022, T024, T025

Goal:
Run `action.mjs` through the existing runtime envelope, artifact and run record path.

Allowed files:
- `src/adapters/mod.rs`
- `src/adapters/node.rs`
- `src/runtime.rs`
- `src/errors.rs`
- `tests/runtime.rs`
- `tests/errors.rs`
- `tests/artifacts.rs`

Test targets:
- `cargo test --test runtime --test errors --test artifacts`

Deliverables:
- Node runtime adapter supporting sync and async `run`.
- Optional `preflight`.
- Output/error envelope compatibility.
- Artifact validation remains Rust-side.

Acceptance criteria:
- JS success path writes `ok: true` envelope.
- JS validation failure maps to recoverable `ValidationError`.
- JS preflight/business failure maps to recoverable `PolicyViolation`.
- Malformed output maps to `ProtocolViolation`.
- Unexpected thrown error maps to non-recoverable `RuntimeError`.
- stdout/stderr remain logs, not result fallback.

Definition of Done:
- Runtime tests pass for Python and JS.
- Run record contains Manifest/action hashes for JS.
- Permissions/env injection behavior remains Manifest-declared.

Validation commands:
- `cargo test --test runtime --test errors --test artifacts`
- `cargo test`

TDD plan:
- RED: Add failing runtime tests for JS success and failure envelopes.
- GREEN: Implement Node runtime adapter.
- REFACTOR: Share adapter runner helpers only after tests pass.

Packet path:
- `.ai-platform/specs/v0.3/packets/T023.yaml`

Evidence required:
- `.ai-platform/evidence/T023/summary.md`
- `.ai-platform/evidence/T023/test-results.md`
- `.ai-platform/evidence/T023/diff.patch`

### T024: Add JS Alpha End-to-end Command Matrix

Status: Accepted
Priority: P0
Depends on: T021, T022, T023
Blocks: T025, T028
Story / Requirement: FR-002, FR-003, FR-004, FR-005, FR-006, FR-010, NFR-004
Parallel: No
Conflicts with: T020, T021, T022, T023, T025

Goal:
Prove the JS alpha capsule works through the complete local command chain.

Allowed files:
- `tests/e2e_matrix.rs`
- `tests/inspect.rs`
- `tests/runtime.rs`
- `tests/manifest.rs`
- `src/inspect.rs`

Test targets:
- `cargo test --test e2e_matrix --test inspect --test runtime --test manifest`

Deliverables:
- JS alpha E2E matrix for `init --js -> manifest -> inspect -> test -> run`.
- Python alias smoke path for `init --py -> manifest`.
- Inspect output includes adapter and entrypoint clearly.
- Python E2E matrix remains green.

Acceptance criteria:
- JS alpha command path requires no package manager install.
- `init --py` remains Python and does not create a separate adapter identity.
- `inspect` explains Manifest-derived adapter/entrypoint contract.
- Stale Manifest behavior works for `action.mjs`.

Definition of Done:
- E2E tests pass on a machine with Node available.
- Node-missing behavior is either skipped with explicit signal in tests or covered by deterministic error tests.

Validation commands:
- `cargo test --test e2e_matrix --test inspect --test runtime --test manifest`
- `cargo test`

TDD plan:
- RED: Add failing JS E2E matrix tests.
- GREEN: Wire missing inspect/runtime/consumer behavior.
- REFACTOR: Keep JS alpha path compact.

Packet path:
- `.ai-platform/specs/v0.3/packets/T024.yaml`

Evidence required:
- `.ai-platform/evidence/T024/summary.md`
- `.ai-platform/evidence/T024/test-results.md`
- `.ai-platform/evidence/T024/diff.patch`

### T025: Extend MCP And Pack Compatibility To JS Alpha

Status: Ready
Priority: P0
Depends on: T024
Blocks: T028
Story / Requirement: FR-007, FR-006, NFR-001, NFR-003, NFR-004
Parallel: No
Conflicts with: T023, T024

Goal:
Show that JS alpha capsules use the same Manifest-derived MCP and `.skr` package surfaces as Python capsules.

Allowed files:
- `tests/mcp_server.rs`
- `tests/fixtures/mcp_stdio.rs`
- `tests/pack.rs`
- `tests/e2e_matrix.rs`
- `src/mcp.rs`
- `src/pack.rs`

Test targets:
- `cargo test --test mcp_server --test pack --test e2e_matrix`

Deliverables:
- JS alpha `serve --mcp --dry-run` contract test.
- JS alpha stdio MCP client matrix if feasible in the existing fixture.
- JS alpha `.skr` package contents test.

Acceptance criteria:
- MCP tool schema comes from Manifest for JS alpha.
- MCP tool call reuses runtime dispatch and produces run evidence.
- `.skr` contains source and Manifest, excludes run history, and does not vendor dependencies.

Definition of Done:
- MCP and pack tests pass for Python and JS alpha.
- Docs or release report notes dependency/runtime-image boundary if needed.

Validation commands:
- `cargo test --test mcp_server --test pack --test e2e_matrix`
- `cargo test`

TDD plan:
- RED: Add failing JS MCP/pack tests.
- GREEN: Make existing MCP/pack code language-neutral where necessary.
- REFACTOR: Avoid duplicating Python/JS MCP assertions.

Packet path:
- `.ai-platform/specs/v0.3/packets/T025.yaml`

Evidence required:
- `.ai-platform/evidence/T025/summary.md`
- `.ai-platform/evidence/T025/test-results.md`
- `.ai-platform/evidence/T025/diff.patch`

### Story S003-4: Improve Author Quality Loop

User value:
Authors can diagnose common Python/JS capsule issues without reading source code.

### T026: Implement Adapter-aware `doctor` / `validate`

Status: Draft
Priority: P1
Depends on: T020
Blocks: T028
Story / Requirement: FR-008, FR-009, FR-010, NFR-001, NFR-002, NFR-004
Parallel: No
Conflicts with: T020, T022, T024

Goal:
Add a non-executing diagnostics entrypoint that explains capsule structure, Manifest freshness and adapter-specific recovery steps.

Allowed files:
- `src/cli.rs`
- `src/consumer.rs`
- `src/manifest.rs`
- `src/inspect.rs`
- `src/doctor.rs`
- `src/main.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`
- `tests/cli.rs`

Test targets:
- `cargo test --test consumer_guards --test instruction_only --test cli`

Deliverables:
- `skillrun doctor --cwd refund` or `skillrun validate --cwd refund`.
- Adapter-aware messages for Python, JS alpha, instruction-only and unsupported TypeScript.
- No business action execution.

Acceptance criteria:
- Diagnostics check required files, Manifest presence/freshness, source hash status and examples presence.
- Diagnostics report `action.ts` as unsupported and suggest compiling to `action.mjs`.
- Diagnostics do not import action for metadata in Consumer Mode.
- Diagnostics do not suggest passing language flags to Consumer Mode commands.

Definition of Done:
- Diagnostics tests cover Python, JS alpha, instruction-only and stale Manifest scenarios.
- Command name and help text are documented.

Validation commands:
- `cargo test --test consumer_guards --test instruction_only --test cli`
- `cargo test`

TDD plan:
- RED: Add failing CLI/diagnostics tests.
- GREEN: Implement minimal diagnostic command.
- REFACTOR: Share stale Manifest recovery helpers only after tests pass.

Packet path:
- `.ai-platform/specs/v0.3/packets/T026.yaml`

Evidence required:
- `.ai-platform/evidence/T026/summary.md`
- `.ai-platform/evidence/T026/test-results.md`
- `.ai-platform/evidence/T026/diff.patch`

### T027: Update README And TypeScript Boundary Docs

Status: Draft
Priority: P1
Depends on: T021
Blocks: T028
Story / Requirement: FR-009, NFR-001, NFR-002
Parallel: Yes
Conflicts with: T028

Goal:
Explain Python stable path, `--py` alias, JS alpha path, TypeScript boundary and language-flag placement without weakening SkillRun's core narrative.

Allowed files:
- `README.md`
- `docs/ssot.md`
- `docs/business-examples.md`
- `.ai-platform/specs/v0.3/issue-drafts.md`

Test targets:
- `git diff --check`
- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`

Deliverables:
- README Quickstart keeps Python as the main path.
- `--py` is documented as an alias in CLI/reference material, not as the README main path.
- JS alpha path is clearly labeled.
- TypeScript boundary is explicit.
- Runtime commands are documented as Manifest-only and language-flag-free.
- No security, dependency or package-manager overclaim.

Acceptance criteria:
- A reader can explain why SkillRun is not FastMCP after the first screen.
- A reader can explain why JS alpha is not full TS support.
- A reader can explain why `init` needs a language flag but `run/test/serve/pack` do not.
- README and SSOT use the same language for `action.mjs`, `action.ts` and `.skr`.

Definition of Done:
- Documentation checks pass.
- No current v0.2 capability is described as already supporting JS before implementation lands.

Validation commands:
- `git diff --check`
- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`

TDD plan:
- RED/GREEN does not apply to documentation-only changes.
- Review uses narrative consistency, scope boundary and diff hygiene checks.

Packet path:
- `.ai-platform/specs/v0.3/packets/T027.yaml`

Evidence required:
- `.ai-platform/evidence/T027/summary.md`
- `.ai-platform/evidence/T027/test-results.md`
- `.ai-platform/evidence/T027/diff.patch`

### T028: Prepare v0.3 Release Matrix And Report

Status: Draft
Priority: P1
Depends on: T024, T025, T026, T027
Blocks: None
Story / Requirement: FR-002, FR-007, FR-009, NFR-001, NFR-002, NFR-004
Parallel: No
Conflicts with: All v0.3 implementation tasks

Goal:
Prepare release-facing evidence that v0.3 delivered adapter boundary, JS alpha and author quality loop without expanding out-of-scope claims.

Allowed files:
- `README.md`
- `RELEASE_NOTES.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.3/tasks.md`
- `.ai-platform/specs/v0.3/analysis.md`
- `tests/e2e_matrix.rs`

Test targets:
- `cargo test`
- `cargo run -- --version`
- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`

Deliverables:
- v0.3 release matrix.
- Updated release notes draft.
- Release report draft.
- Tasks updated to reflect completed evidence after user acceptance.

Acceptance criteria:
- Python release matrix remains green.
- JS alpha release matrix is green where Node is available.
- Release notes state `action.mjs` alpha and TypeScript out-of-scope boundaries.
- `.skr` remains described as source + Manifest archive only.

Definition of Done:
- Full validation passes.
- Release report is ready for user review.
- No task is marked `Accepted` without user acceptance.

Validation commands:
- `cargo test`
- `cargo run -- --version`
- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`

TDD plan:
- RED/GREEN applies only if release matrix tests need code changes.
- Documentation/reporting changes use validation and review evidence.

Packet path:
- `.ai-platform/specs/v0.3/packets/T028.yaml`

Evidence required:
- `.ai-platform/evidence/T028/summary.md`
- `.ai-platform/evidence/T028/test-results.md`
- `.ai-platform/evidence/T028/diff.patch`

## User Review Gate

- Approval: Approved on 2026-05-13.
- Reviewer notes: Work graph reviewed with no blocking findings. Packets T019-T028 are created. T019 is the only currently ready task because downstream tasks depend on unfinished work.
