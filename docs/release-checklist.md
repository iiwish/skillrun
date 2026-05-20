# SkillRun Release Checklist

## 一句话判断

SkillRun release 必须把代码事实、文档事实、远端 CI 事实、Git tag 事实和 GitHub Release 事实对齐；任何一步缺失，都不应把版本描述为 `Released`。

这份 checklist 来自 v0.5.5 的实际发布复盘。它不是替代 `docs/release-policy.md`，而是把发布当天必须执行的动作固定成可检查流程。

## 适用范围

适用于公开发布版本，例如 `v0.5.5`、`v0.5.6`、`v0.6.0`。

不适用于仅在分支内推进的 milestone 文档、实验分支或未决定发布的 integration line。

## 角色边界

- `main` 是稳定主线，只接收已经 review、验证并准备进入公开历史的变更。
- release branch 必须先推远端并通过 CI。
- tag 只能从通过验证的 `main` 创建。
- GitHub Release 创建后，必须回写 release notes 状态。
- package registry publication 是单独决策；不能因为 GitHub Release 已发布就默认执行。

## Phase 0: 发布前确认

确认当前变更已经完成 review：

```powershell
git status --short --branch
git log --oneline main..HEAD
```

必须满足：

- 工作区没有无关修改。
- release branch 名称清晰，例如 `codex/v0.5.6-release-polish`。
- commit message 遵守 Conventional Commits。
- `README.md`、`README.zh-CN.md`、`docs/` 和 `RELEASE_NOTES.md` 对当前版本叙事一致。
- release notes 明确 included behavior、boundaries、known limits 和 validation。
- release notes 明确 binary/crate version、Manifest IR version、IPC / Adapter version 是否变化。

## Phase 1: 本地 release validation

在 release branch 上运行：

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
git diff --check
cargo run --quiet -- --version
dist plan
```

必须满足：

- 所有命令退出码为 0。
- `skillrun --version` 输出与 `Cargo.toml` 和 `RELEASE_NOTES.md` 一致。
- `dist plan` 列出当前版本 tag 对应的 GitHub Release archives、checksum、`skillrun-installer.sh` 和 `skillrun-installer.ps1`。
- 如果只做文档变更，可以不强制跑全量 Rust 测试，但 release/tag 前必须在 `main` 上跑完整 validation。

## Phase 2: 推送 release branch 并等待 CI

推送分支：

```powershell
git push -u origin <release-branch>
```

确认远端 CI：

- `fmt` 通过。
- `clippy` 通过。
- `test` 通过。

如果 `gh` 不可用，可以用 GitHub Checks API 或 GitHub web 页面确认。

发布纪律：

- release branch CI 未通过，不得合并到 `main`。
- 远端 Linux CI 失败不能用本地 Windows 测试结果覆盖。
- 如果 CI 失败日志不可见，先增强 CI 可观测性，再定位失败原因。

## Phase 3: 合并到 main

更新并切换 `main`：

```powershell
git fetch origin
git checkout main
git pull --ff-only origin main
```

使用显式 merge commit 合入：

```powershell
git merge --no-ff <release-branch> -m "merge: <release summary>"
```

合并后必须先在本地 `main` 跑 release validation：

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
git diff --check
cargo run --quiet -- --version
```

通过后推送 `main`：

```powershell
git push origin main
```

## Phase 4: 等待 main CI

必须等待刚推送的 `main` commit 对应远端 CI 全部通过。

必须满足：

- `fmt` 成功。
- `clippy` 成功。
- `test` 成功。
- check run 的 `head_sha` 等于当前 `main` HEAD。

禁止事项：

- 不得在 main CI 还在跑时创建 tag。
- 不得用 release branch CI 代替 main CI。
- 不得 tag 一个还没有推送到远端 main 的本地 commit。

## Phase 5: 创建并推送 tag

先确认 tag 不存在：

```powershell
git tag --list vX.Y.Z
git ls-remote --tags origin vX.Y.Z
```

创建 annotated tag：

```powershell
git tag -a vX.Y.Z -m "SkillRun vX.Y.Z"
git push origin vX.Y.Z
```

必须满足：

- tag 指向当前 `main` HEAD。
- tag 对应 commit 已通过 main CI。
- tag 名称使用 `vX.Y.Z`。

## Phase 6: 创建 GitHub Release

GitHub Release 内容应来自 `RELEASE_NOTES.md` 对应版本段落。

必须包含：

- headline。
- version layers。
- included behavior。
- boundaries。
- validation。
- package publication 是否执行。
- `cargo-dist` 生成的 platform archives、checksum、`sha256.sum`、shell installer 和 PowerShell installer。

如果使用 GitHub API 创建 release，必须确认：

- `tag_name` 是 `vX.Y.Z`。
- `target_commitish` 是 `main` 或 tag 已存在时指向正确 commit。
- `draft=false`。
- `prerelease` 按 release 决策设置；普通稳定 patch 使用 `false`。

## Phase 7: 发布后回写

GitHub Release 创建后，必须回写 `RELEASE_NOTES.md`：

- `Status: Released`
- `Publication:` 明确 main merge、remote push、tag、GitHub Release 是否完成。
- 明确 package registry publication 是否执行。
- 将 validation 中的远端 CI 条目改为已完成事实。

回写后提交并推送 `main`：

```powershell
git add RELEASE_NOTES.md
git commit -m "docs(release): mark vX.Y.Z as released"
git push origin main
```

再次等待这个 docs commit 的 main CI 通过。

如果 tag 已经创建在回写前的 commit 上，这是可接受的；但 release notes 回写必须进入 `main` 公开历史。

## Phase 8: 最终核对

发布完成前最后确认：

```powershell
git status --short --branch
git tag --points-at HEAD
git log --oneline -5
```

必须确认：

- 工作区干净。
- `main` 与 `origin/main` 同步。
- GitHub Release URL 可访问。
- `RELEASE_NOTES.md` 已反映真实发布状态。
- 没有误称 package publication、signed package、OS sandbox 或 runtime image。

## 失败处理

### CI 失败

- 不合并。
- 不 tag。
- 先定位失败原因。
- 如果失败信息不可见，先修 CI 可观测性。
- 修复后重新推 release branch 并等待 CI。

### tag 已推送但 release 未创建

- 不重写 tag。
- 创建 GitHub Release。
- 回写 release notes。

### GitHub Release 已发布但发现严重问题

- 不改写公开 tag。
- 优先发布补丁版本，例如 `v0.5.6`。
- 如涉及安全问题，按 `SECURITY.md` 协调披露和修复。

## v0.5.5 经验固化

v0.5.5 暴露并修复了两个发布工程问题：

- 本地 Windows validation 不能代表 Ubuntu CI；release branch CI 和 main CI 都必须通过。
- 未认证 GitHub API 可能无法下载完整 job log；CI 需要在失败时提供可读 annotation。

因此后续 release 不应跳过远端 CI，也不应把 CI 可观测性当成次要体验问题。
