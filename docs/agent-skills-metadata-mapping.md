# Agent Skills Metadata Mapping

**文档状态**：design proposal  
**最后更新**：2026-05-22  
**范围**：设计 `SKILL.md` frontmatter 与 SkillRun Manifest 的兼容映射；本文不改变当前 Manifest schema。

## 背景

Agent Skills 的最小单元是一个包含 `SKILL.md` 的目录。`SKILL.md` 可以通过 YAML frontmatter 提供 `name`、`description`、`license`、`compatibility`、`metadata`，以及实验性的 `allowed-tools` 等字段，帮助 agent client 做发现、激活和上下文加载。

SkillRun 的最小可执行单元是 Skill Capsule。它可以兼容 Agent Skills 的 authoring / discovery 心智，但运行权威仍必须来自 `skillrun.config.json`、adapter metadata phase 和 generated Manifest。

因此，本设计遵守一个边界：

```text
SKILL.md frontmatter = authoring / discovery metadata
skillrun.config.json = SkillRun runtime configuration
Manifest             = compiled runtime contract
```

## 设计目标

- 保留 Agent Skills frontmatter 的语义，避免 fork 标准。
- 允许 `skillrun manifest` 未来把兼容 metadata 编译进 Manifest。
- 保持 Consumer Mode 只信任 Manifest，不在消费侧重新解析 `SKILL.md` 推断运行行为。
- 防止 `allowed-tools`、`metadata` 等 authoring hint 被误用为 SkillRun runtime permission。
- 让 instruction-only Agent Skills 可以被识别，但不会被当成 runnable capsule。

## 非目标

- 不要求 Agent Skills 作者写 SkillRun 专用 frontmatter。
- 不从 `SKILL.md` frontmatter 推断 action entrypoint、adapter、schema、timeout、环境变量或网络权限。
- 不把 `allowed-tools` 转译成 SkillRun runtime permissions。
- 不扫描 `scripts/`、`references/`、`assets/` 或 Markdown code block 来推断可执行入口。
- 不引入 marketplace trust、publisher verification、签名或 OS sandbox 承诺。

## 字段映射

| `SKILL.md` frontmatter | Agent Skills 语义 | SkillRun 映射策略 | Runtime 权威 |
| --- | --- | --- | --- |
| `name` | 发现阶段的技能名或展示名 | 可作为 Manifest `skill.name` 的候选值；必须通过 SkillRun tool name / package name 校验，不合法时回退到目录名并报告诊断 | 否 |
| `description` | 发现阶段的激活摘要 | 可作为 Manifest 中未来的 `skill.description` 或 `tool.description` 来源；缺失时继续从 SOP 首段生成摘要 | 否 |
| `license` | 包内容授权提示 | 可作为 Manifest 中未来的 package / provenance metadata 保留；不代表作者身份、签名或安全可信 | 否 |
| `compatibility` | Agent client 或 skill spec 兼容提示 | 可原样保存在 Manifest 的 Agent Skills metadata 区域，供 inspect / pack / registry 展示；不决定 SkillRun runtime 是否可执行 | 否 |
| `metadata` | 扩展 metadata | 可作为 opaque object 保留，用于 discovery、分类、homepage、tags 等；其中的 SkillRun runtime 字段默认忽略或诊断为不生效 | 否 |
| `allowed-tools` | Agent client 的工具使用 hint | 只作为 authoring hint 保留；不得映射为 `permissions.files`、`permissions.network` 或 `permissions.env` | 否 |

SkillRun runtime 权威字段继续来自：

| SkillRun 字段 | 来源 | 说明 |
| --- | --- | --- |
| `runtime.adapter` | `skillrun.config.json` 或 convention | 决定使用 Python、Node、command 等 adapter |
| `runtime.entrypoint` | `skillrun.config.json` 或 convention | 决定显式 action 文件或 command entrypoint |
| `runtime.command` | `skillrun.config.json` | 仅 command adapter 使用 |
| `runtime.timeout` | `skillrun.config.json` 或默认值 | 执行超时 |
| `schemas.input` / `schemas.output` | `skillrun.config.json` 或 adapter metadata phase | 输入输出合同 |
| `permissions` | `skillrun.config.json` 或默认值 | 文件、网络和环境变量权限声明 |
| `sources.*.sha256` | generated Manifest | Consumer Mode freshness 和 fail-closed 依据 |

## 优先级

生成 Manifest 时，推荐采用下列优先级：

1. `skillrun.config.json` 负责 runtime、permissions、schemas override、timeout 和 adapter 相关字段。
2. Adapter metadata phase 只负责语言生态可提取的 schema / capability metadata，不定义 SkillRun 产品语义。
3. `SKILL.md` frontmatter 负责 discovery metadata，例如 `name`、`description`、`license`、`compatibility`、`metadata`、`allowed-tools`。
4. `SKILL.md` 正文继续负责 SOP、适用边界、禁止行为、恢复建议和 agent-facing workflow。
5. Generated Manifest 记录最终解析结果和 source hashes；Consumer Mode 只读取 Manifest。

如果 `SKILL.md` frontmatter 和 `skillrun.config.json` 同时表达 runtime 意图，以 `skillrun.config.json` 为准，并在 `doctor` 或 `manifest` 诊断中提示 frontmatter 中的 runtime-like 字段不生效。

## Instruction-only Skill 行为

只有 `SKILL.md` 且没有显式 action / config / Manifest 的目录，应被视为 instruction-only Agent Skill：

- `inspect` 可以报告这是 instruction-only skill。
- `check` / `doctor` 可以解释缺少 action、Manifest 或 runtime contract。
- `run`、`test`、`pack`、`router serve --mcp` 不应把它当成 runnable capsule。
- `skillrun manifest` 不应从 Markdown、`scripts/` 或 `allowed-tools` 猜测 action。

这保持了 Agent Skills 的可用性，同时维护 SkillRun 的 runtime fail-closed 边界。

## Manifest 形态建议

当前 Manifest schema 不包含 Agent Skills metadata 区域。后续如要实现，应走单独 PR 和 Manifest schema review gate。

推荐的兼容性形态是加法字段，而不是改写现有 runtime 字段：

```yaml
skill:
  name: refund
  description: Apply refund policy checks before creating a refund request.
  sop_summary: Refund policy workflow.
  skill_hash: "..."
  agent_skill:
    frontmatter:
      license: Apache-2.0
      compatibility:
        agentskills: ">=0.1"
      metadata:
        tags:
          - finance
      allowed_tools:
        - bash
```

约束：

- `skill.agent_skill.frontmatter` 是 preserved metadata，不是 runtime contract。
- `tool.name` 和 `tool.description` 可以从 validated `name` / `description` 派生，但必须保留现有校验和 fallback。
- `permissions` 仍只来自 `skillrun.config.json` 或 SkillRun 默认值。
- 新字段进入 Manifest 前，需要同步更新 inspect/check/pack/registry/Router contract 影响面说明。

## 实现阶段

### P1: 只读解析与诊断

- `skillrun manifest` 在 Author Mode 解析 `SKILL.md` YAML frontmatter。
- 支持基础 YAML object；frontmatter 格式错误时给出清晰诊断。
- 暂不改变 Consumer Mode。
- 对 runtime-like frontmatter 字段给出“不生效”的诊断。

### P2: Manifest 加法字段

- 经 Manifest schema review gate 后，把兼容 metadata 编译到 Manifest。
- 更新 `inspect --json` 和 human inspect 以展示 Agent Skills metadata。
- 保持旧 Manifest 可读；缺少该字段不影响 Consumer Mode。

### P3: Discovery 体验

- 用 validated `description` 改善 MCP tool description 和 registry / exposure 展示。
- 为 instruction-only Agent Skills 提供更友好的 inspect / doctor 输出。
- 仍不把 instruction-only skill 暴露为 runnable MCP tool。

## 验收标准

- 兼容 Agent Skills 标准字段，不要求标准外 SkillRun frontmatter。
- Runtime 行为只由 `skillrun.config.json`、adapter metadata phase 和 Manifest 决定。
- Consumer Mode 不重新解析 `SKILL.md` 来改变运行行为。
- `allowed-tools` 不等同于 SkillRun permissions。
- Instruction-only skill 不会被隐式执行、打包或暴露。
