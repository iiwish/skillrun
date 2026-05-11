# SkillRun Product Design Contract

Version: v0.1
Status: Confirmed
Source: `docs/mvp.md`
Last updated: 2026-05-11
Review: Approved by user on 2026-05-11

## 1. 产品定位

SkillRun 是用 Rust 实现的本地 CLI/Core，把一份 SOP 和一个显式 Action 编译成 Manifest 驱动的 Skill Capsule，让业务经验从文档和代码的松散组合，变成可测试、可检查、可运行、可分发、可给 Agent 调用的技能单元。

## 2. 目标用户

- 熟悉命令行、SOP 工程化和 Python Action 编写的 AI engineer / platform engineer。
- 已经有团队 SOP，并有或愿意写一个可执行 action 的开发者。
- 希望把能力复用给 Agent、MCP client、CI 或其他本地流程的平台团队。

## 3. User Stories And Scenarios

### US-001: 初始化 Python Action Capsule

As an AI engineer, I want the Rust CLI to create a standard Python Action capsule from one command, so that I can start from a runnable SOP-backed skill skeleton.

Scenario:
1. 用户运行 `skillrun init refund --python`。
2. 系统生成 `SKILL.md`、`action.py`、`examples/default.input.json` 和可选 `skillrun.config.json`。
3. 用户无需新增文件即可继续生成 Manifest。

### US-002: 生成并查看 Manifest 合同

As a platform engineer, I want to generate and inspect the skill Manifest, so that I can understand schema、permissions、adapter、SOP hash 和 MCP tool 摘要。

Scenario:
1. 用户运行 `skillrun manifest`。
2. 系统生成 `.skillrun/manifest.generated.yaml` 并记录 source hashes。
3. 用户运行 `skillrun inspect`，看到核心契约和风险，不需要直接阅读 Manifest。

### US-003: 可控运行和测试 Capsule

As an AI engineer, I want to run the capsule through file-based IPC and structured envelopes, so that stdout cannot be mistaken for successful business output.

Scenario:
1. 用户运行 `skillrun test` 或 `skillrun run --input examples/default.input.json`。
2. 系统创建 run record、input、context、output、stdout、stderr 和 artifact 目录。
3. 成功返回 `ok: true` envelope；失败返回 `ok: false` error envelope。

### US-004: 从 Manifest 暴露 MCP tool

As a platform engineer, I want MCP exposure to come only from Manifest, so that Consumer Mode does not import untrusted source code for metadata.

Scenario:
1. 用户运行 `skillrun serve --mcp`。
2. 系统从 Manifest 暴露 tool 和 `SKILL.md` resource。
3. Manifest stale 时 fail closed。

### US-005: 打包分发 Capsule

As a skill author, I want to pack the capsule into `.skr`, so that a skill package can move independently from the development repository.

Scenario:
1. 用户运行 `skillrun pack`。
2. 系统验证 Manifest source hashes。
3. 系统生成 `dist/refund-0.1.0.skr`，包含 Manifest 和 source files，不包含 run history。

### US-006: 保护 instruction-only Skill

As a skill ecosystem maintainer, I want instruction-only Skill directories to remain non-executable, so that SkillRun does not turn Markdown or scripts into hidden runtime behavior.

Scenario:
1. 目标目录只有 `SKILL.md`、`references/`、`assets/` 或 `scripts/`。
2. `skillrun inspect` 展示 instruction-only 状态。
3. `skillrun run`、`skillrun serve --mcp` 和 `skillrun pack` 拒绝执行或分发。

## 4. 核心用户旅程

1. 用户运行 `skillrun init refund --python` 创建标准 capsule。
2. 用户编辑 `SKILL.md` 和 `action.py`。
3. 用户运行 `skillrun manifest` 生成 Manifest。
4. 用户运行 `skillrun inspect` 检查 SOP hash、schema、permissions、adapter 和 tool 摘要。
5. 用户运行 `skillrun test` 和 `skillrun run --input examples/default.input.json` 验证结构化 output、run record 和 artifacts。
6. 用户运行 `skillrun serve --mcp` 给 Agent 暴露 tool。
7. 用户运行 `skillrun pack` 生成 `.skr` 分发包。

## 4.1 经典业务价值示例

MVP 的完整可运行示例是 `refund`，用于证明 SkillRun 能把退款 SOP、typed input/output、preflight、structured error、run record 和 packaging 串成一条可信路径。

另外三个经典示例只作为 README 或 `docs/business-examples.md` 中的业务说明，不进入 v0.1 实现范围：

- `support_triage`: 证明 SOP summary、stable routing labels 和 missing-context recovery 可以减少 Agent 自由分流。
- `access_request_approval`: 证明 approval boundary、declared env 和 audit note 对权限类工作有价值。
- `vendor_risk_review`: 证明 artifact 和 risk summary 能支持审核型业务，而不是只返回 stdout。

## 5. Functional Requirements

- FR-001: `skillrun init refund --python` 必须由 Rust CLI 生成可运行 Python Action capsule skeleton。
- FR-002: `skillrun manifest` 必须从 `SKILL.md`、`action.py`、可选 `skillrun.config.json` 和 examples 生成 Manifest，并写入 source hashes。
- FR-003: `skillrun inspect` 必须展示 skill name、SOP hash、SOP summary、input/output schema、adapter、runtime command、permissions、examples、preflight 状态和 MCP tool description 摘要。
- FR-004: `skillrun test` 必须使用默认 example 创建 test run，执行 `preflight` 和 `run`，校验 output envelope、artifact path 和 run record。
- FR-005: `skillrun run --input examples/default.input.json` 必须校验 input schema，创建 run-local IPC 文件和 artifacts，注入声明过的 env，并校验 result schema。
- FR-006: MVP 必须支持 `ValidationError`、`PolicyViolation`、`PermissionDenied`、`ProtocolViolation` 和 `RuntimeError` error envelope。
- FR-007: `skillrun serve --mcp` 必须只从 Manifest 暴露 tool 和 `SKILL.md` resource，不重新 import `action.py` 提取 schema。
- FR-008: `skillrun pack` 必须生成 `.skr` tar.gz archive，包含 Manifest 和 source hashes，不包含 `.skillrun/runs/`。
- FR-009: instruction-only Skill 必须被识别为不可运行 capsule，不得从 Markdown、scripts 或 examples 猜测 Action。

## 6. Non-Functional Requirements

- NFR-001: Consumer Mode 在 Manifest 缺失、stale 或 source hash 不匹配时必须 fail closed。
- NFR-002: stdout/stderr 只能作为日志，不能作为成功结果兜底。
- NFR-003: metadata phase 只在 Author Mode 自动执行，不注入用户 secrets，并必须有 timeout。
- NFR-004: MVP 必须诚实表达不提供完整 OS sandbox、签名验证、供应链隔离或依赖 vendoring。
- NFR-005: 每次 run 必须生成唯一 run id 和可审计 run record。
- NFR-006: 项目文档默认中文，关键术语、命令、API、状态和协议名可以保留英文。

## 7. 功能范围

- Rust-first SkillRun runtime with Python Action adapter path。
- Pydantic v2 schema extraction。
- Manifest generation、inspect、test、run、serve as MCP、pack。
- File-based IPC、structured output/error envelope、run record 和 artifact policy。
- 一个高质量 `refund` capsule 示例。
- Instruction-only Skill guard。

## 8. 非目标

- Node adapter。
- OpenAPI-to-MCP。
- MCP proxy / MCP composition。
- HTTP server。
- schedule / workflow。
- marketplace / registry。
- signed package。
- dependency vendoring。
- reproducible runtime image。
- 完整 sandbox。
- 多 action 编排。
- GUI。
- `.run.md` authoring sugar。
- 从 examples 反推高质量 schema。
- 让自然语言 SOP 自动变成硬约束。

## 9. Edge Cases

- 输入缺字段或类型错误时返回 `ValidationError`。
- preflight 或业务规则拒绝时返回 `PolicyViolation`。
- adapter 未写 output 或 output 非法时返回 `ProtocolViolation`。
- artifact path 越界时返回 `PermissionDenied` 或 `ProtocolViolation`。
- Manifest stale 时 Consumer Mode fail closed。
- instruction-only Skill 不得运行、serve 或 pack。

## 10. 约束与假设

- MVP 仅支持 Rust Core + Python Action blessed path。
- `action.py` 顶层应 side-effect free。
- `.skr` 是 skill package，不是包含依赖环境的 reproducible runtime。
- MCP 层保持薄，核心测试不依赖 MCP 行为细节。
- 项目名称是 `SkillRun`；CLI 和 crate 名使用 `skillrun`。

## 11. 数据与集成需求

- 读取和写入本地 capsule 文件、`.skillrun/manifest.generated.yaml`、`.skillrun/runs/`、`.skillrun/artifacts/` 和 `dist/*.skr`。
- Rust Core 通过 Pydantic v2 metadata 子进程从 Python action 提取 input/output JSON Schema。
- Rust Core 通过 file-based IPC 与 Python Action adapter 子进程通信。
- 通过 MCP server 暴露 tool 和 resource。

## 12. Success Criteria

- SC-001: 新用户在 5 分钟内完成 `init -> test`。
- SC-002: 示例 capsule 在干净环境中可重复生成 Manifest。
- SC-003: MCP tool schema 只来自 Manifest。
- SC-004: stdout 不会被当成成功输出。
- SC-005: Manifest stale 时 Consumer Mode fail closed。
- SC-006: `.skr` 能作为独立分发 artifact 被 inspect。
- SC-007: 文档能让用户理解 SkillRun 不是 FastMCP 替代品。

## 13. 验收标准

- `.ai-platform/docs/test-strategy.md` 中的 A001 到 A013 全部通过，并具备 fresh command evidence。
- 默认 `refund` capsule 可以被初始化、测试、运行、MCP 暴露和打包。
- Consumer Mode 不从源码重新提取 metadata。
- `ProtocolViolation`、`ValidationError`、`PolicyViolation` 至少各有一个可复现测试。
- Negative/Security Matrix 中的高风险边界有 automated tests 或明确 documented exception。
- B001 `refund` hero example 完整实现；B002-B004 作为 README 或 docs 中的经典业务示例说明。
- `pack` 生成的 `.skr` 可被解包并通过 Manifest inspect。
- README 可以用同一条叙事解释项目：`Turn one SOP and one action into a tested MCP skill package with a Rust CLI/Core.`

## 14. Clarifications

- Q: 是否批准 `docs/mvp.md` 当前 v0.1 MVP contract 进入 Plan / Work Graph 阶段？
  A: 2026-05-11 用户批准 `docs/mvp.md`。
- Q: 项目命名是否使用 SkillRun 还是带 v2 后缀的名称？
  A: 2026-05-11 用户明确项目仍叫 SkillRun，CLI 和 crate 名使用 `skillrun`。
- Q: Codex 生成项目文档的默认语言是什么？
  A: 2026-05-11 用户要求文档默认中文，关键术语可以使用英文。
- Q: SkillRun 本体使用什么语言实现？
  A: 2026-05-11 用户明确要求 SkillRun 本体改为 Rust；Python 只作为 MVP 首个 Action adapter 目标。

## 15. 开放问题

- None.

## 16. User Review Gate

- Approval: Approved on 2026-05-11
- Reviewer notes: 用户批准 `docs/mvp.md`，并补充项目命名与中文文档语言约定。
