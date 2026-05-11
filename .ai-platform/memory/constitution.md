# SkillRun Constitution

Version: v0.1
Status: Confirmed
Last updated: 2026-05-11
Review: Approved by user on 2026-05-11

## 1. Purpose

本 constitution 记录 SkillRun MVP 交付中的不可协商原则。它适用于产品文档、技术计划、任务拆分、实现、测试、review、release report 和后续 agent handoff。

## 2. Principles

### 2.1 Product Identity And Language

- 项目名称使用 `SkillRun`；CLI、crate、命令和代码标识使用 `skillrun`。
- 不给项目名追加 `v2` 后缀；`v0.1.0` 等版本号只用于 release 或 artifact 版本。
- 项目文档默认使用中文；关键术语、命令、API 名称、状态名、文件名和协议名可以保留英文。

### 2.2 Quality

- 交付必须围绕已确认的 MVP contract：用 Rust 实现的 SkillRun，把一份 SOP 和一个显式 Action 生成可测试、可检查、可运行、可给 Agent 调用、可打包分发的 Skill Capsule。
- 优先保持小而硬的边界，不把 MVP 扩展成通用 Agent framework、FastMCP 替代品、OpenAPI wrapper、marketplace 或完整 sandbox。
- 不通过 placeholder diff、未运行测试、乐观总结或推断用户接受来声明完成。
- 文档和代码变更应最小化无关 churn，并保持稳定文件名、heading 和状态字段，方便后续 agent 解析。

### 2.3 Architecture

- `SKILL.md` 是认知契约，不是 runtime config 或脚本入口。
- SkillRun 本体、CLI、Core、Manifest、IPC、MCP 和 pack 路径必须使用 Rust 实现。
- `action.py` 是 MVP 唯一 blessed user Action entrypoint；Node、OpenAPI、多 action 编排和 GUI 均为 post-MVP。
- `Manifest` 是运行 IR，不是用户主入口；Author Mode 可以生成 Manifest，Consumer Mode 只读取 Manifest 并 fail closed。
- Core 只认 Manifest、IPC、权限声明和 run/artifact 边界；语言知识放在 Adapter，开发者体验放在 SDK。
- stdout/stderr 只能作为日志；结构化结果必须通过 output/error envelope 写入 `SKILLRUN_OUTPUT_JSON`。
- Artifact 是一等公民，必须在 output envelope 中声明，并被限制在 run-local artifact directory 内。

### 2.4 Testing

- 行为变更默认使用 RED-GREEN-REFACTOR；例外需要用户明确批准并写入 execution packet 或 evidence。
- 测试覆盖必须至少证明 `ValidationError`、`PolicyViolation`、`ProtocolViolation`、Manifest stale fail closed、artifact path 边界和 instruction-only Skill guard。
- 每个可执行 task 必须有 validation commands、expected evidence 和真实命令结果。
- MVP 默认支持 Pydantic v2；版本差异必须用明确错误信息暴露，而不是静默降级。

### 2.5 UX And Developer Experience

- 首个用户路径必须支持 `skillrun init -> manifest -> inspect -> test -> run -> serve --mcp -> pack`。
- `inspect` 输出必须让用户不阅读 Manifest 也能理解核心能力、schema、权限、adapter 和风险。
- CLI 错误应面向 Agent 和开发者可恢复，优先返回结构化错误码、message、recoverable 和可选 `llm_hint`。
- 文档和命令示例优先围绕一个高质量 `refund` capsule。

### 2.6 Performance And Reliability

- metadata phase 只在 Author Mode 自动执行，不注入用户 secrets，并必须有 timeout。
- 每次 run 都必须有唯一 run id、run record、input、context、output、stdout、stderr 和 artifact 目录。
- Consumer Mode 在 Manifest 缺失、source hash 不匹配或 stale 时必须 fail closed。

### 2.7 Security, Privacy, Compliance

- MVP 不承诺完整 OS sandbox，文档、CLI warning、pack/install 流程必须诚实表达该边界。
- 只向子进程注入 Manifest 声明过的 env。
- permissions、source hashes、skill hash、manifest hash 和 action hash 必须进入 inspect 或 run evidence。
- Consumer Mode 不为提取 metadata 动态 import 未信任源码。

## 3. Git And Review Policy

- 修改前检查 worktree status，保护用户已有变更。
- 实现 task 前必须有 Confirmed spec、Completed checklist、Ready execution packet、analysis gate 和明确 allowed files。
- 非平凡实现任务优先 delegated execution；主 agent 扮演 Delivery Orchestrator，worker 只执行一个受限 attempt。
- task 进入 `Accepted` 前必须完成 validation、spec compliance review、bug/code-quality review、QA acceptance review，并由用户明确接受。

## 4. Change Process

- Constitution 变更必须先以 Draft 提出，并经用户明确批准后生效。
- 若 constitution 与 spec、plan、task 或 implementation 冲突，冲突是 Critical blocker。
- MVP scope 变更必须回到 `docs/mvp.md` 或对应 feature spec 重新审核。

## 5. Exceptions

- None.

## User Review Gate

- Approval: Approved on 2026-05-11
- Reviewer notes: 用户批准本 constitution 作为 SkillRun MVP 的 blocking policy 生效。
