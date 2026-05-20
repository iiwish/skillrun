# Release 流程

SkillRun 使用 `release-plz` 管理 `skillrun` crate 的版本号、`vX.Y.Z` git tag 和 GitHub Release。该流程只发布 GitHub Release，不自动发布到 `crates.io`。

## 自动化边界

- 合并普通 feature PR 到 `main` 后，`.github/workflows/release-plz.yml` 会运行 `release-plz release-pr`，创建或更新 release PR。
- release PR 负责更新 `Cargo.toml`、`Cargo.lock` 和 `CHANGELOG.md` 等 release metadata。
- `release-plz.toml` 设置 `release_always = false`，因此 `release-plz release` 只在 release PR 合并后创建 release。
- `release-plz.toml` 设置 `git_only = true`，版本检测基于 git tag。
- `release-plz.toml` 和 `Cargo.toml` 均显式禁用 crates.io publish，`release-plz release` 只负责 git tag 与 GitHub Release。
- release tag 使用 `v{{ version }}` 格式，例如 `v0.5.16`。

## 人工 review gate

维护者必须 review release PR 的版本号、变更摘要和 CI 结果。只有 release PR 合并到 `main` 后，自动化才会创建对应的 `vX.Y.Z` tag 与 GitHub Release。

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

推送 `vX.Y.Z` tag 后，Release workflow 会在上传 assets 后执行 post-release guard：

- GitHub Release 必须是当前 tag、非 draft，并且 GitHub latest release 必须指向当前 tag。
- Release assets 必须保持精简，只包含两个 installer、五个平台 archive 和一个 `sha256.sum`。
- `sha256.sum` 只能引用实际展示的五个平台 archive，不能引用内部 manifest、单独 `.sha256` 文件或重复 source archive。
- Linux runner 会执行 `skillrun-installer.sh`，并验证安装出的 `skillrun --version` 与 tag 一致。
- Linux、macOS、Windows runner 会分别下载对应 archive，执行 `skillrun --version` 和 `skillrun host status --json`。

这些检查失败时，release run 必须失败。维护者不应手动把失败的 release 标成完成。
