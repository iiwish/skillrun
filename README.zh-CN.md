# SkillRun

> 面向 AI Agent 的 SOP-backed skill runtime。无 Manifest 护栏，不执行。

[English](README.md)

FastMCP 把函数变成 MCP tool。SkillRun 把 SOP-backed capability 变成 **Skill Capsule**。

SkillRun 是一个 Rust runtime 和 CLI，用一份 SOP 和一个 Action 打包出可检查、可测试、可运行、可分发、可通过 MCP 调用的 Agent skill。它不是通用 Agent framework，不是 marketplace，也不是 OS sandbox。

## 为什么需要 SkillRun

大多数 agent tool 系统从“可调用函数”开始。低风险动作可以这样做，但真实业务流程不够。

SkillRun 从业务能力开始：

```text
Skill Capsule = SOP + action code + schema + examples + permissions
Manifest      = compiled runtime contract
Core          = Rust Manifest-driven runtime
Adapter       = language bridge for user actions
Package       = .skr source + Manifest archive
```

一个 Skill Capsule 携带函数签名无法表达的内容：

- `SKILL.md` 描述 SOP、适用边界、禁止行为和恢复建议。
- 类型化 input/output schema。
- 用 `preflight` 表达审批、策略、缺失上下文和失败恢复边界。
- 结构化 success/error envelope。
- artifact 作为一等输出，而不是 stdout 附属品。
- run record 保存 hash、耗时、日志和执行证据。
- MCP 暴露来自 Manifest，Consumer Mode 不为 metadata 动态 import 未信任源码。

如果你只想把一个函数暴露成 MCP tool，用 FastMCP 更轻。SOP 和代码同样重要时，SkillRun 才有价值。

## 当前状态

当前公开 release candidate：`v0.5.13`。

当前 binary/crate 版本：

```bash
skillrun --version
# skillrun 0.5.13
```

当前已经可用：

- Python `action.py` 稳定 adapter target。
- JS `action.mjs` alpha adapter target。
- Level 0 `command` adapter，用显式 argv 进程遵守 SkillRun IPC。
- Manifest generation，包含 source hash 与 runtime contract 字段。
- `inspect`、`check`、`doctor` 的 human 和 JSON surface。
- `test`、`run` 输出结构化 output/error envelope。
- 从 Manifest 派生 MCP stdio server。
- `.skr` source + Manifest package。
- 本地 `.skr` import 到 capsule registry：
  - `skillrun import <package.skr> --json`
- 本地 capsule `registry` 和 `switchboard`。
- 用于一键挂载的本地 MCP Router：
  - `skillrun router serve --mcp`
  - `skillrun router serve --mcp --dry-run`
- 可逆的 Claude Desktop MCP config 挂载：
  - `skillrun consumer mount apply --client claude-desktop --json`
  - `skillrun consumer mount rollback --client claude-desktop --backup <path> --json`
- 面向 Desktop、Router 检查和自动化消费者的 headless consumer JSON surface：
  - `skillrun consumer inventory --json`
  - `skillrun consumer exposure --json`
  - `skillrun consumer runs list --json`
  - `skillrun consumer runs inspect <run-id> --json`
  - `skillrun consumer mount plan --client <id> --json`

v0.5.13 刻意不加入 Desktop、Tauri、`skillrun ui`、daemon API、Router hot reload、Router process management、Cursor apply、多客户端 mount adapter、signed package trust、dependency installation、package update/reinstall、import from URL、marketplace、`--include-input`、artifact content read、log content read、global run index 或 OS sandbox。

## 快速开始

在仓库根目录运行黄金路径：

```bash
cargo run -- init refund --python --output tmp/quickstart
cargo run -- manifest --cwd tmp/quickstart/refund
cargo run -- inspect --cwd tmp/quickstart/refund
cargo run -- check --cwd tmp/quickstart/refund
cargo run -- doctor --cwd tmp/quickstart/refund
cargo run -- test --cwd tmp/quickstart/refund
cargo run -- run --cwd tmp/quickstart/refund --input examples/default.input.json
cargo run -- serve --mcp --cwd tmp/quickstart/refund --dry-run
cargo run -- pack --cwd tmp/quickstart/refund
```

真实 MCP stdio server：

```bash
cargo run -- serve --mcp --cwd tmp/quickstart/refund
```

`serve --mcp` 是长运行 stdio server。只想检查 MCP contract 时使用 `serve --mcp --dry-run`。

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
  schemas
  permissions
  adapter
  tool description
  source hashes

        |
        +-- inspect / check / doctor
        +-- import / registry / switchboard
        +-- consumer inventory / exposure / runs / mount plan
        +-- test / run
        +-- serve --mcp
        +-- pack
```

Author Mode 可以从本地源码重新生成 Manifest。Consumer Mode 只读取 Manifest，校验 source hash 和 runtime contract 字段；Manifest 缺失、过期或非法时 fail closed。

## 信任模型

SkillRun 对边界保持诚实。

“无 Manifest 护栏，不执行”中的护栏指 Manifest contract、input/output schema、preflight、structured envelope、artifact containment、run evidence 和 Consumer Mode static checks。

它不等于：

- OS 级 sandbox。
- 网络出口隔离。
- 自动安装依赖。
- 签名包信任体系。
- 可复现 runtime image。
- 安全执行任意第三方代码。

运行第三方 action 仍然意味着执行第三方代码。SkillRun 减少的是 Agent 裸调动作的风险：把 SOP、运行合同、证据和失败行为显式化。

关键规则：

- `stdout` 和 `stderr` 只作为日志，结构化结果必须来自 output/error envelope。
- Consumer Mode 不为 metadata 动态 import 未信任源码。
- Manifest 缺失或过期时 fail closed。
- `.skr` 是 source + Manifest archive，不是 secure install format。
- `registry` 是本地 inventory，不是 trust store。
- `switchboard enabled=true` 是未来 exposure intent，不是 trust 或 sandbox 证明。

## Desktop 方向

Desktop 是独立项目。它应该消费 SkillRun Core 暴露的稳定 headless surface，而不是直接读取 `.skillrun/` 内部结构或反解析 MCP text。

推荐边界：

```text
skillrun
  Rust CLI/Core, Manifest, Consumer Mode, Adapter Protocol, runtime, pack,
  registry/switchboard, headless JSON surfaces, Router MVP

skillrun-desktop
  Tauri shell, Capsule Switchboard, MCP Mount Manager, Envelope Explorer,
  official pack browser
```

一键挂载的核心规则是：挂载 SkillRun Router，而不是直接挂 `.skr` 或 capsule 文件夹。`.skr` 是 import/distribution artifact，Router 才是 MCP runtime entry。

## 版本层级

SkillRun 同时存在几类版本：

- `Cargo.toml` 和 `skillrun --version` 标识 binary/crate version。
- `v0.5.13` 这类 Git tag 标识公开 release 边界。
- v0.5.4、v0.5.5、v0.5.6、v0.5.7、v0.5.8、v0.5.9、v0.5.10、v0.5.11、v0.5.12、v0.5.13 这类 milestone 描述交付范围。
- Manifest `manifest_version` 标识 Manifest IR schema。
- IPC / Adapter `protocol_version` 标识 Core 到 adapter process 的文件协议。

当前生成的 Manifest IR 与 IPC protocol version 仍是 `0.1.0`。v0.5.13 硬化 `.skr import` 到 Router exposure 的合同，但不改变这些协议版本。

## 路线图

| Milestone | 重点 |
| --- | --- |
| `v0.2` | Real MCP stdio server 与公开 release candidate |
| `v0.3` | JS Action Alpha via `action.mjs` 与 TypeScript 边界 |
| `v0.4` | Portable Consumer Checks 与 dependency-aware Consumer Mode |
| `v0.5` | Language-agnostic Adapter Protocol 与 Level 0 command adapter |
| `v0.5.4` | Desktop 前的 Core Stabilization Audit |
| `v0.5.5` | Manifest-driven Consumer Mode contract hardening |
| `v0.5.6` | Desktop 前置的 headless consumer JSON contracts |
| `v0.5.7` | Desktop 前的 public narrative 与 contract-surface polish |
| `v0.5.8` | Router runtime MVP，支撑真实一键挂载 |
| `v0.5.9` | Safe Mount Apply，提供可逆 MCP client config 写入 |
| `v0.5.10` | Desktop 前的 Consumer Contract Hardening |
| `v0.5.11` | 面向 Desktop Envelope Explorer 的 Runs Inspect |
| `v0.5.12` | 面向 Desktop 本地 inventory 的 Capsule Import |
| `v0.5.13` | Desktop 前的 Import-to-Router contract hardening |
| `v0.6` | 建议：Consumer Era Desktop 与本地控制面 |

## 示例

可运行示例刻意保持收敛，用来证明 SkillRun 的边界，而不是把项目变成通用 API wrapper。

- `examples/refund`：退款决策，包含政策限额、审批边界、类型化输入、结构化 `PolicyViolation`、artifact、run record、MCP 暴露和 `.skr` package。
- `examples/wecom_team_notice`：本地通知工作流，包含 dry-run preview、审批边界、声明式 `WECOM_WEBHOOK_URL`、结构化 `DependencyError` 和 markdown artifact。
- `examples/commit_message_gate`：校验 Conventional Commits，不自动 stage 文件。
- `examples/bounded_file_patcher`：在声明目录内执行精确文本替换，并记录 patch artifact。
- `examples/readonly_diagnostics_runner`：只运行命名 allowlist 诊断，不接受任意 shell 字符串。
- `examples/command_hello`：Level 0 command adapter contract，不依赖 SkillRun SDK。

文档级业务模式仍保留在项目叙事中，但不扩大当前 runtime scope：Support Triage、Access Request Approval 和 Vendor Risk Review 用来说明一个 portable Agent skill 如何携带稳定分流标签、审批边界和 artifact-backed review evidence。

## 文档

- [文档入口](docs/README.md)
- [架构 SSOT](docs/ssot.md)
- [项目定位](docs/positioning.md)
- [信任模型](docs/trust-model.md)
- [Adapter Protocol](docs/adapter-protocol.md)
- [v0.5.6 Headless Consumer Contract](docs/v0.5.6-headless-consumer-contract.md)
- [v0.5.6 Run History Contract Review](docs/v0.5.6-run-history-contract-review.md)
- [v0.5.6 Mount Plan Contract Review](docs/v0.5.6-mount-plan-contract-review.md)
- [v0.5.8 Router MVP](docs/v0.5.8-router-mvp.md)
- [v0.5.9 Safe Mount Apply](docs/v0.5.9-safe-mount-apply.md)
- [v0.5.10 Consumer Contract Hardening](docs/v0.5.10-consumer-contract-hardening.md)
- [v0.5.11 Runs Inspect](docs/v0.5.11-runs-inspect.md)
- [v0.5.12 Capsule Import](docs/v0.5.12-capsule-import.md)
- [v0.5.13 Import Router Contract](docs/v0.5.13-import-router-contract.md)
- [v0.6 Consumer Era vision](docs/v0.6-consumer-era-vision.md)
- [业务示例](docs/business-examples.md)
- [测试策略](docs/testing.md)
- [发布策略](docs/release-policy.md)
- [Release Checklist](docs/release-checklist.md)
- [贡献指南](CONTRIBUTING.md)
- [安全策略](SECURITY.md)

项目治理文档默认使用中文，方便后续 agent 解析和维护已确认的产品合同。

## 贡献

SkillRun 刻意保持边界收敛。贡献应遵守：

- 项目名称使用 `SkillRun`，CLI、crate、命令和代码标识使用 `skillrun`。
- SkillRun core 行为保持 Rust 实现。
- Python 是稳定 action adapter target；JS `action.mjs` 是窄 alpha 路径。
- 不隐式执行 instruction-only skill。
- 不从 stdout 推断结构化成功。
- 不把 JS alpha 扩成完整 TypeScript support、package-manager ownership、dependency vendoring、registry behavior 或 sandbox claims。
- README 和文档必须清楚区分当前已实现能力与计划能力。

提交前运行基础检查：

```bash
cargo test
```

## License

SkillRun 使用 [Apache License, Version 2.0](LICENSE)。
