# SkillRun Test Strategy

Version: v0.1
Status: Confirmed
Last updated: 2026-05-11

## 测试目标

SkillRun 的测试目标不是“命令能跑”，而是证明：

> 用 Rust 实现的 SkillRun Core 可以把一份 SOP 和一个显式 Action 编译成可信的 Skill Capsule，并在本地、CI、MCP client 和分发包中保持同一份机器契约。

Python `action.py` 是 MVP 的首个 Action adapter 目标；所有 SkillRun 本体测试默认验证 Rust binary 和 Rust Core 行为。

## 分层策略

| Layer | 目的 | Rust 测试目标 | 漏测风险 |
| --- | --- | --- | --- |
| Unit | 验证纯函数和边界判断 | `src/*.rs` unit tests | 局部逻辑不可信 |
| Contract | 验证 CLI、Manifest、envelope、MCP、pack 的外部契约 | `tests/cli.rs`, `tests/manifest.rs`, `tests/mcp_server.rs`, `tests/pack.rs` | 对外行为不稳定 |
| Integration | 验证多个模块协作 | `tests/init.rs`, `tests/runtime.rs`, `tests/consumer_guards.rs` | 命令路径不可用 |
| Negative/Security | 验证失败路径、越界、stale、instruction-only | `tests/errors.rs`, `tests/artifacts.rs`, `tests/permissions.rs`, `tests/instruction_only.rs` | 可信边界不成立 |
| E2E Acceptance | 验证 A001-A013 用户路径 | `tests/e2e_matrix.rs` | MVP 不能验收 |
| Business Examples | 验证业务价值示例 | `tests/business_examples.rs` + docs review | 只证明了技术 wrapper |

## 顶层验收矩阵

| ID | 场景 | 命令 | 必须断言 |
| --- | --- | --- | --- |
| A001 | 初始化 capsule | `cargo run -- init refund --python` | 标准文件存在；默认 example 无网络/密钥；重复初始化非空目录失败 |
| A002 | 生成 Manifest | `cargo run -- manifest` | Manifest 含 schema、source hashes、permissions、adapter、tool description；hash 与文件一致 |
| A003 | Inspect runnable capsule | `cargo run -- inspect` | 不执行 `run`；展示 SOP hash、schema、permissions、adapter、examples、preflight、MCP 摘要 |
| A004 | 默认测试成功 | `cargo run -- test` | 生成 `ok: true` envelope、run record、stdout/stderr logs；run id 唯一 |
| A005 | 真实运行成功 | `cargo run -- run --input examples/default.input.json` | output file 存在；result 符合 schema；display markdown 存在；run record 可追溯到 Manifest hash |
| A006 | 输入非法 | `cargo run -- run` | 返回 `ValidationError`；`recoverable=true`；不调用业务 `run` |
| A007 | SOP/preflight 拒绝 | `cargo run -- run` | 返回 `PolicyViolation`；包含 `llm_hint`；stdout 不影响错误判断 |
| A008 | Adapter 协议违规 | `cargo run -- run` | 返回 `ProtocolViolation`；stdout 成功文本不能兜底 |
| A009 | Artifact 越界 | `cargo test --test artifacts` | `../`、绝对路径、Windows drive path 或不存在文件均不得成功记录为 artifact |
| A010 | Stale Manifest | `cargo test --test consumer_guards` | 修改 `SKILL.md`、`action.py` 或 config 后 Consumer Mode fail closed |
| A011 | MCP 暴露 | `cargo run -- serve --mcp --dry-run` | tool schema 来自 Manifest；resource 指向 `SKILL.md`；不 import `action.py` 提取 metadata |
| A012 | Pack 分发 | `cargo run -- pack` | `.skr` 包含 source 和 Manifest；不包含 `.skillrun/runs/`；解包后可 inspect |
| A013 | Instruction-only 保护 | `cargo test --test instruction_only` | inspect 展示 instruction-only；manifest/run/serve/pack 拒绝隐式执行 |

## T001 Baseline Tests

T001 只验证 Rust CLI skeleton：

- `tests/cli.rs::help_lists_planned_mvp_commands`
- `tests/cli.rs::version_uses_approved_project_name`
- `tests/cli.rs::planned_commands_fail_until_implemented`

这些测试不能假装 init、manifest、runtime、MCP 或 pack 已实现。

## Unit / Contract / Integration Mapping

| Task | Primary tests | 覆盖重点 |
| --- | --- | --- |
| T001 | `tests/cli.rs` | Rust binary help/version/unsupported command |
| T002 | `tests/init.rs` | 初始化目录、模板内容、重复写入保护 |
| T003 | `tests/manifest.rs` | Pydantic v2 metadata、source hash、Manifest 最小字段 |
| T004 | `tests/inspect.rs` | inspect 输出、instruction-only 展示 |
| T005 | `tests/runtime.rs` | IPC 文件、run record、成功 envelope |
| T006 | `tests/errors.rs` | Validation/Policy/Protocol/Runtime error |
| T007 | `tests/artifacts.rs`, `tests/permissions.rs` | artifact containment、declared env |
| T008 | `tests/consumer_guards.rs`, `tests/instruction_only.rs` | stale Manifest、隐式执行拒绝 |
| T009 | `tests/mcp_server.rs` | Manifest 到 MCP tool/resource 的映射 |
| T010 | `tests/pack.rs` | `.skr` 内容、排除 run history、stale fail closed |
| T011 | `tests/e2e_matrix.rs`, `tests/business_examples.rs` | A001-A013、B001-B004、release traceability |

## Negative/Security Matrix

| ID | 风险 | 必须测试或说明 |
| --- | --- | --- |
| N001 | stdout 被当成成功 | stdout 出现成功文本但 output 缺失时仍返回 `ProtocolViolation` |
| N002 | Manifest stale 未发现 | 修改 `SKILL.md` 后 run/serve/pack fail closed |
| N003 | `action.py` stale 未发现 | 修改 action 后 run/serve/pack fail closed |
| N004 | config stale 未发现 | 修改 config 后 Consumer Mode fail closed |
| N005 | artifact path traversal | `../` 被拒绝 |
| N006 | absolute path artifact | 绝对路径被拒绝 |
| N007 | Windows drive path artifact | `C:\...` 形式被拒绝 |
| N008 | undeclared env injection | 未声明 env 不进入子进程 |
| N009 | metadata phase secret injection | Author Mode metadata 不注入 secrets |
| N010 | instruction-only 隐式执行 | `scripts/` 和 Markdown code block 不会变成 action |
| N011 | Pydantic v1 或不兼容 schema | 错误信息明确要求 Pydantic v2 |
| N012 | MCP stale exposure | stale Manifest 不启动 MCP exposure |
| N013 | pack stale source | stale Manifest 不生成 `.skr` |
| N014 | run record 缺少 hash | run record 必须可追溯到 skill、manifest、action hash |
| N015 | stack trace 泄漏到 display | stack trace 只进入 debug logs |
| N016 | `.skr` 被误解为 runtime image | README/pack summary 明确不 vendor dependencies |

## 经典业务示例

| ID | 示例 | v0.1 责任 | 验证方式 |
| --- | --- | --- | --- |
| B001 | Refund Decision | 完整实现 | E2E 运行、policy violation、artifact/run record |
| B002 | Support Triage | 文档级示例 | README 或 `docs/business-examples.md` 说明 routing labels 和 missing-context recovery |
| B003 | Access Request Approval | 文档级示例 | README 或 docs 说明 approval boundary、declared env、audit note |
| B004 | Vendor Risk Review | 文档级示例 | README 或 docs 说明 artifact 和 risk summary |

## Release Gate

MVP 完成前必须满足：

- `cargo test` 通过。
- A001-A013 全部有 fresh command evidence。
- N001-N016 有 automated tests 或明确 documented exception。
- B001 完整实现并通过 E2E。
- B002-B004 至少在 README 或 `docs/business-examples.md` 中以经典示例形式解释业务价值。
- `.skr` 可解包并通过 Manifest inspect。

## User Review Gate

- Approval: Approved on 2026-05-11
- Reviewer notes: 测试策略已改为 Rust-first；Python 只作为 Action adapter 的测试 fixture 和业务示例语言存在。
