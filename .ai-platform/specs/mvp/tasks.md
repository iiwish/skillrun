# SkillRun Post-Spec Work Graph

Version: v0.1
Status: Confirmed
Feature: mvp
Source spec: `docs/mvp.md`
Last updated: 2026-05-12

## 状态定义

- Draft: task 仍需要 execution packet 或依赖未满足。
- Ready: execution packet 完整，可以开始。
- Running: task 正在执行。
- Needs_Review: implementation 和 evidence 已存在，等待用户验收。
- Accepted: 用户已明确接受。
- Blocked: dependency、environment 或 requirement 问题阻止推进。

## Current Gate

T001、T002、T003、T004、T005、T006、T007 和 T008 均已通过复审并进入 `Accepted`。T009-T011 仍为 `Draft`，需要在依赖满足后再逐一 packetize 和执行。

## Epic E001: Capsule And Manifest

Goal:
Deliver Rust CLI skeleton, `init --python`, Python Action template, schema extraction, Manifest generation and source hash tracking.

Stories:
- US-001
- US-002

## Story US-001: 初始化 Python Action Capsule

User outcome:
用户可以用 Rust CLI `skillrun init refund --python` 创建一个可继续生成 Manifest 的 Python Action capsule。

Validation:
- `cargo test --test init`
- `cargo run -- init refund --python --output tmp/e2e-init`

Tasks:
- [x] T001 [US-001] Scaffold Rust crate and CLI entrypoint
- [x] T002 [US-001] Implement Python Action capsule init templates

## Story US-002: 生成并查看 Manifest 合同

User outcome:
用户可以生成 Manifest，并通过 inspect 理解 schema、hash、permissions、adapter 和风险。

Validation:
- `cargo test --test manifest --test inspect`
- `cargo run -- manifest --cwd tmp/e2e-init/refund`
- `cargo run -- inspect --cwd tmp/e2e-init/refund`

Tasks:
- [x] T003 [US-002] Generate Manifest from Python Action metadata
- [x] T004 [US-002] Render inspect output and instruction-only status

## Epic E002: IPC And Run

Goal:
Deliver Rust Core file-based IPC orchestration, Python Action adapter execution, output/error envelope validation, artifact boundaries and run records.

Stories:
- US-003

## Story US-003: 可控运行和测试 Capsule

User outcome:
用户可以运行 `skillrun test` 和 `skillrun run --input examples/default.input.json`，并得到结构化 envelope、run record 和 artifacts。

Validation:
- `cargo test --test runtime --test errors --test artifacts`
- `cargo run -- test --cwd tmp/e2e-init/refund`
- `cargo run -- run --cwd tmp/e2e-init/refund --input examples/default.input.json`

Tasks:
- [x] T005 [US-003] Implement run records and Python Action adapter IPC success path
- [x] T006 [US-003] Implement structured error envelope handling
- [x] T007 [US-003] Enforce artifact and declared permission boundaries

## Epic E003: Inspect And Failure Discipline

Goal:
Deliver stale Manifest detection, Consumer Mode guard and instruction-only Skill protections across commands.

Stories:
- US-002
- US-006

## Story US-006: 保护 instruction-only Skill

User outcome:
普通 Skill 目录不会因为存在 Markdown、references、assets、scripts 或 examples 而被 SkillRun 当成可执行 capsule。

Validation:
- `cargo test --test consumer_guards --test instruction_only`

Tasks:
- [x] T008 [US-006] Implement stale Manifest and instruction-only command guards

## Epic E004: MCP And Pack

Goal:
Deliver Manifest-driven MCP exposure and `.skr` package generation from Rust Core.

Stories:
- US-004
- US-005

## Story US-004: 从 Manifest 暴露 MCP tool

User outcome:
用户可以运行 `skillrun serve --mcp`；MCP tool schema 来自 Manifest，且 stale Manifest fail closed。

Validation:
- `cargo test --test mcp_server`
- `cargo run -- serve --mcp --cwd tmp/e2e-init/refund --dry-run`

Tasks:
- [ ] T009 [US-004] Implement Manifest-driven MCP tool exposure

## Story US-005: 打包分发 Capsule

User outcome:
用户可以运行 `skillrun pack` 生成 `.skr`，其中包含 Manifest 和 source hashes，不包含 run history。

Validation:
- `cargo test --test pack`
- `cargo run -- pack --cwd tmp/e2e-init/refund`

Tasks:
- [ ] T010 [US-005] Implement `.skr` package generation

## Epic E005: End-to-End Refund And Business Proof Slice

Goal:
Use one high-quality `refund` capsule to prove the full acceptance matrix, then use classic docs-level examples to prove SkillRun business value beyond refund without widening runtime scope.

Validation:
- `cargo test`
- `cargo test --test e2e_matrix --test business_examples`
- `cargo run -- init refund --python --output tmp/e2e-full`
- `cargo run -- manifest --cwd tmp/e2e-full/refund`
- `cargo run -- inspect --cwd tmp/e2e-full/refund`
- `cargo run -- test --cwd tmp/e2e-full/refund`
- `cargo run -- run --cwd tmp/e2e-full/refund --input examples/default.input.json`
- `cargo run -- serve --mcp --cwd tmp/e2e-full/refund --dry-run`
- `cargo run -- pack --cwd tmp/e2e-full/refund`

Tasks:
- [ ] T011 [US-001..US-006] Complete refund hero example, business examples, and full test strategy validation

## Task Details

### T001: Scaffold Rust Crate And CLI Entrypoint

Status: Accepted
Priority: P0
Depends on: None
Blocks: T002, T003, T005, T009, T010, T011
Story / Requirement: US-001, FR-001, NFR-006, TDR-005
Parallel: No
Conflicts with: T002, T003, T005, T009, T010

Goal:
Create the Rust crate skeleton, CLI entrypoint, project metadata and baseline integration test harness.

Allowed files:
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `src/main.rs`
- `tests/cli.rs`

Test targets:
- `tests/cli.rs`

Deliverables:
- Cargo binary crate named `skillrun`.
- Minimal CLI help and version behavior.
- Rust integration test harness ready for later tasks.

Acceptance criteria:
- `cargo run -- --help` exits successfully and lists planned MVP commands.
- `cargo run -- --version` prints `skillrun 0.1.0`.
- `cargo test --test cli` passes.
- README states SkillRun is implemented in Rust and does not claim runtime commands are implemented.

Definition of Done:
- Rust CLI skeleton and package metadata exist.
- Baseline tests pass.
- No runtime feature is falsely claimed as implemented.

Validation commands:
- `cargo test --test cli`
- `cargo run -- --help`
- `cargo run -- --version`

TDD plan:
- RED: Add CLI help/version tests that fail before Cargo crate exists.
- GREEN: Add minimal Rust crate and CLI entrypoint.
- REFACTOR: Keep CLI dispatch simple and preserve future command boundaries.

Packet path:
- `.ai-platform/specs/mvp/packets/T001.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T002: Implement Python Action Capsule Init Templates

Status: Accepted
Priority: P0
Depends on: T001
Blocks: T003, T004, T005, T011
Story / Requirement: US-001, FR-001
Parallel: No
Conflicts with: T001, T003

Goal:
Implement `skillrun init refund --python` in Rust so it creates a standard Python Action Skill Capsule with a runnable default example.

Allowed files:
- `README.md`
- `README.zh-CN.md`
- `src/main.rs`
- `src/cli.rs`
- `src/init.rs`
- `templates/python/SKILL.md`
- `templates/python/action.py`
- `templates/python/examples/default.input.json`
- `templates/python/skillrun.config.json`
- `tests/cli.rs`
- `tests/init.rs`

Test targets:
- `tests/cli.rs`
- `tests/init.rs`

Deliverables:
- `init` command with deterministic output.
- Python Action template containing `Input`, `Output`, `preflight` and `run`.
- Default example that needs no network or secrets.

Acceptance criteria:
- `skillrun init refund --python` creates the standard capsule layout.
- Re-running into an existing non-empty target fails clearly.
- Generated `SKILL.md` is SOP text, not runtime config.

Definition of Done:
- Init tests pass.
- Generated capsule can be used by T003 manifest tests.

Validation commands:
- `cargo test --test init`
- `cargo run -- init refund --python --output tmp/e2e-init`

TDD plan:
- RED: Add tests for generated paths, overwrite guard and template content.
- GREEN: Implement init command and templates.
- REFACTOR: Extract reusable filesystem helpers only if duplication becomes real.

Packet path:
- `.ai-platform/specs/mvp/packets/T002.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T003: Generate Manifest From Python Action Metadata

Status: Accepted
Priority: P0
Depends on: T001, T002
Blocks: T004, T005, T008, T009, T010, T011
Story / Requirement: US-002, FR-002, NFR-001, NFR-003, TDR-001, TDR-002
Parallel: No
Conflicts with: T004, T005, T008

Goal:
Implement `skillrun manifest` in Rust for Author Mode using Pydantic v2 metadata extraction and source hash tracking.

Allowed files:
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `README.zh-CN.md`
- `src/main.rs`
- `src/cli.rs`
- `src/config.rs`
- `src/hashing.rs`
- `src/manifest.rs`
- `src/schemas.rs`
- `src/adapters/python.rs`
- `tests/manifest.rs`
- `tests/fixtures/`

Test targets:
- `tests/manifest.rs`

Deliverables:
- `.skillrun/manifest.generated.yaml` generation.
- source hash entries for `SKILL.md`, `action.py` and optional config.
- input/output schema extraction from Pydantic v2 through a Python metadata subprocess.

Acceptance criteria:
- Manifest contains MVP minimum fields from `docs/mvp.md`.
- Missing source hash is treated as failure.
- metadata phase does not require secrets.

Definition of Done:
- Manifest tests pass.
- Generated Manifest can be consumed by inspect and run tasks.

Validation commands:
- `cargo test --test manifest`
- `cargo run -- manifest --cwd tmp/e2e-init/refund`

TDD plan:
- RED: Add fixture tests for schema extraction, hash fields and stale source detection hooks.
- GREEN: Implement manifest generation and metadata extraction.
- REFACTOR: Keep Manifest model stable before downstream tasks depend on it.

Packet path:
- `.ai-platform/specs/mvp/packets/T003.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T004: Render Inspect Output And Instruction-only Status

Status: Accepted
Priority: P1
Depends on: T001, T003
Blocks: T008, T011
Story / Requirement: US-002, US-006, FR-003, FR-009, TDR-004
Parallel: No
Conflicts with: T003

Goal:
Implement `skillrun inspect` for runnable capsules and instruction-only Skill directories.

Allowed files:
- `README.md`
- `README.zh-CN.md`
- `src/main.rs`
- `src/cli.rs`
- `src/inspect.rs`
- `src/manifest.rs`
- `tests/cli.rs`
- `tests/inspect.rs`
- `tests/fixtures/instruction_only/`

Test targets:
- `tests/inspect.rs`

Deliverables:
- Human-readable inspect output for runnable capsule.
- Instruction-only status and reason when `action.py` or valid Manifest is missing.

Acceptance criteria:
- inspect does not execute business `run`.
- inspect shows skill name, SOP hash, schema, adapter, permissions, examples, preflight status and MCP tool summary.
- instruction-only Skill output clearly says it is not a runnable capsule.

Definition of Done:
- Inspect tests pass.
- No command silently upgrades instruction-only Skill into runnable capsule.

Validation commands:
- `cargo test --test inspect`
- `cargo run -- inspect --cwd tmp/e2e-init/refund`

TDD plan:
- RED: Add inspect output tests for runnable and instruction-only directories.
- GREEN: Implement inspect renderer.
- REFACTOR: Share Manifest loading logic with guard task without changing behavior.

Packet path:
- `.ai-platform/specs/mvp/packets/T004.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T005: Implement Run Records And Python Action Adapter IPC Success Path

Status: Accepted
Priority: P0
Depends on: T001, T002, T003
Blocks: T006, T007, T011
Story / Requirement: US-003, FR-004, FR-005, NFR-002, NFR-005, TDR-003
Parallel: No
Conflicts with: T006, T007

Goal:
Implement `skillrun test` and `skillrun run` success path using Rust Core file-based IPC, Python Action adapter execution and run records.

Allowed files:
- `src/main.rs`
- `src/cli.rs`
- `src/errors.rs`
- `src/runtime.rs`
- `src/run_record.rs`
- `src/manifest.rs`
- `src/adapters/python.rs`
- `tests/cli.rs`
- `tests/runtime.rs`
- `tests/fixtures/runtime_success/`
- `README.md`
- `README.zh-CN.md`

Test targets:
- `tests/runtime.rs`

Deliverables:
- Run-local input, context, output, stdout, stderr and artifact directories.
- Adapter reads IPC env vars and writes success envelope.
- `test` uses `examples/default.input.json`.

Acceptance criteria:
- `skillrun test` creates a test run and validates output envelope.
- `skillrun run --input examples/default.input.json` creates unique run id and output.
- stdout/stderr are captured as logs only.

Definition of Done:
- Runtime success tests pass.
- Run record contains skill hash, manifest hash, action hash, status, timestamps and declared permissions.

Validation commands:
- `cargo test --test runtime`
- `cargo run -- test --cwd tmp/e2e-init/refund`
- `cargo run -- run --cwd tmp/e2e-init/refund --input examples/default.input.json`

TDD plan:
- RED: Add failing tests for run directory shape and success envelope.
- GREEN: Implement runtime orchestration and adapter success path.
- REFACTOR: Separate run record persistence from adapter invocation.

Packet path:
- `.ai-platform/specs/mvp/packets/T005.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T006: Implement Structured Error Envelope Handling

Status: Accepted
Priority: P0
Depends on: T005
Blocks: T011
Story / Requirement: US-003, FR-006, TDR-003
Parallel: No
Conflicts with: T005, T007

Goal:
Implement structured error handling for validation, policy, protocol and runtime failures.

Allowed files:
- `src/errors.rs`
- `src/main.rs`
- `src/runtime.rs`
- `src/adapters/python.rs`
- `tests/errors.rs`
- `tests/fixtures/error_cases/`
- `README.md`
- `README.zh-CN.md`

Test targets:
- `tests/errors.rs`

Deliverables:
- `ValidationError` for input schema failures.
- `PolicyViolation` for preflight or business policy rejection.
- `ProtocolViolation` for missing or invalid output.
- `RuntimeError` for uncategorized runtime failures.

Acceptance criteria:
- Error envelope includes `code`, `message`, `recoverable` and optional `llm_hint`.
- Stack traces stay in debug logs, not final display output.
- Protocol violation is not masked by stdout.

Definition of Done:
- Error tests pass.
- At least one reproducible test exists for `ValidationError`, `PolicyViolation` and `ProtocolViolation`.

Validation commands:
- `cargo test --test errors`

TDD plan:
- RED: Add failing tests for each required error code.
- GREEN: Implement error mapping and adapter exception handling.
- REFACTOR: Normalize error envelope construction.

Packet path:
- `.ai-platform/specs/mvp/packets/T006.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T007: Enforce Artifact And Declared Permission Boundaries

Status: Accepted
Priority: P1
Depends on: T005
Blocks: T011
Story / Requirement: US-003, FR-004, FR-005, NFR-004, NFR-005, TDR-003
Parallel: No
Conflicts with: T005, T006

Goal:
Validate artifact path boundaries and declared env injection behavior for run/test.

Allowed files:
- `README.md`
- `README.zh-CN.md`
- `src/main.rs`
- `src/config.rs`
- `src/manifest.rs`
- `src/errors.rs`
- `src/runtime.rs`
- `src/run_record.rs`
- `src/permissions.rs`
- `src/adapters/python.rs`
- `tests/artifacts.rs`
- `tests/permissions.rs`
- `tests/fixtures/artifact_cases/`

Test targets:
- `tests/artifacts.rs`
- `tests/permissions.rs`

Deliverables:
- Artifact path normalization and containment check.
- Declared env-only injection into child process.
- Permission records in inspect/run evidence.

Acceptance criteria:
- Artifact outside `SKILLRUN_ARTIFACT_DIR` is rejected.
- Undeclared env vars are not injected by SkillRun.
- Run record includes declared permissions.

Definition of Done:
- Artifact and permission tests pass.
- Security boundary limitations remain explicit in docs and CLI warnings.

Validation commands:
- `cargo test --test artifacts --test permissions`

TDD plan:
- RED: Add failing tests for artifact traversal and env injection.
- GREEN: Implement permission and artifact validation.
- REFACTOR: Keep path checks reusable by pack or future install commands.

Packet path:
- `.ai-platform/specs/mvp/packets/T007.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T008: Implement Stale Manifest And Instruction-only Command Guards

Status: Accepted
Priority: P0
Depends on: T003, T004
Blocks: T009, T010, T011
Story / Requirement: US-002, US-006, FR-002, FR-007, FR-008, FR-009, NFR-001, TDR-002, TDR-004
Parallel: No
Conflicts with: T003, T004, T009, T010

Goal:
Centralize Consumer Mode validation so stale Manifest and instruction-only directories fail closed across commands.

Allowed files:
- `src/main.rs`
- `src/cli.rs`
- `src/consumer.rs`
- `src/manifest.rs`
- `src/inspect.rs`
- `tests/cli.rs`
- `tests/inspect.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`
- `tests/fixtures/stale_manifest/`
- `tests/fixtures/instruction_only/`

Test targets:
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`

Deliverables:
- Hash mismatch detection.
- Missing Manifest and missing `action.py` guard.
- Command-specific refusal messages for run, serve and pack.

Acceptance criteria:
- `serve --mcp` and `pack` fail closed when Manifest is stale.
- `manifest` refuses to guess action from Markdown, scripts or examples.
- `run` refuses instruction-only Skill with clear next step.

Definition of Done:
- Guard tests pass.
- Downstream MCP and pack tasks can call the same guard API.

Validation commands:
- `cargo test --test consumer_guards --test instruction_only`

TDD plan:
- RED: Add stale Manifest and instruction-only refusal tests.
- GREEN: Implement central Consumer Mode guard.
- REFACTOR: Deduplicate command error formatting.

Packet path:
- `.ai-platform/specs/mvp/packets/T008.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T009: Implement Manifest-driven MCP Tool Exposure

Status: Draft
Priority: P1
Depends on: T003, T008
Blocks: T011
Story / Requirement: US-004, FR-007, NFR-001, TDR-006
Parallel: No
Conflicts with: T008, T010

Goal:
Implement `skillrun serve --mcp` as a thin Manifest-driven MCP exposure path.

Allowed files:
- `Cargo.toml`
- `src/main.rs`
- `src/cli.rs`
- `src/mcp.rs`
- `src/manifest.rs`
- `tests/mcp_server.rs`

Test targets:
- `tests/mcp_server.rs`

Deliverables:
- MCP tool schema and description derived from Manifest.
- `SKILL.md` exposed as resource.
- `--dry-run` mode for local contract verification.

Acceptance criteria:
- MCP tool input schema comes from Manifest.
- `serve --mcp` does not import `action.py` for metadata.
- stale Manifest fails closed before server startup.

Definition of Done:
- MCP tests pass.
- Any MCP SDK dependency is isolated to the MCP server module.

Validation commands:
- `cargo test --test mcp_server`
- `cargo run -- serve --mcp --cwd tmp/e2e-init/refund --dry-run`

TDD plan:
- RED: Add tests for Manifest-to-tool schema mapping and stale failure.
- GREEN: Implement MCP contract builder and dry-run.
- REFACTOR: Keep runtime invocation path shared with `run`.

Packet path:
- `.ai-platform/specs/mvp/packets/T009.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T010: Implement `.skr` Package Generation

Status: Draft
Priority: P1
Depends on: T003, T008
Blocks: T011
Story / Requirement: US-005, FR-008, NFR-004, TDR-007
Parallel: No
Conflicts with: T008, T009

Goal:
Implement `skillrun pack` so a capsule can be distributed as a `.skr` archive.

Allowed files:
- `src/main.rs`
- `src/cli.rs`
- `src/pack.rs`
- `src/consumer.rs`
- `tests/pack.rs`
- `tests/fixtures/pack_cases/`

Test targets:
- `tests/pack.rs`

Deliverables:
- `dist/refund-0.1.0.skr` tar.gz archive for the default example and equivalent archive naming for other capsule names.
- Pack preflight using Manifest source hash validation.
- Exclusion of `.skillrun/runs/`.

Acceptance criteria:
- `.skr` contains `SKILL.md`, `action.py`, optional config, Manifest and examples.
- `.skr` excludes run history.
- pack fails when Manifest is missing or stale.

Definition of Done:
- Pack tests pass.
- README or generated summary states `.skr` does not vendor dependencies.

Validation commands:
- `cargo test --test pack`
- `cargo run -- pack --cwd tmp/e2e-init/refund`

TDD plan:
- RED: Add archive content and stale Manifest tests.
- GREEN: Implement pack command and archive filtering.
- REFACTOR: Share source-hash preflight with Consumer Mode guard.

Packet path:
- `.ai-platform/specs/mvp/packets/T010.yaml`

Evidence required:
- Changed files.
- RED/GREEN validation results.
- Diff summary.
- Residual risks.

### T011: Complete Refund Hero Example, Business Examples, And Full Test Strategy Validation

Status: Draft
Priority: P0
Depends on: T002, T003, T004, T005, T006, T007, T008, T009, T010
Blocks: Release acceptance
Story / Requirement: US-001, US-002, US-003, US-004, US-005, US-006, FR-001 through FR-009, NFR-001 through NFR-006
Parallel: No
Conflicts with: All implementation tasks

Goal:
Polish the `refund` hero example, README and end-to-end validation so the full test strategy is demonstrably covered: A001-A013 acceptance, Negative/Security Matrix, B001 implemented business proof and B002-B004 docs-level business examples.

Allowed files:
- `README.md`
- `docs/business-examples.md`
- `examples/refund/SKILL.md`
- `examples/refund/action.py`
- `examples/refund/examples/default.input.json`
- `examples/refund/examples/policy_violation.input.json`
- `examples/refund/examples/invalid.input.json`
- `tests/e2e_matrix.rs`
- `tests/business_examples.rs`
- `.ai-platform/docs/release-report.md`

Test targets:
- `tests/e2e_matrix.rs`
- `tests/business_examples.rs`

Deliverables:
- High-quality B001 `refund` hero example.
- README and `docs/business-examples.md` with approved SkillRun narrative and B002-B004 classic business examples.
- Full A001-A013 validation summary.
- Negative/Security Matrix coverage summary.

Acceptance criteria:
- A001-A013 all have fresh command evidence.
- B001 is fully implemented and tested end-to-end.
- B002-B004 are explained in README or `docs/business-examples.md` without entering v0.1 implementation scope.
- Negative/Security Matrix cases N001-N016 have automated tests or explicit documented exceptions.

Definition of Done:
- `cargo test` passes.
- E2E command sequence completes on generated refund capsule.
- `.skr` generated from refund can be unpacked and inspected.
- Release report is updated to `Ready_For_User_Review`.

Validation commands:
- `cargo test`
- `cargo test --test e2e_matrix --test business_examples`
- `cargo run -- init refund --python --output tmp/e2e-full`
- `cargo run -- manifest --cwd tmp/e2e-full/refund`
- `cargo run -- inspect --cwd tmp/e2e-full/refund`
- `cargo run -- test --cwd tmp/e2e-full/refund`
- `cargo run -- run --cwd tmp/e2e-full/refund --input examples/default.input.json`
- `cargo run -- serve --mcp --cwd tmp/e2e-full/refund --dry-run`
- `cargo run -- pack --cwd tmp/e2e-full/refund`

TDD plan:
- RED: Add failing E2E and business-example tests for uncovered A001-A013, N001-N016 and B001-B004 expectations.
- GREEN: Fill gaps in example, docs and integration behavior.
- REFACTOR: Remove duplicated fixture setup and keep README concise.

Packet path:
- `.ai-platform/specs/mvp/packets/T011.yaml`

Evidence required:
- Changed files.
- Full validation command results.
- A001-A013 coverage summary.
- N001-N016 coverage or documented exceptions.
- B001-B004 business example summary.
- Diff summary.
- Residual risks.

## Dependency Graph

- T001 -> T002, T003, T005, T009, T010, T011
- T002 -> T003, T004, T005, T011
- T003 -> T004, T005, T008, T009, T010, T011
- T004 -> T008, T011
- T005 -> T006, T007, T011
- T006 -> T011
- T007 -> T011
- T008 -> T009, T010, T011
- T009 -> T011
- T010 -> T011

## Parallel Eligibility

- No task is currently marked parallel. Parallel execution can be reconsidered only after prior dependencies are accepted and write scopes are rechecked.
- T006 and T007 are intentionally serial because both touch runtime and adapter error/permission paths.
- T009 and T010 are intentionally serial because both touch CLI dispatch and Consumer Mode guard integration.

## User Review Gate

- Approval: Approved on 2026-05-11
- Reviewer notes: Rust-first correction replaces the earlier non-Rust implementation path. T001 awaits explicit user acceptance before T002 starts.
