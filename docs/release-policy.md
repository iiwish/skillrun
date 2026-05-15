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
- 发布时从干净 `main` 打 tag，例如 `v0.2.0`。
- 只有需要维护旧版本补丁时，才从 tag 创建 `release/v<major>.<minor>` 分支。

## Release Candidate 闸门

进入 release candidate 前必须确认：

- README、中文 README、`docs/` 和 `RELEASE_NOTES.md` 已同步。
- `Cargo.toml` 版本号、CLI `--version` 输出和 release notes 一致。
- release notes 明确说明 binary/crate version、Manifest IR version、Adapter Protocol version 是否变化。
- `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 通过。
- MCP、runtime、packaging 和 security boundary 相关行为有测试或明确验证证据。
- release notes 明确列出 included behavior、boundaries、known limits 和 validation。

## 发布步骤

1. 从版本集成分支合并到 `main`。
2. 在 `main` 上运行 release validation。
3. 更新 `RELEASE_NOTES.md`。
4. 创建 signed 或 annotated tag：

```bash
git tag -a v0.2.0 -m "SkillRun v0.2.0"
```

5. 推送 `main` 和 tag。
6. 在 GitHub Release 中粘贴 release notes，标注 pre-1.0 边界。

## 回滚与补丁

- 未公开的 tag 可以删除并重建；公开后不要改写 tag 历史。
- 已公开版本发现严重问题时，优先发布补丁版本，例如 `v0.2.1`。
- 安全问题按 [安全政策](../SECURITY.md) 协调披露和修复。
