# SkillRun 愿景

**文档状态**：Ready_For_User_Review
**版本**：v0.4.2 vision baseline
**最后更新**：2026-05-14

---

## 愿景判断

Agent 时代真正稀缺的不是“更多 tool”，而是能被 Agent 调用、能被人审查、能被团队分发、能被运行时约束的业务能力单元。

SkillRun 的长期愿景是让团队可以把经验封装成 **Skill Capsule**：

```text
一份 SOP
+ 一个类型化 Action
+ 明确 schema
+ preflight 硬边界
+ examples
+ permissions
+ structured evidence
= 一个可分发、可检查、可运行的 Agent skill
```

## 长期目标

SkillRun 希望成为 SOP-backed agent skills 的本地优先基础设施：

- 作者可以低摩擦地把团队流程变成 Skill Capsule。
- 消费者可以在运行前 inspect/check/test，而不是盲目信任源码。
- Agent 可以通过 MCP 或其他 invocation surface 调用同一份 Manifest-bound skill。
- 团队可以用 run records、artifacts 和 structured errors 追溯每次调用。
- 社区可以复用、审查和演进参考胶囊，而不是复制脆弱 prompt。

## 不可替代性

SkillRun 的不可替代性不来自“一键把函数变 tool”，而来自少数稳定概念：

- **Skill Capsule**：业务能力的分发原子。
- **Manifest-bound execution**：运行时只按编译后的合同执行。
- **SOP-backed action**：自然语言流程和执行代码必须一起出现。
- **Preflight as hard boundary**：关键业务边界进入代码级预检。
- **Consumer Mode checks**：消费第三方 skill 前先做静态诊断。
- **Structured run evidence**：结果、错误、artifact 和日志可复查。

## 信任演进阶段

SkillRun 的安全叙事必须分阶段，不能把未来能力伪装成当前能力。

### v0.4.x：Manifest-bound skill runtime

当前阶段提供：

- Manifest freshness 和 source hash 检查。
- Consumer Mode 不为 metadata 动态 import 源码。
- declared env injection。
- artifact path containment。
- structured output/error envelope。
- dependency-aware `check`。
- `.skr` source + Manifest archive。

当前阶段不提供：

- OS sandbox。
- signed package。
- dependency vendoring。
- reproducible runtime image。
- registry trust。

### v0.5：Language-agnostic Adapter Protocol

v0.5 可以把“支持语言”收敛成 Adapter Protocol，而不是 Core 内置语言生态：

```text
Core: Rust runtime, only understands Manifest + IPC + envelopes
Adapter Protocol: language-neutral metadata/run contract
SDK Wrapper: language-specific authoring convenience
```

Python 和 JS 是官方 adapter path；Ruby、PHP、Go 等未来语言应通过同一份 protocol 接入，而不是改变 Core。

### Future：stronger trust distribution

后续可以探索：

- signed capsules。
- registry trust metadata。
- stronger dependency profiles。
- sandboxed runtime。
- reproducible capsule environment。

这些是未来能力，不是 v0.4.2 的承诺。

## 社区传播方式

SkillRun 最容易被社区复述的表达应该短而准确：

```text
FastMCP exposes functions. SkillRun ships skills.
```

或者：

```text
Move agent safety rules out of fragile prompts and into testable Skill Contracts.
```

中文：

```text
把脆弱 prompt 里的工具使用规则，移进可检查、可测试、可运行拦截的 Skill Contract。
```

## 官方胶囊策略

官方参考胶囊不应该被称为 marketplace 或 store。v0.4.2 使用 **Official Example Capsules** 或 **Capsule Gallery**。

第一批参考胶囊应优先证明 SkillRun 的核心，而不是证明它能包一切：

- `commit_message_gate`：把提交规范变成 preflight。
- `bounded_file_patcher`：把文件修改边界变成精确 patch contract。
- `readonly_diagnostics_runner`：把高风险 shell 需求缩小成 allowlisted diagnostics。

这些示例展示的是 Manifest-bound contracts，不是完整安全平台。
