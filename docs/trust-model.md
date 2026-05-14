# SkillRun Trust Model

**文档状态**：Ready_For_User_Review
**版本**：v0.4.2 trust baseline
**最后更新**：2026-05-14

---

## 核心判断

SkillRun 提供的是 **manifest-bound execution and inspection layer**，不是 OS sandbox。

它能减少 Agent 裸调 action 的风险，但不能把任意第三方代码变成安全代码。运行别人分发的 action 仍然意味着执行别人写的代码。

## 当前可信边界

v0.4.2 可以诚实承诺以下边界：

- Consumer Mode 不为 metadata extraction 动态 import 未信任源码。
- Manifest 缺失、过期或 source hash 不匹配时 fail closed。
- `inspect` 展示 Manifest contract、schema、permissions、adapter、entrypoint 和 source hashes。
- `check` 从 Manifest 和 host runtime probes 诊断 readiness，不安装依赖。
- Core 只注入 Manifest 声明过的业务 env。
- stdout/stderr 只作为日志；结构化结果必须来自 output/error envelope。
- artifact path 必须留在 run-local artifact directory 内。
- structured errors 为 Agent 提供可恢复行为建议。

## 当前不承诺

v0.4.2 不承诺：

- 完整 OS sandbox。
- 网络 egress 强制隔离。
- 文件系统强制 sandbox。
- 恶意 action 检测。
- signed capsule。
- registry 审核。
- dependency vendoring。
- reproducible runtime image。
- 自动安装 Python、Node、Pydantic 或 npm packages。

权限声明是运行合同、inspect/check 信息和部分 runtime 边界，不是完整系统级强制隔离。

## Author Mode

Author Mode 面向本地作者。作者通常信任自己的源码，因此 SkillRun 可以为了生成 Manifest 而运行 adapter metadata phase。

Author Mode 允许：

- convention discovery。
- metadata extraction。
- Manifest regeneration。
- local test/run。

Author Mode 约束：

- metadata phase 不注入业务 secrets。
- metadata phase 应有 timeout。
- action module 顶层应 side-effect free。

## Consumer Mode

Consumer Mode 面向使用别人分发的 capsule。

Consumer Mode 只信任：

- 已生成 Manifest。
- source hashes。
- runtime requirements。
- declared permissions。
- examples 和 static capsule structure。

Consumer Mode 不做：

- 为了提取 schema import `action.py` 或 `action.mjs`。
- 在 Manifest stale 时猜测运行方式。
- 从 stdout 推断成功。

## `.skr` 信任边界

`.skr` 是 source + Manifest archive。

它不是：

- secure install format。
- registry package。
- dependency bundle。
- runtime image。
- signed artifact。

正确消费路径：

```bash
skillrun inspect --cwd <capsule>
skillrun check --cwd <capsule>
skillrun test --cwd <capsule>
```

然后再决定是否让 Agent 通过 MCP 调用。

## 官方参考胶囊的边界

官方示例胶囊用于展示 SkillRun 的约束模型，不代表完整安全产品。

- `commit_message_gate` 默认只验证 commit message，不自动 stage。
- `bounded_file_patcher` 只做精确 old-text/new-text replacement，不是 OS sandbox。
- `readonly_diagnostics_runner` 只接受 allowlisted diagnostics，不接受任意 shell command。

这些示例的价值在于：把 prompt 里的规则移到 schema、preflight、Manifest 和 run evidence 中。
