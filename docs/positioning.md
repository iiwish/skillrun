# SkillRun 项目定位

**文档状态**：Ready_For_User_Review
**版本**：v0.4.2 positioning baseline
**最后更新**：2026-05-14

---

## 一句话定位

**SkillRun 是 Manifest 驱动的 Skill Capsule runtime：把一份 SOP、一个类型化 Action、schema、examples、permissions 和 preflight 编译成可检查、可测试、可运行、可分发、可通过 MCP 调用的 Agent skill。**

英文表达：

> SkillRun packages SOP-backed actions into tested, manifest-bound skills for AI agents.

更完整：

> SkillRun compiles `SKILL.md`, typed action code, schemas, examples, permissions and preflight checks into a portable Skill Capsule that can be inspected, checked, run, packed and exposed as MCP tools.

## 核心对比

```text
FastMCP exposes functions.
SkillRun ships skills.
```

更精确地说：

```text
FastMCP turns code into tools.
SkillRun turns SOP-backed actions into portable, testable skills.
```

SkillRun 不抢“把函数暴露成 tool”的战场。它解决的是另一个问题：当 Agent 调用一个真实业务动作时，SOP、输入输出结构、前置边界、失败恢复、产物和审计证据必须跟 action 一起移动。

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

## Context Skill 与 Runtime Capsule

不要把所有 `SKILL.md` 都叫 SkillRun Capsule。

```text
Context Skill    = SKILL.md + references/scripts/templates，由 Codex、Claude Code 等 Agent 加载
Runtime Capsule  = SKILL.md + action + schema + examples + permissions + Manifest，由 SkillRun 执行
```

Context Skill 的价值是让 Agent 获得工作流、判断标准和上下文。它不需要 SkillRun runtime，不生成 Manifest-derived MCP tool，也不应该进入 `switchboard enable`。

Runtime Capsule 的价值是把一个真实动作放进可检查、可测试、可拒绝、可审计的执行合同。只有作者显式提供 action、schema、examples 和 Manifest 后，普通 Skill 才升级为 SkillRun Capsule。

## SkillRun 是什么

- Manifest-driven runtime for SOP-backed agent skills。
- 本地优先的 CLI/Core，用 Rust 实现。
- 把 `SKILL.md`、action、schema、examples、permissions 和 preflight 编译成 Manifest。
- 用 Manifest 生成 inspect/check/run/test/pack/MCP 暴露路径。
- 识别 instruction-only Skill，并拒绝把 Markdown、scripts、references、assets 或 examples 推断成可执行 action。
- 用 structured output/error envelope、artifact 和 run record 保留执行证据。
- 用 Consumer Mode 避免为 metadata extraction 动态 import 未信任源码。

## SkillRun 不是什么

- 不是 FastMCP 替代品。
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

## v0.4.2 定位

v0.4.2 是文档与示例型 patch release。它不改变 runtime 架构，不引入新 adapter，不引入 registry，也不扩大安全承诺。

v0.4.2 交付重点：

- 明确项目定位、愿景和 trust model。
- 增加官方参考胶囊，展示 SOP-backed preflight 的通用价值。
- 保持 README 面向当前状态，愿景文档单独承载长期叙事。
- 为 v0.5 的 language-agnostic Adapter Protocol 讨论留出架构入口。
