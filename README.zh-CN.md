# SkillRun

> 用一份 SOP 和一个 Action，生成一个可测试、可检查、可给 Agent 调用、可打包分发的 MCP 技能包。

[English](README.md)

SkillRun 是 Rust-first 的本地 CLI/Core，用来构建 **SOP-backed Skill Capsule**。它把一份面向人和 Agent 的业务 SOP、一个显式 Action、schema、examples 和 permissions 编译成 Manifest 驱动的技能包，使它可以被检查、测试、运行、暴露给 MCP client，并作为 `.skr` artifact 分发。

SkillRun 不是又一个“把函数包装成工具”的层。它关注的是：当 Agent 调用一个动作时，业务上下文、恢复规则、审计证据和运行契约也必须一起到场。

## 当前状态

SkillRun 正处在 v0.1.0 MVP 建设阶段。

- 当前实现：已推进到 `T006` Rust structured error envelopes。
- 当前可用：`skillrun --help`、`skillrun --version`、`skillrun init <name> --python`、`skillrun manifest --cwd <capsule>`、`skillrun inspect --cwd <capsule>`、`skillrun test --cwd <capsule>`、`skillrun run --cwd <capsule> --input <file>`、structured error envelopes，以及 CLI/init/manifest/inspect/runtime/error 路径的 contract tests。
- CLI 已列出计划中的 MVP 命令，但 `serve` 和 `pack` 尚未实现。
- SkillRun 本体、CLI、Manifest、IPC、MCP 暴露和 pack 路径使用 Rust 实现。
- Python `action.py` 是首个计划中的 Action adapter 目标。它是用户 action 的语言，不是 SkillRun 本体的实现语言。

## 为什么需要 SkillRun

多数 Agent tool 系统从“可调用函数”开始。SkillRun 从“业务能力”开始：

```text
Skill Capsule = SOP + action code + schema + examples + permissions
Manifest      = compiled runtime contract
Core          = Rust manifest-driven runtime and MCP server
Adapter       = language bridge for user actions
Package       = immutable .skr distribution artifact
```

当你希望一个 Agent 可调用能力不只携带函数签名时，SkillRun 才有价值：

- `SKILL.md` 作为认知契约，描述业务 SOP。
- typed input/output schema。
- 用 `preflight` 表达政策、审批和前置条件边界。
- 结构化 success/error envelope。
- artifact 是一等输出，而不是 stdout 附属品。
- run record 保留 hash、日志和执行证据。
- MCP 暴露基于 Manifest，Consumer Mode 不重新 import 源码提取 metadata。

如果你只想把一个 Python 函数暴露成 MCP tool，FastMCP 这类轻量工具可能更合适。SkillRun 面向的是需要可检查、可测试、可分发的 SOP-backed capability。

## 核心流程

```text
refund/
  SKILL.md
  action.py
  examples/
    default.input.json
  skillrun.config.json
  .skillrun/
    manifest.generated.yaml

        |
        | skillrun manifest
        v

Manifest-driven contract
  schema
  permissions
  adapter
  tool description
  source hashes

        |
        +-- skillrun inspect
        +-- skillrun test
        +-- skillrun run --input examples/default.input.json
        +-- skillrun serve --mcp
        +-- skillrun pack
```

生成的 Manifest 是运行契约。Author Mode 可以从本地源文件重新生成它；Consumer Mode 只读取 Manifest，校验 source hashes，并在 Manifest 缺失或过期时 fail closed。

## MVP 计划工作流

```bash
skillrun init refund --python
cd refund
# edit SKILL.md
# edit action.py
skillrun manifest
skillrun inspect
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp
skillrun pack
```

首个 hero example 是 `refund`：一个退款决策 capsule，包含政策限额、审批边界、类型化输入、结构化 `PolicyViolation` 错误和可审计 run record。

## 现在能运行什么

仓库当前包含 Rust CLI skeleton、`init --python` capsule 生成器、Manifest 生成器、inspect renderer 和 test/run 成功路径：

```bash
cargo test
cargo run -- --help
cargo run -- --version
cargo run -- init refund --python --output tmp/e2e-init
cargo run -- manifest --cwd tmp/e2e-init/refund
cargo run -- inspect --cwd tmp/e2e-init/refund
cargo run -- test --cwd tmp/e2e-init/refund
cargo run -- run --cwd tmp/e2e-init/refund --input examples/default.input.json
```

示例输出：

```text
skillrun 0.1.0
```

后续 MCP 和 packaging 命令在实现前会明确返回 `command not implemented yet`，避免让用户误以为这些行为已经落地。

## 安全模型

SkillRun 对信任边界保持诚实：

- `stdout` 和 `stderr` 只作为日志。结构化结果必须来自 output/error envelope。
- Consumer Mode 不应该为了提取 metadata 而动态 import 未信任源码。
- Manifest 缺失或 stale 时 fail closed。
- 声明过的环境变量和 artifact path 属于运行契约。
- v0.1 不声称提供完整 OS sandbox。运行第三方 action 仍然意味着执行第三方代码。

MVP 的目标是建立小而硬的边界：不隐式执行 instruction-only skill，不把 stdout 当成功兜底，不在 Consumer Mode 为 metadata import 源码。

## 路线图

| Milestone | 重点 |
| --- | --- |
| `T001` | Rust CLI skeleton、help、version、unsupported command behavior |
| `T002` | `init --python` capsule skeleton |
| `T003` | Manifest generation、Pydantic v2 schema extraction、source hashes |
| `T004` | `inspect` 和 instruction-only status |
| `T005` | IPC runtime、output envelopes、run records |
| `T006` | structured errors 和 failure discipline |
| `T007` | artifact containment 和 declared environment handling |
| `T008` | Consumer guards 和 stale Manifest behavior |
| `T009` | Manifest-driven MCP exposure |
| `T010` | `.skr` packaging |
| `T011` | E2E acceptance matrix 和 business examples |

## 文档

- [MVP 合同](docs/mvp.md)
- [架构 SSOT](docs/ssot.md)
- [经典业务示例](docs/business-examples.md)
- [测试策略](.ai-platform/docs/test-strategy.md)

项目治理文档默认使用中文，方便后续 agent 维护已确认的产品合同。

## 贡献

SkillRun 在 v0.1 阶段有意保持尖锐和克制。贡献时请保持这些项目约定：

- 项目名使用 `SkillRun`，CLI、crate、命令和代码标识使用 `skillrun`。
- SkillRun Core 行为使用 Rust 实现。
- Python 只是首个 Action adapter 目标。
- 不隐式执行 instruction-only skill。
- 不从 stdout 推断结构化成功结果。
- README 和文档必须清楚区分“已经实现”和“计划实现”。

提交变更前运行基线检查：

```bash
cargo test
```

## 许可证

SkillRun 使用 [Apache License, Version 2.0](LICENSE)。
