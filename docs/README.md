# SkillRun 文档入口

本目录收敛 SkillRun 的公开项目文档和维护者策略。面向外部贡献者的入口放在仓库根目录，面向维护和治理的说明放在 `docs/`。

## 用户与贡献者入口

- [English README](../README.md)：项目定位、当前能力、快速命令和路线图。
- [中文 README](../README.zh-CN.md)：中文项目介绍和使用说明。
- [贡献指南](../CONTRIBUTING.md)：开发环境、分支、提交和 PR 规范。
- [安全政策](../SECURITY.md)：漏洞报告方式、支持版本和安全边界。
- [行为准则](../CODE_OF_CONDUCT.md)：社区协作基本规则。

## 项目设计与范围

- [MVP 合同](mvp.md)：MVP 范围、非目标和发布边界。
- [架构 SSOT](ssot.md)：核心架构、Manifest、runtime 和 adapter 约定。
- [Adapter Protocol](adapter-protocol.md)：v0.5 起的 Core-to-adapter 南向协议、capability levels、IPC/envelope 和 conformance 边界。
- [项目定位](positioning.md)：SkillRun 的最强公开定位、生态边界和 v0.4.2 叙事。
- [愿景](vision.md)：长期愿景、信任演进阶段和官方胶囊策略。
- [信任模型](trust-model.md)：当前可信边界、非承诺和 `.skr` 消费边界。
- [v0.4 Portable Consumer Checks](v0.4-portable-consumer-checks.md)：dependency-aware Consumer Mode、`check` 边界和 `.skr` 可诊断分发合同。
- [v0.4.1 WeCom Team Notice](v0.4.1-wecom-team-notice.md)：本地企业微信通知示例的 Skill Capsule 设计。
- [v0.4.2 官方参考胶囊](v0.4.2-official-capsules.md)：Commit Message Gate、Bounded File Patcher 和 Read-only Diagnostics Runner。
- [v0.4.3 CI 与 runtime 错误稳定化](v0.4.3-ci-stabilization.md)：修复 v0.4.2 合入后的 Linux CI 问题，并稳定缺失 metadata runtime 的跨平台错误文案。
- [v0.5 Adapter Protocol](v0.5-adapter-protocol.md)：语言无关 Adapter Protocol 和 Level 0 command adapter 的 v0.5 计划。
- [v0.5.1 Contract Stabilization](v0.5.1-contract-stabilization.md)：统一 guardrail 定义、trust model、官网叙事和 `output` envelope 字段。
- [v0.5.2 Consumer JSON Surface](v0.5.2-consumer-json-surface.md)：为 Desktop、Router 和自动化提供 `inspect/check/doctor --json` 的只读机器接口计划。
- [v0.5.3 Capsule Registry + Switchboard](v0.5.3-capsule-registry-switchboard.md)：为 v0.6 建立本地 capsule inventory 与 enable/disable 控制面计划。
- [v0.5.4 Core Stabilization Audit](v0.5.4-core-stabilization-audit.md)：进入 Desktop 前对整个 `skillrun` Core 的稳定化审核、风险清单和修复 work graph。
- [v0.5.5 Core Contract Hardening](v0.5.5-core-contract-hardening.md)：进入 Desktop / v0.6 前的 Manifest、schema 与 adapter lifecycle 合同硬化。
- [v0.5.5 Release Gate Review](v0.5.5-release-gate-review.md)：用命令合同矩阵确认 Consumer Mode 执行、MCP 暴露和 `.skr` 分发共享 Manifest 静态合同。
- [v0.5.5 发布后复盘](v0.5.5-post-release-review.md)：记录 v0.5.5 发布事实、远端 CI 暴露的问题、修复结论和 v0.5.6 推荐边界。
- [v0.5.6 CI Diagnostics Review](v0.5.6-ci-diagnostics-review.md)：复审 `cargo test` 失败 annotation 是否适合作为发布工程基线。
- [v0.5.6 Headless Consumer Contract](v0.5.6-headless-consumer-contract.md)：定义 Desktop / Router 前置的本地消费者控制面 JSON 合同与非 UI 边界。
- [v0.5.6 Run History Contract Review](v0.5.6-run-history-contract-review.md)：审查 Envelope Explorer 前置的 run history 查询合同、隐私边界和实现切分。
- [v0.5.6 Mount Plan Contract Review](v0.5.6-mount-plan-contract-review.md)：审查一键挂载的 plan-first 合同、Router 挂载边界和不写配置约束。
- [v0.5.6 Release Gate Review](v0.5.6-release-gate-review.md)：复审 v0.5.6 T001-T006 完成状态、边界、验证结果和 release decision 建议。
- [v0.5.6 Release Polish Plan](v0.5.6-release-polish-plan.md)：把 v0.5.6 拆成发布工程、CI 诊断和 headless consumer JSON surface 的可执行任务。
- [v0.5.7 Public Surface Plan](v0.5.7-public-surface-plan.md)：进入 Desktop 前收束 README、官网、Desktop handoff 和 release 叙事的公开表层计划。
- [v0.5.8 Router MVP](v0.5.8-router-mvp.md)：实现一键挂载所需的最小本地 MCP Router runtime，不引入 daemon、Desktop 或配置写入。
- [v0.5.9 Safe Mount Apply](v0.5.9-safe-mount-apply.md)：定义可逆、可审计的 MCP client config apply / rollback 边界。
- [v0.5.10 Consumer Contract Hardening](v0.5.10-consumer-contract-hardening.md)：计划在 Desktop 前校准公开文档、mount backup 合同和 consumer JSON 边界。
- [v0.5.11 Runs Inspect](v0.5.11-runs-inspect.md)：计划为 Desktop Envelope Explorer 提供单次 run evidence 查询合同。
- [v0.5.12 Capsule Import](v0.5.12-capsule-import.md)：计划补齐 `.skr import` 的 Core 合同，让 Desktop 通过稳定 CLI JSON 导入 capsule。
- [v0.6 Consumer Era 愿景](v0.6-consumer-era-vision.md)：本地消费者控制面、SkillRun Router、一键 MCP 挂载、Tauri/Desktop 边界和官方领域包策略。
- [经典业务示例](business-examples.md)：SOP-backed capability 的业务样例。

## 维护者流程

- [测试策略](testing.md)：本地验证、CI 检查和 release validation。
- [发布策略](release-policy.md)：版本号、release candidate、tag 和发布流程。
- [Release Checklist](release-checklist.md)：基于 v0.5.5 发布复盘固化的逐步发布检查清单。
- [分支保护建议](branch-protection.md)：公开前后 `main` 分支保护规则。

## 文档维护规则

- 项目治理文档默认使用中文。
- README、issue template、PR template 等公开协作入口可以使用英文，以降低开源协作门槛。
- 文档必须明确区分当前已实现能力、release candidate 行为和计划能力。
- 当运行时契约、CLI 行为或安全边界变化时，同步更新 README、相关 docs 和 release notes。
