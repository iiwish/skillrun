# Release 流程

SkillRun 使用 `release-plz` 管理 `skillrun` crate 的版本号、lockfile 和 changelog release PR。公开发布仍是 GitHub-only：不发布到 `crates.io`，由维护者在 review gate 后创建 `vX.Y.Z` git tag 和 draft GitHub Release，再交给 cargo-dist workflow 上传产物并公开 release。

## 自动化边界

- 合并普通 feature PR 到 `main` 后，`.github/workflows/release-plz.yml` 会运行 `release-plz release-pr`，创建或更新 release PR。
- release PR 负责更新 `Cargo.toml`、`Cargo.lock` 和 `CHANGELOG.md` 等 release metadata。
- `release-plz.toml` 设置 `git_only = true`，版本检测基于 git tag。
- `release-plz.toml` 和 `Cargo.toml` 均显式禁用 crates.io publish。因为 `Cargo.toml` 使用 `publish = false`，`release-plz release` 不应被视为创建 GitHub-only tag / release 的唯一机制。
- `.github/workflows/release-plz.yml` 包含 release consistency guard：如果 `Cargo.toml` 的版本号没有对应 `vX.Y.Z` tag 和 GitHub Release，workflow 必须失败，避免出现“版本已 bump 但没有公开 release”的假绿状态。创建 tag 和 draft GitHub Release 后，可以 rerun 该 workflow 确认 guard 变绿。
- release tag 使用 `v{{ version }}` 格式，例如 `v0.5.16`。

## 人工 review gate

维护者必须 review release PR 的版本号、变更摘要和 CI 结果。release PR 合并到 `main` 后，维护者需要确认 `Cargo.toml` 版本、README 版本文案、changelog 和远端 CI 都一致，再创建对应的 `vX.Y.Z` tag 与 draft GitHub Release。

最小 GitHub-only 发布顺序：

1. 合并已 review 的 release metadata PR 到 `main`。
2. 等待 `main` Rust CI 通过。
3. 从该 `main` 提交创建并推送 `vX.Y.Z` tag。
4. 创建同 tag 的 draft GitHub Release。
5. 等待 tag push 触发的 Release workflow 上传 cargo-dist assets 并将 release 公开。
6. 确认 asset list、checksum、installer smoke 和 archive smoke 通过。

## GitHub 设置要求

仓库的 GitHub Actions workflow permissions 需要允许 `GITHUB_TOKEN` 创建和更新 Pull Request。否则 `release-plz-pr` job 无法创建 release PR。

## 发布前验证

release PR 合并前至少确认：

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## 发布后验证

推送 `vX.Y.Z` tag 且存在同 tag draft GitHub Release 后，Release workflow 会在上传 assets 后执行 post-release guard：

- GitHub Release 必须是当前 tag、非 draft，并且 GitHub latest release 必须指向当前 tag。
- Release assets 必须保持精简，只包含两个 installer、五个平台 archive 和一个 `sha256.sum`。
- `sha256.sum` 只能引用实际展示的五个平台 archive，不能引用内部 manifest、单独 `.sha256` 文件或重复 source archive。
- Linux runner 会执行 `skillrun-installer.sh`，并验证安装出的 `skillrun --version` 与 tag 一致。
- Linux、macOS、Windows runner 会分别下载对应 archive，执行 `skillrun --version` 和 `skillrun host status --json`。

这些检查失败时，release run 必须失败。维护者不应手动把失败的 release 标成完成。
