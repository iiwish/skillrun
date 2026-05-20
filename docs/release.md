# Release 流程

SkillRun 使用 `release-plz` 管理 `skillrun` crate 的版本号、`vX.Y.Z` git tag 和 GitHub Release。该流程只发布 GitHub Release，不自动发布到 `crates.io`。

## 自动化边界

- 合并普通 feature PR 到 `main` 后，`.github/workflows/release-plz.yml` 会运行 `release-plz release-pr`，创建或更新 release PR。
- release PR 负责更新 `Cargo.toml`、`Cargo.lock` 和 `CHANGELOG.md` 等 release metadata。
- `release-plz.toml` 设置 `release_always = false`，因此 `release-plz release` 只在 release PR 合并后创建 release。
- `release-plz.toml` 设置 `git_only = true`，版本检测基于 git tag，`cargo publish` 会被跳过。
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
