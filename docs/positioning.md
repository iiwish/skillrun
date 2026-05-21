# SkillRun 项目定位

**文档状态**：Ready_For_User_Review
**版本**：v0.4.2 positioning baseline
**最后更新**：2026-05-14

---

## 一句话定位

**SkillRun 是可执行 Agent Skills 的 runtime 与打包工具链：把一份 SOP、一个类型化 Action、schema、examples、permissions 和 preflight 编译成可检查、可测试、可运行、可分发、可通过 MCP 调用的 Skill Capsule。**

英文表达：

> SkillRun is a runtime and packaging toolchain for executable Agent Skills.

更完整：

> SkillRun compiles `SKILL.md`, typed action code, schemas, examples, permissions and preflight checks into a portable Skill Capsule that can be inspected, checked, run, packed, evidenced, and exposed as MCP tools.

## 生态位置

Agent Skills 已经成为“给 agent 增加能力”的事实标准。SkillRun 不应该把自己讲成另一套 Skills 标准，而应该站在 Agent Skills 的心智之上，补足可执行技能需要的 runtime contract。

```text
Agent Skills = Agent 如何发现和学习一个能力
MCP          = Agent 如何调用外部能力
SkillRun    = 可执行能力如何被检查、运行、打包、留证和挂载
```

一句话：

```text
Agent Skills 让技能可移植。
SkillRun 让可执行技能可检查、可运行、可追溯。
```

完整边界见 [Agent Skills Compatibility](agent-skills-compatibility.md)。

## 核心对比

```text
Agent Skills package instructions.
FastMCP exposes functions.
SkillRun runs executable skills with contracts.
```

更精确地说：

```text
Agent Skills turn knowledge into portable agent context.
FastMCP turns code into tools.
SkillRun turns SOP-backed actions into portable, testable, evidenced skills.
```

SkillRun 不抢“定义 Skills 格式”的战场，也不抢“把函数暴露成 tool”的战场。它解决的是另一个问题：当 Agent 调用一个真实业务动作时，SOP、输入输出结构、前置边界、失败恢复、产物和审计证据必须跟 action 一起移动。

## 产品原子

SkillRun 的产品原子是 **Skill Capsule**，不是单个函数、单个 Markdown 文件、MCP server 或 YAML 配置。

```text
Skill Capsule = SOP + action code + schema + examples + permissions
Manifest      = compiled runtime contract
Core          = Rust manifest-driven runtime
Adapter       = language bridge for user actions
Package       = .skr source + Manifest archive
```

一个 Capsule 对 Agent 是一个可调用技能；对作者是一个小目录；对 Consumer Mode 是一份静态 Manifest 加 source hashes；对 MCP client 是一个 Manifest-derived tool 和相关 resources。

## SkillRun 是什么

- Manifest-driven runtime for SOP-backed agent skills。
- 可执行 Agent Skills 的 runtime / contract / evidence 层。
- 本地优先的 CLI/Core，用 Rust 实现。
- 把 `SKILL.md`、action、schema、examples、permissions 和 preflight 编译成 Manifest。
- 用 Manifest 生成 inspect/check/run/test/pack/MCP 暴露路径。
- 用 structured output/error envelope、artifact 和 run record 保留执行证据。
- 用 Consumer Mode 避免为 metadata extraction 动态 import 未信任源码。

## SkillRun 不是什么

- 不是 FastMCP 替代品。
- 不是 Agent Skills 替代标准。
- 不是通用 Agent framework。
- 不是任意 Markdown 自动执行器。
- 不是 OpenAPI-to-MCP 包装器。
- 不是 marketplace 或 registry 的第一版。
- 不是完整 OS sandbox。
- 不是 dependency bundle、runtime image 或 secure install format。
- 不是把 YAML 暴露给用户手写的 action runtime。

## 为什么不是 Docker Engine

Docker 类比有传播价值，但不适合作为主定位。Docker 暗示镜像、强隔离、依赖封装、可复现运行环境和 registry 生态。SkillRun v0.4.2 的真实边界是：

- Manifest-bound execution。
- Consumer-side static checks。
- Structured run records。
- Source + Manifest `.skr` archive。
- Honest trust model without OS sandbox claims。

因此更准确的类别是：

> Manifest-driven runtime and packaging toolchain for SOP-backed agent skills.

## MCP 的位置

MCP 是 SkillRun 的北向 invocation surface，不是 SkillRun 的全部身份。

```text
Agent / MCP client
      |
      v
Manifest-derived MCP tool
      |
      v
SkillRun Core reads Manifest
      |
      v
Adapter runs action through IPC
```

SkillRun 的长期边界是 Manifest 和 Adapter Protocol；MCP 是当前最重要的对外调用接口之一。

## Agent Skills 的位置

Agent Skills 是 SkillRun 应该兼容的 authoring / discovery 层。普通 instruction-only skill 不需要 SkillRun；只有当 skill 包含真实可执行 action，并且需要 schema、preflight、consumer checks、run evidence、`.skr` packaging 或 Router exposure 时，SkillRun 才进入。

```text
Agent Skill   = SKILL.md + scripts/references/assets
Skill Capsule = Agent Skill-compatible SOP + action + Manifest + runtime evidence
```

SkillRun-specific runtime 配置应优先放在 `skillrun.config.json` 或 generated Manifest 中，而不是强行污染 `SKILL.md` 的 Agent Skills 标准字段。

## v0.4.2 定位

v0.4.2 是文档与示例型 patch release。它不改变 runtime 架构，不引入新 adapter，不引入 registry，也不扩大安全承诺。

v0.4.2 交付重点：

- 明确项目定位、愿景和 trust model。
- 增加官方参考胶囊，展示 SOP-backed preflight 的通用价值。
- 保持 README 面向当前状态，愿景文档单独承载长期叙事。
- 为 v0.5 的 language-agnostic Adapter Protocol 讨论留出架构入口。
