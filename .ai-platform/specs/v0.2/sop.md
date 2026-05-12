# SkillRun v0.2 Development SOP

Version: v0.2-sop
Status: Confirmed
Source: User request on 2026-05-12
Last updated: 2026-05-12
Review: Codex review passed on 2026-05-12; user authorized confirmation after review

## 1. 一句话判断

v0.2 不应该横向扩功能；它应该把 SkillRun 从 “MVP contract proof” 推到 “可公开发布的最小真实产品”。

核心目标：

> **完成 README/发布叙事修正，并把 `serve --mcp --dry-run` 推进为真实 long-running MCP stdio server；然后再发布。**

## 2. v0.2 北极星

v0.1 已证明 SkillRun 的内部契约成立：

- Skill Capsule 可以生成 Manifest。
- Runtime 可以通过 IPC 执行 Python Action。
- stdout 不被当成结构化成功。
- Manifest stale 时 Consumer Mode fail closed。
- `.skr` 可以作为 source + Manifest archive。
- MCP contract 可以从 Manifest dry-run 渲染。

v0.2 要证明的是外部采用路径成立：

> **一个用户可以把 refund capsule 暴露成真实 MCP server，让 MCP client 通过 Manifest-derived tool 调用它，同时 README 不夸大安全、包管理或 MCP 完成度。**

这比新增 Node adapter、registry、install、sandbox 或 marketplace 更重要。

## 3. 版本定位

v0.2 是第一版 public release candidate。

发布名称建议：

```text
SkillRun v0.2.0 - Manifest-driven SOP-backed skills with real MCP stdio serving
```

不要把它叫成：

- stable runtime
- secure skill package manager
- marketplace preview
- general agent framework
- FastMCP replacement

## 4. 必须坚持的边界

v0.2 继续继承 v0.1 Constitution 中的原则：

- SkillRun 本体继续使用 Rust。
- Python `action.py` 仍是唯一 blessed Action adapter。
- Manifest 仍是运行 IR，不是用户主入口。
- Consumer Mode 不为 metadata 动态 import 未信任源码。
- stdout/stderr 只作为日志。
- `.skr` 不承诺 signed package、dependency vendoring 或 reproducible runtime。
- permissions 不等于 sandbox。

v0.2 明确不做：

- Node adapter。
- Streamable HTTP / SSE / remote MCP hosting。
- MCP auth。
- `.skr install`、registry、marketplace。
- signed package。
- dependency vendoring。
- OS sandbox。
- 多 action 编排。
- GUI。
- OpenAPI-to-MCP。
- `.run.md` authoring sugar。

## 5. v0.2 Scope

### 5.1 P0 Scope

P0 只包含公开发布前必须完成的内容。

1. README 和发布叙事修正
   - 第一屏改成 `manifest-driven Agent skill capsule`。
   - 明确 FastMCP 和 SkillRun 的边界。
   - 明确 v0.2 的 MCP 能力是真实 stdio server。
   - 明确 `.skr` 是 source + Manifest archive。
   - 明确 v0.2 仍然不是 sandbox。

2. 真实 MCP stdio server
   - `skillrun serve --mcp --cwd examples/refund` 这类命令可以启动 long-running server。
   - server 只从 Manifest 暴露 tool/schema/resource。
   - tool call 通过现有 Rust runtime + IPC 执行 action。
   - Manifest stale 或缺失时启动前 fail closed。
   - stdio JSON-RPC 输出不得与日志混用；日志必须走 stderr 或文件。

3. MCP 协议兼容验证
   - 实现前必须对照官方 MCP specification 的当前 stdio transport、tools 和 resources contract。
   - 测试覆盖 initialize、tools/list、tools/call、resources/list、resources/read 的最小路径。
   - 至少有一个 scripted MCP client fixture 或协议级 integration test。

4. Release hygiene
   - 版本号、README、中文 README、release report、known limitations 一致。
   - `cargo test` 全量通过。
   - public release notes 明确 v0.2 能做什么和不能做什么。

### 5.2 P1 Scope

P1 只有在 P0 完成且不会延迟发布时才允许进入。

- `skillrun inspect --json`，用于 issue/CI/MCP contract diff。
- 更清晰的 MCP server startup diagnostics。
- README 中加入一个最小 MCP client 配置示例。

P1 不得挤占 P0。

## 6. Codex 开发工作流

项目继续采用 Codex-first governed delivery。

### 6.1 Issue 进入条件

每个 v0.2 issue 必须包含：

- Problem：要解决的真实问题。
- User-visible outcome：用户完成后能观察到什么。
- Non-goals：明确不做什么。
- Affected surface：CLI、runtime、MCP、docs、tests、release。
- Expected files：可能修改的文件或目录。
- Acceptance criteria：可验证结果。
- Validation commands：至少包含相关测试命令。
- Risk notes：是否触碰 Manifest、IPC、MCP protocol、安全边界或发布叙事。

没有这些字段的 issue 只能进入 `needs-clarification`，不能直接编码。

### 6.2 PR 规则

- 一个 PR 只处理一个 governed task。
- PR 标题建议：

```text
T012: Rewrite README release narrative
T013: Implement MCP stdio server loop
```

- 分支使用 `codex/` 前缀，例如：

```text
codex/v0.2-readme-narrative
codex/v0.2-mcp-stdio-server
```

- PR 必须包含：
  - task ID。
  - changed files。
  - validation commands and results。
  - behavior notes。
  - residual risks。
  - screenshots/log snippets only when relevant。

### 6.3 Codex 执行纪律

Codex 不应从模糊 issue 直接写代码。

执行顺序：

1. 读取 `AGENTS.md`、`docs/ssot.md`、`docs/mvp.md`、本 SOP 和相关 task。
2. 检查 `git status --short`。
3. 确认 task 状态、allowed files、validation commands。
4. 行为变更默认 TDD：RED -> GREEN -> REFACTOR。
5. 只修改 task 允许范围。
6. 运行 fresh validation。
7. 写 evidence。
8. 做 spec compliance review 和 bug/code-quality review。
9. 等用户或 maintainer 明确接受。

## 7. Human Review Gates

v0.2 必须经过这些人工闸门。

1. SOP approval
   - 本文件从 `Draft` 进入 `Confirmed`。

2. v0.2 Spec approval
   - 创建 `.ai-platform/specs/v0.2/spec.md`。
   - 明确 requirements、non-goals、acceptance criteria。
   - 用户批准后才能进入 plan/task。

3. Plan and task approval
   - 创建 `.ai-platform/specs/v0.2/plan.md` 和 `tasks.md`。
   - 任务全部保持 `Draft`，直到用户批准 work graph。

4. Packet gate
   - 每个 implementation task 必须有 packet：

```text
.ai-platform/specs/v0.2/packets/T0XX.yaml
```

5. Analysis gate
   - 执行前创建 `.ai-platform/specs/v0.2/analysis.md`。
   - Critical/High findings 未处理前不得执行。

6. Release gate
   - release report 进入 `Ready_For_User_Review` 后，由用户决定发布。

## 8. 建议任务序列

以下是 Draft task sequence，不是 approved work graph。

### T012: Rewrite README Release Narrative

Status: Draft
Priority: P0
Depends on: SOP approval

目标：

- 把 README 第一屏从 “tested MCP skill package” 改成 “manifest-driven Agent skill capsule”。
- 同步更新 `README.zh-CN.md`。
- 明确 v0.2 发布前不会公开发布。

允许修改范围：

- `README.md`
- `README.zh-CN.md`
- `Cargo.toml` description，如需要

验证命令：

- `cargo test`

### T013: Define MCP v0.2 Protocol Contract

Status: Draft
Priority: P0
Depends on: T012

目标：

- 对照官方 MCP specification，写出 SkillRun v0.2 最小 MCP contract。
- 明确 stdio transport、JSON-RPC framing、tool/list、tool/call、resource/list、resource/read。
- 明确日志不得写 stdout。

允许修改范围：

- `.ai-platform/specs/v0.2/spec.md`
- `.ai-platform/specs/v0.2/plan.md`
- `docs/ssot.md`，仅在需要记录长期架构决策时

验证命令：

- 文档 review。
- 后续 implementation task 必须引用该 contract。

### T014: Implement Long-running MCP Stdio Server

Status: Draft
Priority: P0
Depends on: T013

目标：

- `skillrun serve --mcp --cwd examples/refund` 这类命令可以启动真实 stdio server。
- 支持 MCP initialize 和 capabilities 响应。
- 不再只返回 dry-run contract。

允许修改范围：

- `src/mcp.rs`
- `src/cli.rs`
- `src/main.rs`
- `tests/mcp_server.rs`

验证命令：

- `cargo test --test mcp_server`
- `cargo test`

### T015: Wire MCP Tool Calls To Runtime

Status: Draft
Priority: P0
Depends on: T014

目标：

- MCP `tools/list` 从 Manifest 返回 tool schema。
- MCP `tools/call` 调用现有 runtime，返回 output/error envelope。
- Manifest stale 时 fail closed。

允许修改范围：

- `src/mcp.rs`
- `src/runtime.rs`，仅限必要接口提取
- `tests/mcp_server.rs`
- `tests/runtime.rs`，如需补 contract

验证命令：

- `cargo test --test mcp_server`
- `cargo test --test runtime`
- `cargo test`

### T016: Expose MCP Resources From Manifest

Status: Draft
Priority: P0
Depends on: T014

目标：

- MCP `resources/list` 暴露 `SKILL.md` 和 examples resource。
- MCP `resources/read` 返回受控内容。
- 不动态 import action。

允许修改范围：

- `src/mcp.rs`
- `tests/mcp_server.rs`

验证命令：

- `cargo test --test mcp_server`
- `cargo test`

### T017: Add MCP Client Fixture And E2E Coverage

Status: Draft
Priority: P0
Depends on: T015, T016

目标：

- 增加 scripted MCP client fixture，覆盖 initialize、tools/list、tools/call、resources/list、resources/read。
- 确认 stdout 只承载 JSON-RPC messages。
- 确认 stderr/log 文件承载诊断信息。

允许修改范围：

- `tests/mcp_server.rs`
- `tests/e2e_matrix.rs`
- `tests/fixtures/`，如需要

验证命令：

- `cargo test --test mcp_server`
- `cargo test --test e2e_matrix`
- `cargo test`

### T018: Prepare v0.2 Release Candidate

Status: Draft
Priority: P0
Depends on: T012, T017

目标：

- 更新版本、README、release report、known limitations。
- 确保 v0.2 对外承诺与实现一致。
- 生成 release checklist。

允许修改范围：

- `Cargo.toml`
- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.2/analysis.md`
- `docs/` 中与 release 直接相关的文件

验证命令：

- `cargo test`
- `cargo run -- --version`
- `cargo run -- serve --mcp --cwd examples/refund`，通过 scripted client 或测试 harness 验证

## 9. v0.2 Definition of Done

v0.2 只有同时满足以下条件才可发布：

- README 第一屏能在 30 秒内解释为什么 SkillRun 不是 FastMCP。
- `skillrun serve --mcp --cwd examples/refund` 是真实 long-running MCP stdio server。
- MCP tool schema 来自 Manifest。
- MCP tool call 通过现有 runtime 执行，不绕过 IPC。
- MCP resource 读取不 import action。
- Manifest stale 时 server fail closed。
- stdout 不混入非 JSON-RPC 日志。
- `.skr` 文档仍然诚实说明不是 signed package、不是 sandbox、不是 dependency bundle。
- `cargo test` 通过。
- v0.2 release report 基于真实 evidence，而不是乐观总结。

## 10. 发布后 issue/PR 机制

v0.2 发布后再进入持续演进。

推荐 labels：

- `type:docs`
- `type:mcp`
- `type:runtime`
- `type:manifest`
- `type:packaging`
- `type:security-boundary`
- `type:dx`
- `scope:v0.2`
- `scope:post-v0.2`
- `needs-clarification`
- `ready-for-codex`
- `blocked`

Issue 到 PR 的状态流：

```text
idea
-> needs-clarification
-> spec-ready
-> ready-for-codex
-> in-progress
-> needs-review
-> accepted
```

Codex 只处理 `ready-for-codex` issue。没有 task ID、allowed files 和 validation commands 的 issue 不应进入实现。

## 11. 推荐的下一步

1. 用户审阅本 SOP。
2. 若同意，将本 SOP 状态改为 `Confirmed`。
3. 创建 `.ai-platform/specs/v0.2/spec.md`，只描述 v0.2 P0 scope。
4. 创建 v0.2 checklist、plan、tasks 和 packets。
5. 先执行 T012 README 叙事修正。
6. 再执行 MCP stdio server 主线。

## 12. Resolved Decisions

1. v0.2 只支持 MCP stdio transport；Streamable HTTP / SSE / remote hosting 均为 post-v0.2。
2. v0.2 release gate 以协议级 scripted MCP client fixture 为必须项；具体 MCP client 手动验证可以作为 release note 附加证据，但不是 blocking gate。
3. v0.2 作为第一个 public release candidate；不单独公开发布 v0.1。
4. README 以英文为主，`README.zh-CN.md` 保持完整中文镜像；治理细节继续放在 `.ai-platform/` 和 `docs/`。

## 13. SOP Review Notes

- Blocking findings: None.
- Medium findings resolved: 将 MCP 目标限定为 stdio transport；将协议版本锚定为 current official latest `2025-11-25`，并要求 implementation task 在执行前再次核对官方 specification。
- Residual risk: MCP specification 可能继续演进；v0.2 implementation packet 必须记录实际对照的官方 protocol version 和 source URL。
