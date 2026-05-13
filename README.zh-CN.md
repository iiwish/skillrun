# SkillRun

> 把一份 SOP 和一个 Action 打包成可分发的 Agent 技能。

[English](README.md)

FastMCP turns functions into MCP tools。
SkillRun turns SOP-backed capabilities into **Skill Capsules**。

一个 Skill Capsule 携带函数签名无法表达的内容：typed input/output、preflight、structured errors、artifacts、run evidence、declared permissions，以及 Manifest-derived MCP contract。

SkillRun 关注的是：当 Agent 调用一个动作时，业务上下文、恢复规则、审计证据和运行契约也必须一起到场。如果你只想暴露一个函数，FastMCP 更轻；如果 SOP 和代码同样重要，SkillRun 才有价值。

## 当前状态

SkillRun v0.4.0 是当前 release candidate 版本。package version 已提升到 `0.4.0`；尚未创建 `v0.4.0` tag，也尚未执行远端 push 或 package publication。

- 当前实现：v0.2 MCP stdio 行为、v0.3 JS Action Alpha，以及 v0.4 Portable Consumer Checks。
- 当前可用：`skillrun --help`、`skillrun --version`、`skillrun init <name> --python`、`skillrun init <name> --py`、`skillrun init <name> --js`、`skillrun manifest --cwd <capsule>`、`skillrun inspect --cwd <capsule>`、`skillrun check --cwd <capsule>`、`skillrun doctor --cwd <capsule>`、`skillrun test --cwd <capsule>`、`skillrun run --cwd <capsule> --input <file>`、`skillrun serve --mcp --cwd <capsule>`、`skillrun serve --mcp --cwd <capsule> --dry-run`、`skillrun pack --cwd <capsule>`、structured error envelopes、`DependencyError`、artifact validation、declared env injection、stale Manifest guards、instruction-only guards、Manifest-derived MCP tools/resources、`.skr` package generation，以及 skeleton/init/manifest/inspect/check/doctor/runtime/error/artifact/permission/consumer-guard/MCP/pack 路径的 release tests。
- v0.2 保留 `serve --mcp --dry-run` 作为 contract inspection，但普通 `serve --mcp` 路径已经是真实 long-running MCP stdio server。
- SkillRun 本体、CLI、Manifest、IPC、MCP 暴露和 pack 路径使用 Rust 实现。
- Python `action.py` 是稳定 Action adapter 目标。JS `action.mjs` 是 alpha adapter 目标。二者都是用户 action 语言，不是 SkillRun 本体实现语言。

## 为什么需要 SkillRun

多数 Agent tool 系统从“可调用函数”开始。SkillRun 从“业务能力”开始：

```text
Skill Capsule = SOP + action code + schema + examples + permissions
Manifest      = compiled runtime contract
Core          = Rust manifest-driven runtime
Adapter       = language bridge for user actions
Package       = .skr source + Manifest archive
```

当你希望一个 Agent 可调用能力不只携带函数签名时，SkillRun 才有价值：

- `SKILL.md` 作为认知契约，描述业务 SOP。
- typed input/output schema。
- 用 `preflight` 表达政策、审批和前置条件边界。
- 结构化 success/error envelope。
- artifact 是一等输出，而不是 stdout 附属品。
- run record 保留 hash、日志和执行证据。
- MCP 暴露来自 Manifest，Consumer Mode 不重新 import 源码提取 metadata。

如果你只想把一个 Python 函数暴露成 MCP tool，FastMCP 这类轻量工具可能更合适。SkillRun 面向的是需要可检查、可测试、可分发的 SOP-backed capability。

## 核心流程

```text
refund/
  SKILL.md
  action.py                  # Python stable path
  # action.mjs               # JS alpha path
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
        +-- skillrun check
        +-- skillrun doctor
        +-- skillrun test
        +-- skillrun run --input examples/default.input.json
        +-- skillrun serve --mcp             # MCP stdio server
        +-- skillrun serve --mcp --dry-run   # contract inspection
        +-- skillrun pack
```

生成的 Manifest 是运行契约。Author Mode 可以从本地源文件重新生成它；Consumer Mode 只读取 Manifest，校验 source hashes，并在 Manifest 缺失或过期时 fail closed。

v0.2 中，`skillrun serve --mcp` 已经是一个真实 MCP stdio server，同时 tool 和 resource 仍然从 Manifest 派生。

v0.4 中，`skillrun check` 是面向自动化的 readiness command。它读取 Manifest、source hashes、entrypoint、examples 和 runtime requirements，解释当前机器能否消费或运行这个 capsule。它不会 import `action.py` 或 `action.mjs`，也不会安装依赖。

## Release Candidate 工作流

```bash
skillrun init refund --python
cd refund
# edit SKILL.md
# edit action.py
skillrun manifest
skillrun inspect
skillrun check
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp
skillrun pack
```

`--py` 只是 `--python` 的短别名。README 主 Quickstart 保持 `--python`，因为 Python 是稳定路径。

语言 flag 只属于 `init`。`manifest`、`inspect`、`check`、`doctor`、`test`、`run`、`serve --mcp` 和 `pack` 读取 capsule 与生成的 Manifest，不接受 `--python`、`--py` 或 `--js`。

`inspect`、`check` 和 `doctor` 的职责不同：

- `inspect` 展示 Manifest contract：SOP summary、schemas、permissions、adapter、entrypoint、examples 和 source hashes。
- `check` 从静态 capsule 数据和 runtime probes 诊断当前 host readiness。
- `doctor` 是人类友好的诊断视图，并遵守同一条 Consumer Mode 边界。

首个 hero example 是 `refund`：一个退款决策 capsule，包含政策限额、审批边界、类型化输入、结构化 `PolicyViolation` 错误和可审计 run record。

## JS Action Alpha

v0.3 的 JS 支持刻意保持很窄：

```bash
skillrun init refund-js --js
cd refund-js
# edit SKILL.md
# edit action.mjs
skillrun manifest
skillrun inspect
skillrun check
skillrun doctor
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp --dry-run
skillrun pack
```

JS alpha contract 是 canonical ESM `action.mjs`，显式导出 `inputSchema`、`outputSchema`、可选 `preflight` 和 `run`。SkillRun v0.3 不从 TypeScript types、JSDoc、Zod、TypeBox、examples 或 package metadata 推断 schema。

`action.ts` 不是 runtime entrypoint。作者可以自行把 TypeScript 编译到 `action.mjs`，但 SkillRun v0.3 不运行 `ts-node`、`tsx`、source maps、CJS/ESM 兼容矩阵或 package-manager install flow。

## 让 Agent 在调用前学习 Capsule

SkillRun Capsule 应该先被 AI 助手学会，再被调用。给 AI 的应该是直接指向 Capsule 文件夹的 URL 或 repo path，而不是项目首页。这个文件夹应包含 `SKILL.md`、`skillrun.config.json`、`action.py` 或 `action.mjs`，以及 `examples/`。

```text
请先学习这个 SkillRun Capsule，再使用它：
<capsule-folder-url-or-repo-path>

1. 阅读 SKILL.md，理解 purpose、SOP、prohibited behavior、required context 和 recovery guidance。
2. 阅读 skillrun.config.json 和已生成的 Manifest，确认 adapter 与 entrypoint。
3. 只把 action.py 或 action.mjs 当成该 capsule 的 action contract；不要推断未声明语言或 package-manager 行为。
4. 阅读 examples/default.input.json，理解调用时需要的输入形态。
5. 如果你能访问工作区，运行 `skillrun inspect --cwd <capsule>`、`skillrun check --cwd <capsule>`、`skillrun doctor --cwd <capsule>` 和 `skillrun test --cwd <capsule>`。
6. 调用 MCP tool 时，不要从 stdout 推断成功。只看 output/error envelope、artifacts 和 run record。
```

发布自己的 skill 时，应使用真实的 Capsule 文件夹链接。这样 Agent 不会把 Capsule 当成一个松散函数调用，而是先学习 SOP、adapter entrypoint、示例输入和失败行为，再通过 MCP tool 使用它。

## 现在能运行什么

仓库当前包含 Rust CLI、`init --python` 和 `init --py` Python capsule 生成器、`init --js` JS alpha capsule 生成器、Manifest 生成器、inspect renderer、dependency-aware `check`、doctor diagnostics、test/run 路径、MCP stdio server、MCP dry-run contract renderer、`.skr` package generation，以及 B001 `refund` hero example：

```bash
cargo test
cargo run -- --help
cargo run -- --version
cargo run -- init refund --python --output tmp/e2e-init
cargo run -- manifest --cwd tmp/e2e-init/refund
cargo run -- inspect --cwd tmp/e2e-init/refund
cargo run -- check --cwd tmp/e2e-init/refund
cargo run -- doctor --cwd tmp/e2e-init/refund
cargo run -- test --cwd tmp/e2e-init/refund
cargo run -- run --cwd tmp/e2e-init/refund --input examples/default.input.json
cargo run -- serve --mcp --cwd tmp/e2e-init/refund --dry-run
cargo run -- pack --cwd tmp/e2e-init/refund
```

当前本地 binary 输出：

```text
skillrun 0.4.0
```

真实 `serve --mcp` 命令是 long-running stdio server，并已经通过 scripted MCP client release matrix 验证。

`.skr` package 是 source/Manifest archive。它不是 signed package，不 vendor dependencies，也不提供 reproducible runtime image。解包后，consumer 仍然可以运行 `inspect` 和 `check` 来理解 capsule，并在不执行 action source 的前提下诊断缺失的 Python、Node 或 Pydantic 依赖。

## Release Candidate 限制

当前 release candidate 刻意保持收敛：

- MCP transport 仅支持 stdio。
- 每个 capsule 暴露一个 Manifest-derived primary tool。
- Python `action.py` 是稳定 action adapter 目标。
- JS `action.mjs` 只是 alpha，不是完整 TypeScript 支持。
- `action.ts`、直接 TypeScript runtime execution、`ts-node`、`tsx`、type-to-schema extraction、source maps、CJS compatibility、npm install flow 和 dependency vendoring 都不在 v0.3 范围内。
- `.skr` 是 source + Manifest archive，不是 signed package、registry package、dependency bundle 或 runtime image。
- `check` 诊断 dependency readiness；它不会安装 Python、Node、Pydantic、npm packages，也不会创建 virtual environments。
- 缺失 runtime dependencies 会在 CLI runtime path 和 MCP tool calls 中返回结构化 `DependencyError`。
- SkillRun 不提供 OS sandbox。运行第三方 action 仍然意味着执行第三方代码。
- 尚未创建 `v0.4.0` tag。tag creation、remote push 和 package publication 是单独的显式决策。

## 安全模型

SkillRun 对信任边界保持诚实：

- `stdout` 和 `stderr` 只作为日志。结构化结果必须来自 output/error envelope。
- Consumer Mode 不应该为了提取 metadata 而动态 import 未信任源码。
- `skillrun check` 和 `skillrun doctor` 是 Consumer Mode diagnostics；它们不会为了 metadata import action source。
- Manifest 缺失或 stale 时 fail closed。
- 声明过的环境变量和 artifact path 属于运行契约。
- SkillRun 不声称提供完整 OS sandbox。运行第三方 action 仍然意味着执行第三方代码。
- `.skr` 不是 secure install format、registry package 或 dependency bundle。
- Dependency readiness 不是 sandboxing、vendoring 或 reproducible environment creation。

目标是建立小而硬的边界：不隐式执行 instruction-only skill，不把 stdout 当成功兜底，不在 Consumer Mode 为 metadata import 源码。

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
| `v0.2` | Real MCP stdio server 和 public release candidate readiness |
| `v0.3` | Adapter boundary、JS Action Alpha via `action.mjs`、`doctor` 和 explicit TypeScript boundary |
| `v0.4` | Portable Consumer Checks、dependency-aware Consumer Mode、`check` 和 structured `DependencyError` |

## 经典业务示例

SkillRun 的业务证明刻意保持收敛：

- `B001: Refund Decision` 已在 `examples/refund` 中完整实现，并通过 success、`PolicyViolation`、`ValidationError`、run record、MCP dry-run exposure 和 `.skr` packaging 做端到端验证。
- `B002: Support Triage` 是 docs-level example，用于说明 stable routing labels 和 missing-context recovery。
- `B003: Access Request Approval` 是 docs-level example，用于说明 approval boundary、declared env 和 audit note。
- `B004: Vendor Risk Review` 是 docs-level example，用于说明 artifact-first review summary，以及不 vendor dependencies 的 package distribution。
- `B005: WeCom Team Notice` 已在 `examples/wecom_team_notice` 中作为 v0.4.1 official runnable example 实现，用于说明 dry-run preview、approval boundary、declared `WECOM_WEBHOOK_URL`、structured `DependencyError`、markdown artifact，以及真实本地通知工作流的 MCP 使用方式。

当前可运行示例仍刻意保持收敛：`refund` 证明安全与审计边界，`wecom_team_notice` 证明更贴近日常工作的本地通知场景，但不把 SkillRun 变成企业微信 adapter 或 API wrapper。

## 文档

- [文档入口](docs/README.md)
- [MVP 合同](docs/mvp.md)
- [架构 SSOT](docs/ssot.md)
- [v0.4 Portable Consumer Checks](docs/v0.4-portable-consumer-checks.md)
- [经典业务示例](docs/business-examples.md)
- [测试策略](docs/testing.md)
- [发布策略](docs/release-policy.md)
- [分支保护建议](docs/branch-protection.md)
- [贡献指南](CONTRIBUTING.md)
- [安全政策](SECURITY.md)
- [行为准则](CODE_OF_CONDUCT.md)

项目治理文档默认使用中文，方便后续 agent 维护已确认的产品合同。

## 贡献

SkillRun 有意保持尖锐和克制。贡献时请保持这些项目约定：

- 项目名使用 `SkillRun`，CLI、crate、命令和代码标识使用 `skillrun`。
- SkillRun Core 行为使用 Rust 实现。
- Python `action.py` 是稳定 Action adapter 目标。
- JS `action.mjs` 是窄边界 alpha adapter path。
- 不隐式执行 instruction-only skill。
- 不从 stdout 推断结构化成功结果。
- 不把 JS alpha 扩成完整 TypeScript support、package-manager ownership、dependency vendoring、registry behavior 或 sandbox claims。
- README 和文档必须清楚区分“已经实现”和“计划实现”。

提交变更前运行基线检查：

```bash
cargo test
```

## 许可证

SkillRun 使用 [Apache License, Version 2.0](LICENSE)。
