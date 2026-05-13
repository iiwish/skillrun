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
- [v0.4 Portable Consumer Checks](v0.4-portable-consumer-checks.md)：dependency-aware Consumer Mode、`check` 边界和 `.skr` 可诊断分发合同。
- [v0.4.1 WeCom Team Notice](v0.4.1-wecom-team-notice.md)：本地企业微信通知示例的 Skill Capsule 设计。
- [经典业务示例](business-examples.md)：SOP-backed capability 的业务样例。

## 维护者流程

- [测试策略](testing.md)：本地验证、CI 检查和 release validation。
- [发布策略](release-policy.md)：版本号、release candidate、tag 和发布流程。
- [分支保护建议](branch-protection.md)：公开前后 `main` 分支保护规则。

## 文档维护规则

- 项目治理文档默认使用中文。
- README、issue template、PR template 等公开协作入口可以使用英文，以降低开源协作门槛。
- 文档必须明确区分当前已实现能力、release candidate 行为和计划能力。
- 当运行时契约、CLI 行为或安全边界变化时，同步更新 README、相关 docs 和 release notes。
