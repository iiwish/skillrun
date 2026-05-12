# SkillRun v0.2 Work Graph

Version: v0.2
Status: Confirmed
Feature: v0.2
Source spec: `.ai-platform/specs/v0.2/spec.md`
Last updated: 2026-05-12
Review: Codex review passed on 2026-05-12; user authorized continuation after review

## 状态定义

- Draft: task 仍需要用户批准、requirements checklist、analysis 或 execution packet。
- Ready: execution packet 完整，可以开始。
- Running: task 正在执行。
- Needs_Review: implementation 和 evidence 已存在，等待 review/验收。
- Accepted: 用户已明确接受。
- Blocked: dependency、environment 或 requirement 问题阻止推进。

## Current Gate

本 work graph 已通过 review 并进入 `Confirmed`。requirements checklist 和 analysis 通过后，只有依赖已满足且 packet 完整的 task 可以进入 `Ready`。T012 是第一项可执行任务；T013-T018 仍受依赖链约束。

## Epic E201: Release Narrative

Goal:
让 README 首页形成正确的公开项目心智：SkillRun 是 Manifest-driven SOP-backed Skill Capsule runtime，不是 FastMCP 替代品或安全 package manager。

Stories:

- US-201

### T012: Rewrite README Release Narrative

Status: Accepted
Priority: P0
Depends on: v0.2 SOP Confirmed
Blocks: T018
Story / Requirement: US-201, FR-201, NFR-202
Parallel: No
Conflicts with: T018

目标:

更新 README 和中文 README，让项目第一屏、状态说明、安全边界和 MCP 能力与 v0.2 发布目标一致。

允许修改范围:

- `README.md`
- `README.zh-CN.md`
- `Cargo.toml`

Test targets:

- `tests/business_examples.rs`
- `tests/cli.rs`

交付内容:

- README 第一屏改为 `manifest-driven Agent skill capsule` 叙事。
- FastMCP 边界、MCP stdio server、`.skr` 限制和非 sandbox 边界表达清楚。
- Cargo description 如仍有 `tested MCP skill package` 误导则同步修正。

验收标准:

- README 不再暗示 v0.1 dry-run 即完整 MCP server。
- README 明确 v0.2 发布前仍是 unreleased development state。
- 中英文 README 语义一致。

Definition of Done:

- 文档 diff reviewed。
- `cargo test` 通过。
- evidence 记录 changed files、diff summary、validation result、residual risk。

验证命令:

- `cargo test`

TDD plan:

- RED: 如现有文档测试无法捕捉叙事漂移，先补充 README content assertion。
- GREEN: 更新 README/Cargo description。
- REFACTOR: 收敛重复状态说明。

Packet path:

- `.ai-platform/specs/v0.2/packets/T012.yaml`

完成证据:

- Changed files。
- RED/GREEN validation results。
- Diff summary。
- Residual risk。

## Epic E202: MCP Stdio Server

Goal:
把 `serve --mcp` 从 dry-run contract inspection 推进为真实 long-running MCP stdio server。

Stories:

- US-202
- US-203
- US-204

### T013: Add v0.2 MCP Protocol Contract Tests

Status: Accepted
Priority: P0
Depends on: T012
Blocks: T014, T015, T016, T017
Story / Requirement: US-202, US-203, US-204, FR-202, FR-203, FR-208, NFR-201, NFR-204
Parallel: No
Conflicts with: T014, T015, T016, T017

目标:

先用测试定义 v0.2 MCP stdio contract，包括 initialize、notifications/initialized、tools/list、tools/call、resources/list、resources/read 和 stdout/stderr discipline。

允许修改范围:

- `tests/mcp_server.rs`
- `tests/fixtures/`
- `.ai-platform/specs/v0.2/plan.md`，仅限记录测试发现的 protocol contract clarification

Test targets:

- `tests/mcp_server.rs`

交付内容:

- Scripted MCP client fixture。
- 初始 RED 测试，证明当前 `serve --mcp` 非 dry-run 仍未实现。
- 测试使用官方 MCP `2025-11-25` contract。

验收标准:

- 新测试能表达目标行为。
- RED 阶段失败原因是 server mode 未实现或缺少 protocol behavior，而不是测试 harness 错误。

Definition of Done:

- RED evidence 已记录。
- 测试 fixture 有 timeout，避免 hanging child process。
- 不修改 production MCP implementation。

验证命令:

- `cargo test --test mcp_server`

TDD plan:

- RED: 增加协议级测试并确认失败。
- GREEN: 本 task 不做 production GREEN；GREEN 由后续 T014-T016 完成。
- REFACTOR: 清理 fixture helper 命名。

Packet path:

- `.ai-platform/specs/v0.2/packets/T013.yaml`

完成证据:

- Changed files。
- RED command result。
- Fixture design summary。
- Residual risk。

### T014: Implement Long-running MCP Stdio Lifecycle

Status: Accepted
Priority: P0
Depends on: T013
Blocks: T015, T016, T017
Story / Requirement: US-202, FR-202, FR-203, NFR-201, NFR-204, NFR-205
Parallel: No
Conflicts with: T013, T015, T016, T017

目标:

实现 `skillrun serve --mcp` 的 stdio JSON-RPC loop、initialize 响应、initialized notification 处理和 graceful stdin EOF shutdown。

允许修改范围:

- `src/mcp.rs`
- `src/cli.rs`
- `src/main.rs`
- `tests/mcp_server.rs`

Test targets:

- `tests/mcp_server.rs`
- `tests/cli.rs`

交付内容:

- Non-dry-run `serve --mcp` 不再返回 not implemented。
- Server startup 前校验 Manifest。
- stdout 只输出 JSON-RPC messages。
- Unrecognized/invalid method 有 deterministic JSON-RPC error。

验收标准:

- initialize test pass。
- stale Manifest startup fail closed。
- dry-run 行为保留。

Definition of Done:

- RED/GREEN evidence 记录。
- `cargo test --test mcp_server` 通过。
- `cargo test --test cli` 通过。

验证命令:

- `cargo test --test mcp_server`
- `cargo test --test cli`
- `cargo test`

TDD plan:

- RED: 使用 T013 的 failing lifecycle tests。
- GREEN: 实现最小 server loop。
- REFACTOR: 提取 JSON-RPC helper，避免 handler 代码散落在 CLI。

Packet path:

- `.ai-platform/specs/v0.2/packets/T014.yaml`

完成证据:

- Changed files。
- RED/GREEN validation results。
- Diff summary。
- Residual risk。

### T015: Wire MCP Tools To SkillRun Runtime

Status: Accepted
Priority: P0
Depends on: T014
Blocks: T017
Story / Requirement: US-203, FR-204, FR-205, NFR-203
Parallel: No
Conflicts with: T014, T016, T017

目标:

实现 `tools/list` 和 `tools/call`，并确保 `tools/call` 复用现有 runtime + IPC，而不是复制 adapter 执行逻辑。

允许修改范围:

- `src/mcp.rs`
- `src/runtime.rs`
- `tests/mcp_server.rs`
- `tests/runtime.rs`

Test targets:

- `tests/mcp_server.rs`
- `tests/runtime.rs`

交付内容:

- `tools/list` 返回 Manifest-derived tool。
- `tools/call` 成功路径生成 run record。
- `tools/call` validation/policy error 映射为 MCP `isError: true` result。
- 不动态 import action 提取 metadata。

验收标准:

- Schema matches Manifest。
- Tool call success 和 structured error 都被测试覆盖。
- Existing CLI `run` / `test` 行为不回归。

Definition of Done:

- `cargo test --test mcp_server` 通过。
- `cargo test --test runtime` 通过。
- `cargo test` 通过。

验证命令:

- `cargo test --test mcp_server`
- `cargo test --test runtime`
- `cargo test`

TDD plan:

- RED: tools/list/tools/call tests fail against lifecycle-only server。
- GREEN: 最小 runtime wiring。
- REFACTOR: 提取 runtime API，同时保持 CLI behavior。

Packet path:

- `.ai-platform/specs/v0.2/packets/T015.yaml`

完成证据:

- Changed files。
- RED/GREEN validation results。
- Run record evidence。
- Diff summary。
- Residual risk。

### T016: Expose MCP Resources From Manifest

Status: Accepted
Priority: P0
Depends on: T014
Blocks: T017
Story / Requirement: US-204, FR-206, FR-207, NFR-203
Parallel: No
Conflicts with: T015, T017

目标:

实现 `resources/list` 和 `resources/read`，受控暴露 `SKILL.md` 和 examples，不暴露 run history 或 arbitrary files。

允许修改范围:

- `src/mcp.rs`
- `tests/mcp_server.rs`

Test targets:

- `tests/mcp_server.rs`

交付内容:

- `resources/list` 返回 `skillrun://...` URI。
- `resources/read` 返回 `SKILL.md` markdown text 和 examples JSON text。
- 未知 URI、path traversal、stale Manifest fail closed。

验收标准:

- Resource 读取不 import action。
- `.skillrun/runs/` 不可被读取。
- URI handling deterministic。

Definition of Done:

- `cargo test --test mcp_server` 通过。
- `cargo test` 通过。

验证命令:

- `cargo test --test mcp_server`
- `cargo test`

TDD plan:

- RED: resources/list/read tests fail before handler exists。
- GREEN: 添加最小 resource registry。
- REFACTOR: 与 dry-run contract 的 resource helper 复用。

Packet path:

- `.ai-platform/specs/v0.2/packets/T016.yaml`

完成证据:

- Changed files。
- RED/GREEN validation results。
- Diff summary。
- Residual risk。

### T017: Complete MCP E2E Fixture And Release Matrix

Status: Accepted
Priority: P0
Depends on: T015, T016
Blocks: T018
Story / Requirement: US-202, US-203, US-204, FR-208, NFR-201, NFR-204
Parallel: No
Conflicts with: T014, T015, T016, T018

目标:

把 MCP lifecycle、tools、resources 和 stdout/stderr discipline 纳入 release-level E2E coverage。

允许修改范围:

- `tests/mcp_server.rs`
- `tests/e2e_matrix.rs`
- `tests/fixtures/`
- `.ai-platform/specs/v0.2/analysis.md`

Test targets:

- `tests/mcp_server.rs`
- `tests/e2e_matrix.rs`

交付内容:

- Scripted client 完整跑通 initialize -> tools/list -> tools/call -> resources/list -> resources/read。
- stdout JSON-RPC discipline test。
- release matrix 增加 v0.2 MCP coverage。

验收标准:

- `cargo test --test mcp_server` 通过。
- `cargo test --test e2e_matrix` 通过。
- 子进程无 hanging。

Definition of Done:

- Fresh full validation。
- Analysis 更新为无 Critical/High findings。
- Evidence 完整。

验证命令:

- `cargo test --test mcp_server`
- `cargo test --test e2e_matrix`
- `cargo test`

TDD plan:

- RED: 缺失 release-level coverage 时测试失败或 analysis 标出 gap。
- GREEN: 补齐 fixture/E2E。
- REFACTOR: 去重 test helpers。

Packet path:

- `.ai-platform/specs/v0.2/packets/T017.yaml`

完成证据:

- Changed files。
- RED/GREEN validation results。
- Full validation result。
- Diff summary。
- Residual risk。

## Epic E203: Release Candidate

Goal:
把 v0.2 做成可发布的 public release candidate，并保留真实 evidence。

Stories:

- US-205

### T018: Prepare v0.2 Release Candidate

Status: Accepted
Priority: P0
Depends on: T012, T017
Blocks: Release decision
Story / Requirement: US-205, FR-209, NFR-202, NFR-205, NFR-206
Parallel: No
Conflicts with: T012, T017

目标:

更新版本、README 状态、release report 和 known limitations，准备 v0.2 public release candidate。

允许修改范围:

- `Cargo.toml`
- `Cargo.lock`，仅当 version update 导致必要变化
- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.2/analysis.md`
- `docs/ssot.md`
- `docs/mvp.md`，仅限说明 v0.1 未单独发布且 v0.2 是 public release candidate

Test targets:

- `tests/cli.rs`
- `tests/business_examples.rs`
- `tests/e2e_matrix.rs`

交付内容:

- Version 和 docs 一致。
- Release report 基于 T012-T017 evidence。
- Known limitations 不夸大安全或 package 能力。
- Release checklist 可供用户做最终发布决策。

验收标准:

- `cargo run -- --version` 显示目标版本。
- `cargo test` 通过。
- Release report 状态为 `Ready_For_User_Review`。

Definition of Done:

- Full validation passed。
- Release report ready for user review。
- 用户明确决定 publish / hold / revise 前，不打 tag。

验证命令:

- `cargo test`
- `cargo run -- --version`
- `cargo run -- serve --mcp --cwd examples/refund`，通过 scripted client 或测试 harness 验证

TDD plan:

- RED: 版本/docs/release report inconsistency checks，如已有测试不覆盖则补。
- GREEN: 更新 release docs/version。
- REFACTOR: 删除过时 v0.1 public release 语气。

Packet path:

- `.ai-platform/specs/v0.2/packets/T018.yaml`

完成证据:

- Changed files。
- Full validation results。
- Release report diff summary。
- Residual risk。

## User Review Gate

- Approval: Confirmed after Codex review on 2026-05-12
- Reviewer notes: No blocking findings. T012 is Ready after packet creation; T013-T018 remain dependency-gated Draft tasks.
