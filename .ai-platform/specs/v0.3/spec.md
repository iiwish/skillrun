# SkillRun v0.3 Feature Spec

Version: v0.3
Status: Confirmed
Created: 2026-05-12
Updated: 2026-05-13
Source: post-v0.2 direction review; TS/JS scope review; CLI language selection review
Review: Confirmed for planning by user on 2026-05-12; CLI language selection confirmed on 2026-05-13

## 一句话判断

v0.3 应该把作者质量闭环和第一个 adapter generalization 绑定在一起：支持 **JS Action Alpha**，但不承诺完整 TypeScript 工具链。

> v0.3 要证明两件事：陌生作者能低摩擦写出高质量 Skill Capsule；SkillRun Core 也真的不是 Python wrapper，而是 Manifest-driven multi-adapter runtime。

v0.2 证明外部调用路径成立；v0.3 要证明 authoring quality loop 和第二语言 adapter 边界成立。

## 北极星

**v0.3 Authoring Quality Loop + JS Action Alpha**

让作者在本地完成这条闭环：

```text
init -> edit SOP/action -> manifest -> validate/doctor -> test -> inspect -> improve
```

并让同一条闭环至少支持两条明确路径：

```text
skillrun init refund --python
skillrun init refund --py
skillrun init refund --js
```

目标不是把 SkillRun 扩成通用 Node/TypeScript runtime，而是用 `action.mjs` 这条窄路径证明：

- `Manifest` 是稳定 runtime IR。
- Rust Core 通过 adapter boundary 调用 action。
- Consumer Mode 仍只信静态 Manifest，不为 schema 动态 import 未信任源码。

## 设计原则

- 继续把 `Manifest` 作为 runtime IR，不把它变成默认手写入口。
- Python action 继续是稳定 blessed path；JS action 是 v0.3 alpha blessed path。
- JS path 只承诺 canonical ESM：`action.mjs`。
- v0.3 不承诺 full TypeScript support、type-to-schema 自动提取、CJS 兼容矩阵或 package manager 集成。
- JS schema 必须显式导出 JSON Schema；不从 TypeScript type、JSDoc 或 examples 猜 schema。
- `init` 阶段必须显式选择模板；`--py` 是 `--python` 的短别名，不是新 adapter。
- `manifest` 阶段可以在 Author Mode 中按 config 优先、文件后缀次之的规则确定 adapter。
- `test`、`run`、`serve --mcp`、`pack` 阶段只读取 Manifest，不接受语言选择 flag。
- 继续避免 sandbox、registry、marketplace、HTTP server、dependency vendoring 和 runtime image。
- 所有新能力必须加强 `Skill Capsule = SOP + action + schema + examples + permissions` 这个心智模型。

## Requirement Map

### Functional Requirements

- FR-001: SkillRun Core must dispatch metadata extraction and runtime execution through an explicit adapter boundary instead of assuming Python-only paths.
- FR-002: Existing Python capsule behavior must remain compatible across `init --python`, `manifest`, `inspect`, `test`, `run`, `serve --mcp`, and `pack`.
- FR-003: `skillrun init refund --js` must generate a runnable JS alpha capsule with `SKILL.md`, `action.mjs`, `examples/default.input.json`, and `skillrun.config.json`.
- FR-004: JS metadata extraction must read explicit JSON Schema exports from `action.mjs`; it must not infer schema from TypeScript types, JSDoc, Zod, TypeBox, or examples.
- FR-005: JS runtime execution must support `preflight(input, ctx)` and `run(input, ctx)`, including async `run`, and must produce the same output/error envelope contract as Python.
- FR-006: Consumer Mode must continue to validate static Manifest source hashes and must not dynamically import source code for metadata.
- FR-007: JS alpha capsules must preserve Manifest-derived MCP exposure and `.skr` packaging behavior.
- FR-008: `doctor` / `validate` or equivalent diagnostics must be adapter-aware and must not execute business action code.
- FR-009: README and release-facing docs must explain the Python stable path, JS alpha path, and TypeScript boundary without overstating security or dependency management.
- FR-010: CLI language selection must be explicit only at `init`; `--py` must behave as an alias for `--python`, `manifest` must resolve adapter by config-first deterministic convention, and Consumer Mode commands must not accept language flags.

### Non-functional Requirements

- NFR-001: v0.3 must not expand SkillRun security claims beyond current honest Consumer Mode and permission-declaration boundaries.
- NFR-002: v0.3 must not introduce package manager install, dependency vendoring, reproducible runtime image, or signed package responsibilities.
- NFR-003: Manifest and IPC changes must preserve v0.2 behavior or be versioned explicitly.
- NFR-004: Every behavior change must be covered by targeted tests and at least one release-level matrix path before acceptance.

## In Scope

### P0: Adapter Boundary Generalization

- 从 Rust Core 中抽出 adapter dispatch，避免 manifest generation 和 runtime execution 直接硬编码 Python。
- Python adapter 和 Node adapter 必须暴露等价的 metadata/run contract。
- Manifest 继续记录 `runtime.adapter`、`runtime.entrypoint`、source hashes、schema、examples 和 permissions。
- Consumer Mode 不为 metadata 动态 import 源码；只按 Manifest 启动 adapter 子进程执行 action。

### P0: JS Action Alpha

- 增加 `skillrun init refund --js` 形式的 JS alpha 初始化路径。
- 生成 `action.mjs`、`SKILL.md`、`examples/default.input.json`、`skillrun.config.json`。
- `skillrun.config.json` 使用 `runtime.adapter: "node"` 与 `runtime.entrypoint: "action.mjs"`。
- `action.mjs` 使用 ESM，并显式导出：
  - `inputSchema`
  - `outputSchema`
  - optional `preflight(input, ctx)`
- `run(input, ctx)`，允许 async
- JS capsule 必须跑通 `manifest`、`inspect`、`test`、`run`、`serve --mcp` 和 `pack`。

### P0: CLI Language Selection Semantics

- `skillrun init refund --python` 是 README 主 Quickstart。
- `skillrun init refund --py` 是 `--python` 的短别名，生成相同 Python capsule。
- `skillrun init refund --js` 生成 JS alpha capsule。
- `skillrun init refund` 不设置隐式默认语言；缺少模板 flag 时必须报错并给出可恢复提示。
- `--python` / `--py` 与 `--js` 必须互斥。
- `skillrun manifest` 在 Author Mode 中按 `skillrun.config.json` 优先；无 config 时只允许唯一 action 文件 convention：`action.py` 或 `action.mjs`。
- `skillrun test`、`skillrun run`、`skillrun serve --mcp`、`skillrun pack` 不接受 `--python`、`--py` 或 `--js`；它们只读取 Manifest。

### P0: Author Diagnostics

- 设计并实现一个最小诊断入口，候选命令为 `skillrun doctor` 或 `skillrun validate`。
- 诊断必须 adapter-aware：能解释 Python/JS capsule 的缺失文件、stale Manifest、schema 缺失、examples 缺失、permission/artifact 错误。
- 诊断不得执行业务 action。
- 诊断不得把 Node/TS dependency 安装问题包装成 SkillRun 自动解决的问题。

### P1: Golden Author Path

- README Quickstart 必须能让新作者 10 分钟内完成第一个 Python capsule。
- README 可以加入紧凑 JS alpha path，但不能让 JS 抢走主叙事。
- 错误恢复路径要明确告诉作者下一步运行哪个命令。
- `skillrun init --python`、`skillrun init --py` 和 `skillrun init --js` 的模板都必须像真实业务 skill，而不是玩具函数；README 主路径仍使用 `--python`。

### P1: Manifest Explanation

- 提供更清楚的 Manifest-derived contract 解释，可能通过 `inspect` 增强或新命令实现。
- 重点解释 tool schema、resource exposure、permissions、source hashes、adapter、entrypoint 和 run evidence。
- 必须明确 MCP contract 来自 Manifest，而不是 live source import。

### P1: Example Quality

- 继续保留 `refund` Python capsule 作为唯一必须完整运行的 hero example。
- JS starter capsule 可作为 adapter generalization example，但不得引入真实外部 API、密钥、网络依赖或 package manager requirement。
- 示例目标是解释业务边界和 adapter 边界，而不是展示框架能力堆叠。

### P1: Contribution Shape

- 把 post-v0.2 issue 切成小而明确的 author-DX / adapter / JS-action / docs / MCP-compat / diagnostics 任务。
- 新贡献必须声明是否改变 runtime contract、Manifest shape、adapter contract 或安全叙事。

## Out Of Scope

- Full TypeScript support。
- 从 TypeScript type、JSDoc、Zod、TypeBox 或 examples 自动生成 schema。
- `action.ts` 直接运行、`ts-node`、`tsx`、build pipeline 或 source map 支持。
- CJS / ESM 双模块兼容矩阵；v0.3 只认 canonical `action.mjs`。
- `skillrun init refund` 的隐式默认语言。
- `skillrun run --python`、`skillrun test --js` 等 Consumer Mode language flags。
- npm / pnpm / yarn install flow。
- Dependency vendoring、reproducible runtime image。
- HTTP / SSE / Streamable HTTP MCP transport。
- Marketplace、registry、install flow。
- OS sandbox。
- 多 action 编排。
- GUI。
- OpenAPI import。
- Agent framework abstraction。

这些不是永远不做，而是不进入 v0.3。v0.3 的稀缺资源必须用来验证 adapter boundary 和作者质量闭环，而不是接管 Node/TypeScript 生态。

## Success Criteria

- 新作者可以从 README 完成 `init -> manifest -> test -> serve --mcp`，并理解 SkillRun 与 FastMCP 的边界。
- Python capsule 仍是稳定主路径，JS capsule 作为 alpha path 跑通完整命令链。
- Rust Core 不再把 Python 当成唯一 action adapter 假设。
- `doctor/validate` 或等价诊断入口能在不运行 action 的前提下给出 adapter-aware 结论。
- JS schema 来自显式 JSON Schema export，不来自 TypeScript type inference。
- CLI 语言语义清楚：`init` 显式模板选择，`manifest` config/convention 解析，Consumer Mode 只认 Manifest。
- v0.3 没有扩大安全承诺，也没有把 `.skr` 描述成安全安装格式、依赖包或 runtime image。
- README、SSOT、release notes 和 issue drafts 对 v0.3 边界保持一致。

## Review Gate

- Approval: Confirmed for planning.
- Reviewer notes: 本 spec 定义方向与 CLI 语义，不授权直接实现。进入实现前需要 plan / tasks / packets 获得用户批准。
