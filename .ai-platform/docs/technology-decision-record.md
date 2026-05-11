# SkillRun Technology Decision Record

Version: v0.1
Status: Confirmed
Last updated: 2026-05-11
Review: Rust-first correction approved by the user on 2026-05-11

## Decision Summary

SkillRun MVP 采用 Rust-first、Manifest-driven、file-based IPC 的本地 CLI/Core 架构。首轮交付只证明一个高质量 `refund` Skill Capsule 可以完成 `init -> manifest -> inspect -> test -> run -> serve --mcp -> pack`，并用 structured envelope、source hash、run record 和 instruction-only guard 建立可信边界。

Python `action.py` 是 MVP 的首个用户 Action adapter path，不是 SkillRun 本体实现语言。SkillRun 本体、CLI、Core、Manifest、IPC、MCP 和 `.skr` pack 均使用 Rust 实现。

## Constitution Check

- Constitution source: `.ai-platform/memory/constitution.md`
- Constitution status: Confirmed
- Relevant principles:
  - 项目名称使用 `SkillRun`，CLI 和 crate 名使用 `skillrun`。
  - 文档默认中文，关键术语、协议名、命令名和文件名可保留英文。
  - SkillRun 本体使用 Rust 实现；Python 只作为 MVP 首个 Action adapter 目标。
  - Consumer Mode 只读取 Manifest，stale 或 hash mismatch 时 fail closed。
  - 行为变更默认 RED-GREEN-REFACTOR。
  - stdout/stderr 只能作为日志，成功和失败结果必须来自 output/error envelope。
- Violations: None.

## Supporting Artifacts

- Product design: `.ai-platform/docs/product-design.md`
- Source MVP contract: `docs/mvp.md`
- Architecture baseline: `docs/ssot.md`
- Requirements checklist: `.ai-platform/docs/requirements-checklist.md`
- Test strategy: `.ai-platform/docs/test-strategy.md`
- Business examples: `docs/business-examples.md`
- Work graph: `.ai-platform/docs/tasks.md`
- Release report: `.ai-platform/docs/release-report.md`
- Execution packet directory: `.ai-platform/specs/mvp/packets/`

## TDR-001: Rust Core With Python Action Adapter

Decision:
MVP 实现 Rust CLI/Core，并只启用 Python `action.py` 作为首个可运行 Action adapter path。Rust Core 负责命令分发、Manifest 生成/读取、IPC 编排、run record、MCP exposure 和 `.skr` pack；Python 只存在于被运行的用户 action 和 metadata/run adapter 子进程中。

Requirement mapping:
- FR-001, FR-002, FR-004, FR-005
- NFR-003

Rationale:
- Rust 更适合交付一个本地优先、可分发、边界清晰的 CLI/Core。
- Python Action path 仍然保留，因为当前目标用户最容易用 Pydantic v2 表达业务输入输出 schema。
- 把 Core 与 Action 语言隔离，可以避免 SkillRun 退化为某个语言包或脚本 wrapper。

Alternatives considered:
- 用 Python 实现 SkillRun 本体：拒绝，用户已明确要求 SkillRun 改为 Rust，且项目不需要保留历史兼容。
- 同时支持 Node adapter：推迟到 post-MVP，避免在 Core、Manifest、IPC 和 package 尚未稳定时引入第二套语言语义。
- 通用 shell command runtime：推迟，避免 SkillRun 退化为任意脚本包装器。

Risks:
- Python metadata phase 仍然可能有作者侧副作用。
- Pydantic v2 版本差异可能导致 schema extraction 不稳定。

Mitigations:
- metadata phase 只在 Author Mode 自动执行，不注入 secrets，设置 timeout，并在文档中明确信任边界。
- 固定支持 Pydantic v2，并提供明确错误信息。

Task impact:
- T001 建立 Rust crate、binary 和 Rust CLI contract tests。
- T002 由 Rust CLI 生成 Python Action capsule template。
- T003 由 Rust Core 通过 Python metadata adapter 子进程生成 Manifest。
- T005 由 Rust Core 编排 Python Action adapter IPC success path。

## TDR-002: Manifest-driven Consumer Mode

Decision:
Consumer Mode 只读取已生成 Manifest，不为 metadata 重新 import source code；Manifest 缺失、stale 或 source hash 不匹配时 fail closed。

Requirement mapping:
- FR-002, FR-003, FR-007, FR-008
- NFR-001

Rationale:
- Manifest 是运行 IR，能让 MCP exposure、pack 和 inspect 共享同一份机器契约。
- fail closed 是 SkillRun 区分普通 action wrapper 的关键可信边界。

Alternatives considered:
- serve 时重新 import `action.py`：拒绝，这会破坏 Consumer Mode 安全模型。
- 允许 Manifest 自动修复 stale source：拒绝，消费场景不应隐式执行未信任代码。

Mitigations:
- `inspect` 和错误信息必须解释 source hash mismatch、下一步命令和风险原因。

Task impact:
- T003 写入 Manifest source hashes。
- T004 展示 Manifest 和 stale 信息。
- T008 实现 Consumer Mode guard。
- T009 和 T010 复用同一套 Manifest validation。

## TDR-003: File-based IPC And Structured Envelopes

Decision:
Rust Core 与 adapter 子进程通过 `SKILLRUN_CONTEXT_JSON`、`SKILLRUN_INPUT_JSON`、`SKILLRUN_OUTPUT_JSON`、`SKILLRUN_ARTIFACT_DIR` 进行 file-based IPC；stdout/stderr 只作为日志。

Requirement mapping:
- FR-004, FR-005, FR-006
- NFR-002, NFR-005

Rationale:
- 文件 IPC 简单、可审计、跨语言可迁移，并能自然保留 run evidence。
- output/error envelope 让 Agent 可恢复，而不是只能读人类日志。

Alternatives considered:
- 解析 stdout：拒绝，stdout 常混入日志，不能作为可信成功结果。
- 进程内调用 Action：拒绝作为 Consumer Mode 默认路径，因为会模糊 Core/Adapter 边界。

Task impact:
- T005 建立 run directory、context/input/output/log 文件和成功 envelope。
- T006 覆盖 `ValidationError`、`PolicyViolation`、`ProtocolViolation` 和 `RuntimeError`。
- T007 覆盖 env 注入、artifact 越界和 declared permissions evidence。

## TDR-004: Instruction-only Skill Guard

Decision:
没有显式 `action.py` 和有效 Manifest 的 Skill 目录必须保持 instruction-only；SkillRun 不从 Markdown、scripts、references、assets 或 examples 猜测可执行入口。

Requirement mapping:
- FR-009

Rationale:
- 保护 Skill 生态中认知层和运行层分离。
- 避免把普通 Skill 目录变成潜在可执行目录。

Task impact:
- T004 覆盖 inspect 的 instruction-only 状态。
- T008 覆盖 manifest、run、serve 和 pack 的拒绝路径。

## TDR-005: Rust Repository Layout And CLI Stack

Decision:
使用标准 Cargo binary crate。项目入口为 `Cargo.toml` 和 `src/main.rs`；后续 Rust modules 按职责扩展到 `src/cli.rs`、`src/manifest.rs`、`src/runtime.rs`、`src/consumer.rs`、`src/mcp.rs`、`src/pack.rs` 等。测试使用 Rust integration tests，放在 `tests/*.rs`。T001 不引入 CLI framework，先用 Rust stdlib 交付 help/version skeleton；后续如确有必要可引入 `clap`。

Requirement mapping:
- FR-001 through FR-008
- NFR-006

Rationale:
- Cargo binary crate 是 Rust CLI 的清晰默认形态。
- stdlib CLI skeleton 足够支撑 T001，减少首个任务依赖面。
- Rust integration tests 可以直接验证真实 `skillrun` binary 行为。

Alternatives considered:
- 继续使用 Python `src/` layout：拒绝，已经与用户要求冲突。
- T001 引入 `clap`：暂缓，当前 skeleton 不需要复杂 CLI ergonomics。

Task impact:
- T001 负责 `Cargo.toml`、`src/main.rs`、`tests/cli.rs`、`README.md`。
- 后续 task 只在各自 allowed files 中扩展 Rust modules。

## TDR-006: Thin MCP Layer

Decision:
`serve --mcp` 是 Manifest-driven thin adapter。MCP tool schema、tool description 和 resource 内容全部来自 Manifest 和 `SKILL.md`；核心测试优先验证 Rust contract builder，MCP SDK 集成保持在独立模块。

Requirement mapping:
- FR-007
- NFR-001

Rationale:
- MCP 行为变化不应污染 Core、Manifest、IPC 和 pack 的稳定性。
- Thin layer 能证明 Agent exposure 价值，同时保留 post-MVP 替换或扩展空间。

Task impact:
- T009 只修改 MCP server、CLI serve command 和相关 tests，不改变 Manifest 或 runtime contract。

## TDR-007: Package Primitive

Decision:
`skillrun pack` 生成 `.skr` tar.gz archive，包含 source files、examples 和 `.skillrun/manifest.generated.yaml`，不包含 `.skillrun/runs/`，也不承诺依赖 vendoring 或 reproducible runtime。

Requirement mapping:
- FR-008
- NFR-004

Rationale:
- `.skr` 是首个分发原语，和 Git repo 协作形态分开。
- 不打包依赖环境能保持 MVP scope 清晰，并诚实表达安全和供应链边界。

Task impact:
- T010 实现 pack preflight、archive 内容过滤、README summary 和 unpack/inspect 测试。

## TDR-008: Layered Test Strategy And Business Examples

Decision:
MVP 测试不再只使用 A001-A013 简表。A001-A013 作为 release acceptance gate 保留，但必须下钻到 Unit、Contract、Integration、Negative/Security、E2E Acceptance 和 Business Examples 六层测试。业务价值证明使用 B001-B004，其中 B001 `refund` 是完整可运行 hero example，B002-B004 是 README/docs 级经典示例，不扩大 v0.1 runtime scope。

Requirement mapping:
- FR-001 through FR-009
- NFR-001 through NFR-006

Rationale:
- A001-A013 能说明要验收什么，但不能单独说明如何防止可信边界漏测。
- SkillRun 的价值不只是 CLI passing，必须证明 SOP、Agent recovery、artifact 和 package distribution 的组合价值。

Task impact:
- T001-T010 各自覆盖对应 Unit、Contract、Integration 和 Negative/Security case。
- T011 负责 release-level E2E acceptance、B001 hero example、B002-B004 docs narrative 和 traceability summary。

## Consequences For Tasks

- Work graph 按 milestone 拆成 T001-T011，每个任务都对应 feature slice 和 validation command。
- 所有行为任务默认先写 failing tests，再实现最小 Rust code。
- T009 MCP 和 T010 Pack 依赖 T003 Manifest 和 T008 Consumer guard。
- T011 作为端到端收口任务，负责高质量 `refund` 示例、README、A001-A013、Negative/Security Matrix 和 B001-B004 业务示例验证。

## User Review Gate

- Approval: Approved on 2026-05-11
- Reviewer notes: 用户明确纠正 SkillRun 本体必须使用 Rust；本 TDR 已移除 Python package 作为实现方案的历史包袱。
