# 发布策略

SkillRun 使用 SemVer 风格版本号和 `vX.Y.Z` Git tag。项目仍处于 `0.x` 阶段，因此 minor 版本可以包含破坏性调整，但必须在 release notes 中明确说明。

## 版本层级

SkillRun 同时存在几类版本号，不能混用：

- `Cargo.toml` / CLI `skillrun --version`：二进制与 crate 版本，用于发布产物和用户安装识别。
- Git tag `vX.Y.Z`：公开 release 边界，只能从稳定 `main` 创建。
- 文档里出现的 v0.5.2、v0.5.3、v0.5.4：milestone / integration line 名称，用于描述一组设计与交付范围；除非 release decision 明确确认，否则不等于已发布 tag。
- Manifest `manifest_version`：Manifest IR schema 版本。它描述 capsule runtime contract 的结构，不随每个 CLI patch 自动变化。
- IPC / Adapter `protocol_version`：Core 与 adapter process 的文件协议版本。它描述运行时 southbound contract，不等于 crate 版本。

发布前必须显式说明这些层级是否变化。只做文档、测试、Consumer JSON fixture 或治理 hardening 时，不应为了 milestone 名称自动修改 Manifest IR 或 Adapter Protocol 版本。

## 发布分支与 tag

- `main` 是稳定主线。
- 版本开发优先在 `codex/v<major>.<minor>-integration` 分支完成，例如 `codex/v0.3-integration`。
- 发布 tag 由 `release-plz` 在 release PR 合并后从 `main` 创建，例如 `v0.5.16`。
- 只有需要维护旧版本补丁时，才从 tag 创建 `release/v<major>.<minor>` 分支。
- `skillrun` 当前使用 `git_only = true` 的 release-plz 流程，并在 `release-plz.toml` 与 `Cargo.toml` 中显式禁用 crates.io publish。

## Release Candidate 闸门

进入 release candidate 前必须确认：

- README、中文 README、`docs/` 和 `RELEASE_NOTES.md` 已同步。
- `Cargo.toml` 版本号、CLI `--version` 输出和 release notes 一致。
- release notes 明确说明 binary/crate version、Manifest IR version、Adapter Protocol version 是否变化。
- `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 通过。
- MCP、runtime、packaging 和 security boundary 相关行为有测试或明确验证证据。
- release notes 明确列出 included behavior、boundaries、known limits 和 validation。

## 发布步骤

具体发布执行清单见 [Release Checklist](release-checklist.md) 和 [Release 流程](release.md)。本节只定义发布原则和最小步骤。

1. 从版本集成分支合并到 `main`。
2. 等待 `release-plz-pr` job 创建或更新 release PR。
3. Review release PR 中的版本号、`Cargo.toml`、`Cargo.lock`、`CHANGELOG.md` 或 release notes 摘要。
4. 在 release PR 上确认 release validation 与远端 CI 结果。
5. 合并 release PR 到 `main`。
6. 由 `release-plz-release` job 创建 `vX.Y.Z` tag 和 GitHub Release。
7. 确认 GitHub Release 内容可追溯到合并提交/PR，并记录本次是否执行 package registry publication；默认不发布 `crates.io`。

## 回滚与补丁

- 未公开的 tag 可以删除并重建；公开后不要改写 tag 历史。
- 已公开版本发现严重问题时，优先发布补丁版本，例如 `v0.2.1`。
- 安全问题按 [安全政策](../SECURITY.md) 协调披露和修复。
