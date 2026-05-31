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
- [Agent Skills Compatibility](agent-skills-compatibility.md)：说明 SkillRun 与 Agent Skills、MCP 的关系，以及可执行技能的 runtime / contract / evidence 边界。
- [Agent Skills Metadata Mapping](agent-skills-metadata-mapping.md)：设计 `SKILL.md` frontmatter 中 `name`、`description`、`license`、`compatibility`、`metadata`、`allowed-tools` 到 Manifest 的兼容映射。
- [Team Distribution Route](team-distribution.md)：记录目标不变、路线调整的产品基线：用团队分发 MCP tools 和 Agent Skills 作为外部入口，但保持 SkillRun 的 runtime / contract / evidence 边界。
- [Team Catalog Contract Draft](team-catalog-contract.md)：为 Team Library 设计保守的 catalog / install plan 草案，第一阶段只让 `.skr` item 进入 Core install/update 路径。
- [项目定位](positioning.md)：SkillRun 的最强公开定位、生态边界和 v0.4.2 叙事。
- [愿景](vision.md)：长期愿景、信任演进阶段和官方胶囊策略。
- [信任模型](trust-model.md)：当前可信边界、非承诺和 `.skr` 消费边界。
- [Provenance and Trust Design](provenance-trust-design.md)：checksum、GitHub asset digest、signature、provenance、publisher identity、notarization 与 sandbox 文案边界。
- [Release Provenance Attestation Spike](release-provenance-attestation-spike.md)：审计 cargo-dist release workflow 的 artifact path、GitHub attestation 接入点和 release checklist 变更范围。
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
- [v0.5.13 Import Router Contract](v0.5.13-import-router-contract.md)：计划硬化 import、switchboard、exposure 与 Router 的端到端消费合同。
- [v0.5.14 Desktop Host Readiness](v0.5.14-desktop-host-readiness.md)：为 tray-first Desktop 增加 `host status --json` 握手合同，不引入 daemon 或 UI。
- [v0.5.15 Desktop Contract Freeze](v0.5.15-desktop-contract-freeze.md)：冻结 `desktop.alpha` contract set，并为 `import --json` 增加结构化失败合同。
- [v0.6 Consumer Era 愿景](v0.6-consumer-era-vision.md)：本地消费者控制面、SkillRun Router、一键 MCP 挂载、Tauri/Desktop 边界和官方领域包策略。
- [v0.6 Skill Capsule Contract](v0.6-skill-capsule-contract.md)：冻结 Skill Capsule 作为 Agent Skill 基础单元的 Manifest、Adapter、Consumer JSON、Package、Run Evidence 和 fail-closed 合同。
- [Run Evidence and Local Index](run-evidence-index.md)：说明 `consumer runs list/inspect/index` 的稳定 surface、summary-only 隐私边界和 index staleness 诊断。
- [Team Catalog Draft JSON Schema](contracts/team-catalog.draft.schema.json)：`team.catalog.v1` catalog 文件的草案 schema；用于已实现 inspect/plan 和后续 apply 的输入校验基线。
- [Team Catalog Inspect JSON Schema](contracts/team-catalog-inspect.schema.json)：`team catalog inspect --json` 的 `team.catalog.inspect.v1` 机器可读合同。
- [Team Catalog Status JSON Schema](contracts/team-catalog-status.schema.json)：`team catalog status --json` 的 `team.catalog.status.v1` 机器可读合同。
- [Team Catalog Install Plan JSON Schema](contracts/team-catalog-install-plan.schema.json)：`team catalog install plan --json` 的 `team.catalog.install_plan.v1` 机器可读合同。
- [Team Catalog Install Apply JSON Schema](contracts/team-catalog-install-apply.schema.json)：`team catalog install apply --json` 的 `team.catalog.install_apply.v1` 机器可读合同。
- [Router MCP JSON Schema](contracts/router-mcp.schema.json)：`router serve --mcp --dry-run` 的 `router.mcp.v1` 机器可读合同。
- [Router Status JSON Schema](contracts/router-status.schema.json)：`router status --json` 的 `router.status.v1` 路由快照合同。
- [Public Site Data](public-site-data.json)：供 `skillrun-www` 同步的版本、安装、quickstart、trust boundary 与 team distribution 公开数据；schema 见 [Public Site Data JSON Schema](public-site-data.schema.json)。
- [经典业务示例](business-examples.md)：SOP-backed capability 的业务样例。

## 维护者流程

- [测试策略](testing.md)：本地验证、CI 检查和 release validation。
- [Release 流程](release.md)：release-plz 自动版本号、release PR、`vX.Y.Z` tag 和 GitHub Release 流程。
- [发布策略](release-policy.md)：版本号、release candidate、tag 和发布流程。
- [Release Checklist](release-checklist.md)：基于 v0.5.5 发布复盘固化的逐步发布检查清单。
- [原生二进制分发](native-distribution.md)：GitHub Releases artifacts、checksum、installer、Homebrew tap 与 npm wrapper 边界。
- [分支保护建议](branch-protection.md)：公开前后 `main` 分支保护规则。

## 文档维护规则

- 项目治理文档默认使用中文。
- README、issue template、PR template 等公开协作入口可以使用英文，以降低开源协作门槛。
- 文档必须明确区分当前已实现能力、release candidate 行为和计划能力。
- 当运行时契约、CLI 行为或安全边界变化时，同步更新 README、相关 docs 和 release notes。
- 当 release、installer、公开 quickstart 或 trust boundary 文案变化时，同步更新 `public-site-data.json`，让官网不要继续手写漂移。
