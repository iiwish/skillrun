# SkillRun v0.3 Feature Spec

Version: v0.3
Status: Ready_For_User_Review
Created: 2026-05-12
Source: post-v0.2 direction review

## 一句话判断

v0.3 不应该横向扩展协议或运行时，而应该验证第二个核心假设：

> 陌生作者能否低摩擦地写出、诊断并改进一个高质量 Skill Capsule。

v0.2 证明外部调用路径成立；v0.3 要证明 authoring quality loop 成立。

## 北极星

**v0.3 Authoring Quality Loop**

让作者在本地完成这条闭环：

```text
init -> edit SOP/action -> manifest -> validate/doctor -> test -> inspect -> improve
```

目标不是让 SkillRun 做更多类型的事情，而是让同一个 Skill Capsule 模型更容易被真实开发者正确使用。

## 设计原则

- 继续把 `Manifest` 作为 runtime IR，不把它变成默认手写入口。
- 继续保持 Rust Core + Python action blessed path。
- 继续避免 sandbox、registry、marketplace、HTTP server、Node adapter。
- 优先改善错误信息、诊断路径、模板质量和作者反馈，而不是增加执行能力。
- 所有新能力必须加强 `Skill Capsule = SOP + action + schema + examples + permissions` 这个心智模型。

## In Scope

### P0: Author Diagnostics

- 改善 stale Manifest、缺失 schema、缺失 examples、instruction-only 与 permission/artifact 错误的解释。
- 设计并实现一个最小诊断入口，候选命令为 `skillrun doctor` 或 `skillrun validate`。
- 诊断必须不执行业务 action；它只检查 capsule structure、Manifest freshness、schema/example consistency 和 release-facing warnings。

### P0: Golden Author Path

- 收紧 `skillrun init --python` 模板，使默认 capsule 更接近真实业务 skill，而不是玩具函数。
- README Quickstart 必须能让新作者 10 分钟内完成第一个 capsule。
- 错误恢复路径要明确告诉作者下一步运行哪个命令。

### P1: Manifest Explanation

- 提供更清楚的 Manifest-derived contract 解释，可能通过 `inspect` 增强或新命令实现。
- 重点解释 tool schema、resource exposure、permissions、source hashes 和 run evidence。

### P1: Example Quality

- 继续保留 `refund` 作为唯一必须完整运行的 hero example。
- 可新增一个 docs-level 或 lightweight capsule example，但不得引入真实外部 API、密钥或网络依赖。
- 示例目标是解释业务边界，而不是展示框架能力堆叠。

### P1: Contribution Shape

- 把 post-v0.2 issue 切成小而明确的 author-DX / docs / MCP-compat / diagnostics 任务。
- 新贡献必须声明是否改变 runtime contract、Manifest shape 或安全叙事。

## Out Of Scope

- Node adapter。
- HTTP / SSE / Streamable HTTP MCP transport。
- Marketplace、registry、install flow。
- Signed package、dependency vendoring、reproducible runtime image。
- OS sandbox。
- 多 action 编排。
- GUI。
- OpenAPI import。
- Agent framework abstraction。

这些不是永远不做，而是不进入 v0.3。v0.3 的稀缺资源必须用来降低作者写出好 capsule 的摩擦。

## Success Criteria

- 新作者可以从 README 完成 `init -> manifest -> test -> serve --mcp`，并理解 SkillRun 与 FastMCP 的边界。
- 常见错误不需要读源码就能恢复。
- `doctor/validate` 或等价诊断入口能在不运行 action 的前提下给出有用结论。
- v0.3 没有扩大安全承诺，也没有把 `.skr` 描述成安全安装格式。
- README、release notes 和 issue drafts 对 v0.3 边界保持一致。

## Review Gate

- Approval: Pending
- Reviewer notes: 本 spec 只定义方向，不授权直接实现。进入实现前需要 plan / tasks / packets。
