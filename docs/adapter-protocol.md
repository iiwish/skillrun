# SkillRun Adapter Protocol

**文档状态**：v0.5.0 confirmed contract
**版本**：adapter.v1 draft for v0.5.0 implementation
**最后更新**：2026-05-14

---

## 一句话判断

Adapter Protocol 是 SkillRun Core 与 action 进程之间的南向执行契约：Core 只认 Manifest、IPC、Envelope 和 Artifact；语言生态通过 adapter 接入，但不能反向定义 SkillRun 的产品语义。

这份文档不是“新增一门语言”的说明，而是定义未来所有语言接入必须遵守的最小边界。

## 边界模型

```text
MCP / CLI / Agent Framework
        |
        v
SkillRun Core
  - Rust runtime
  - reads Manifest
  - validates schemas, permissions, source hashes
  - creates IPC files and artifact directory
  - validates output/error/artifact envelopes
        |
        v
Adapter Protocol
  - metadata phase
  - run phase
  - runtime requirement diagnostics
  - stdout/stderr discipline
        |
        v
Language Adapter / Command Adapter
        |
        v
Action code
```

Core 的稳定职责：

- 读取 Manifest。
- 准备 `SKILLRUN_CONTEXT_JSON`、`SKILLRUN_INPUT_JSON`、`SKILLRUN_OUTPUT_JSON`、`SKILLRUN_ARTIFACT_DIR`。
- 启动 adapter 或 command 进程。
- 校验 output/error envelope。
- 校验 artifact path 不能逃逸 artifact directory。
- 记录 run evidence。

Adapter 的稳定职责：

- 在 metadata phase 中报告 schema、capability 和 runtime requirements。
- 在 run phase 中读取 IPC 输入，执行 action，并写入 envelope。
- 将 stdout/stderr 当作日志，而不是结构化结果。
- 把 adapter/language/package-manager 细节留在 adapter 层，不污染 Core。

SDK 的稳定职责：

- 降低作者实现 `preflight`、`run`、typed I/O 和 artifact 的摩擦。
- 不成为 runtime authority。
- 不绕过 Manifest、IPC 或 envelope contract。

## Capability Levels

### Level 0: Command Adapter

Level 0 是 v0.5.0 必须实际实现的最小语言无关路径。

约束：

- `runtime.adapter = "command"`。
- `runtime.command` 必须是 argv array，不是 shell string。
- input/output schema 来自 config 或 Manifest。
- 不进行动态 metadata import。
- Consumer Mode 只信任 Manifest。
- command 进程必须使用标准 IPC 环境变量。
- command 进程必须写 `SKILLRUN_OUTPUT_JSON`。
- stdout/stderr 只作为日志。
- readiness 只诊断 executable 是否存在，不安装依赖。

Level 0 不是 Ruby/PHP/Go 官方支持，也不是通用 shell runner。它证明的是：任何进程只要遵守 SkillRun IPC/envelope contract，就可以被 Core 以语言无关方式调用。

### Level 1: Community Adapter

Level 1 是社区维护的语言 adapter。

要求：

- 实现 metadata phase 和 run phase。
- 通过 adapter conformance fixtures。
- 明确 runtime requirements。
- 可以提供有限 SDK ergonomics。
- 不承诺由 SkillRun core team 长期维护。

### Level 2: Blessed Adapter

Level 2 是官方维护的 adapter。

要求：

- 通过 conformance fixtures。
- 有模板、文档、release tests 和 readiness diagnostics。
- 在 README 和 release notes 中明确列为稳定或 alpha。
- 维护者愿意承担兼容性与回归风险。

## Metadata Phase

Metadata phase 只存在于 Author Mode，用于生成或刷新 Manifest。

输入：

- capsule root。
- action entrypoint。
- adapter runtime config。
- 不包含 secrets。

输出：

- input schema。
- output schema。
- adapter capability。
- runtime requirements。
- preflight support marker。
- artifact support marker。

约束：

- 必须有 timeout。
- 不执行业务 action。
- 不注入生产 secrets。
- 失败应返回可诊断错误，而不是让 Core 猜测 schema。

Consumer Mode 不运行 metadata phase。Consumer Mode 只读取 Manifest，并在 Manifest 缺失、过期或 hash 不匹配时 fail closed。

## Run Phase

Run phase 是 Core 调用 adapter/action 的执行路径。

Core 创建：

```text
SKILLRUN_CONTEXT_JSON
SKILLRUN_INPUT_JSON
SKILLRUN_OUTPUT_JSON
SKILLRUN_ARTIFACT_DIR
```

Adapter 必须：

- 读取 context 和 input。
- 执行 preflight 和 action。
- 把成功或失败写入 `SKILLRUN_OUTPUT_JSON`。
- 将 artifact 写入 `SKILLRUN_ARTIFACT_DIR` 内。
- 不把 stdout 当作 result。

成功 envelope：

```json
{
  "ok": true,
  "result": {},
  "artifacts": [],
  "display": {
    "markdown": "Done."
  }
}
```

失败 envelope：

```json
{
  "ok": false,
  "error": {
    "code": "PolicyViolation",
    "message": "Blocked by preflight policy.",
    "llm_hint": "Ask the user for approval before retrying.",
    "recoverable": true,
    "details": {}
  }
}
```

如果 adapter 成功退出但未写 output file，Core 必须返回 `ProtocolViolation`。

## Runtime Requirements

Runtime requirements 是诊断合同，不是安装计划。

允许表达：

- executable 名称和版本期望。
- package 名称和版本期望。
- requirements 用于 metadata、runtime 或两者。

禁止承诺：

- 自动安装。
- 自动创建 virtualenv、node_modules、bundle、composer vendor 或 Go module cache。
- dependency vendoring。
- reproducible runtime image。

缺失依赖应映射为 `DependencyError`，并带有可读的恢复建议。

## Conformance Fixtures

v0.5.0 起，adapter 行为应被 conformance fixtures 描述，而不是只靠文档约定。

最小覆盖：

- success envelope 写入 `SKILLRUN_OUTPUT_JSON`。
- invalid input 在执行前映射为 `ValidationError`。
- preflight rejection 映射为 `PolicyViolation`。
- missing executable 映射为 `DependencyError`。
- malformed output 映射为 `ProtocolViolation`。
- command exits zero without output file 映射为 `ProtocolViolation`。
- stdout text 不会成为 result。
- artifact path traversal 被拒绝。
- Consumer Mode 不 import source 以提取 metadata。

## Manifest Fields

v0.5.0 推荐 Manifest 记录：

```yaml
runtime:
  adapter: command
  protocol_version: adapter.v1
  command:
    - ruby
    - action.rb
```

作者侧 `skillrun.config.json` 可以声明：

```json
{
  "runtime": {
    "adapter": "command",
    "command": ["ruby", "action.rb"],
    "timeout": "30s"
  },
  "input_schema": {
    "type": "object",
    "required": ["name"],
    "additionalProperties": false,
    "properties": {
      "name": { "type": "string" }
    }
  },
  "output_schema": {
    "type": "object",
    "required": ["message"],
    "additionalProperties": false,
    "properties": {
      "message": { "type": "string" }
    }
  }
}
```

`command` 必须是 argv array。SkillRun 不应把一个 shell string 交给系统 shell 解释。

## Security Claims

Adapter Protocol 提供的是执行边界和诊断边界，不是 sandbox。

可以诚实声明：

- Consumer Mode 不为 metadata import 未信任源码。
- stdout/stderr 不被当作成功结果。
- output/error/artifact contract 由 Core 校验。
- 缺失依赖以结构化错误暴露。
- artifact path 受到 Core 校验。

不能声明：

- 任意第三方 action 是安全的。
- command adapter 是 sandbox。
- `.skr` 是 secure install format。
- dependency readiness 等于隔离执行。

SkillRun 的安全叙事必须保持克制：它减少隐式执行、隐式成功和隐式信任，但运行第三方 action 仍然意味着执行第三方代码。

## v0.5.0 Scope

v0.5.0 应交付：

- 公开 Adapter Protocol 文档。
- Python stable 和 JS alpha 行为到 protocol 的映射。
- adapter conformance tests 的第一批 fixtures。
- Level 0 command adapter 的 Manifest/config 支持。
- Level 0 command adapter runtime。
- 一个不暗示新 blessed language 的 command adapter 示例。

v0.5.0 不做：

- 完整 TypeScript runtime。
- package-manager install flow。
- dependency vendoring。
- registry 或 marketplace。
- OS sandbox。
- 新 MCP transport。
- `.skr` runtime image。
