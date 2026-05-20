# 原生二进制分发

本文定义 `skillrun` CLI 的原生二进制分发入口。主分发物是 GitHub Releases 上的预编译 binary，不要求普通用户安装 Rust toolchain。

## 分发层级

SkillRun 的 CLI 分发按以下层级推进：

1. P0：GitHub Releases 上的原生 binary archives 与 checksum。
2. P1：shell / PowerShell installer，让 macOS、Linux、Windows 用户一条命令安装到 PATH。
3. P2：Homebrew tap，作为 macOS/Linux 的包管理入口。当前预留 tap 路径，不在本任务内发布。
4. P3：npm wrapper，仅作为 JS / agent 生态补充入口。npm package 只负责下载或引用对应平台的原生 binary，不重新实现 CLI。
5. P4：`crates.io` / `cargo install skillrun` 仅服务 Rust 开发者，不作为普通用户主入口。

## Release 工具

SkillRun 使用 `cargo-dist` 生成 release artifacts、checksum 和安装脚本。仓库配置位于：

- `dist-workspace.toml`
- `.github/workflows/release.yml`
- `Cargo.toml` 的 `[profile.dist]`

release workflow 在推送 `vX.Y.Z` 形式 tag 时运行，并将 artifacts 上传到对应 GitHub Release。tag 仍按 `docs/release-checklist.md` 的顺序从已验证的 `main` 创建。

## 平台矩阵

首批平台矩阵固定为：

| Target triple | Archive |
| --- | --- |
| `aarch64-apple-darwin` | `skillrun-aarch64-apple-darwin.tar.xz` |
| `x86_64-apple-darwin` | `skillrun-x86_64-apple-darwin.tar.xz` |
| `x86_64-unknown-linux-gnu` | `skillrun-x86_64-unknown-linux-gnu.tar.xz` |
| `aarch64-unknown-linux-gnu` | `skillrun-aarch64-unknown-linux-gnu.tar.xz` |
| `x86_64-pc-windows-msvc` | `skillrun-x86_64-pc-windows-msvc.zip` |

每个 archive 都包含：

- `skillrun` 或 `skillrun.exe`
- `LICENSE`
- `README.md`

## Checksum 产物

每次 release 生成：

- `sha256.sum`：汇总 checksum。
- `skillrun-<target>.tar.xz.sha256` 或 `skillrun-<target>.zip.sha256`：单个 archive checksum。
- `source.tar.gz.sha256`：source archive checksum。

用户可用 `sha256sum -c sha256.sum` 或平台等价工具校验下载文件。文档和 release notes 不应把 checksum 描述为代码签名或 notarization。

## 安装入口

macOS / Linux：

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/iiwish/skillrun/releases/latest/download/skillrun-installer.sh | sh
skillrun --version
```

Windows PowerShell：

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/iiwish/skillrun/releases/latest/download/skillrun-installer.ps1 | iex"
skillrun --version
```

默认安装路径由 `cargo-dist` 的 `install-path = "CARGO_HOME"` 控制。用户也可以直接从 GitHub Release 下载对应平台 archive，解压后将 `skillrun` 或 `skillrun.exe` 放入 PATH。

安装后的 `skillrun --version` 必须与 release tag 去掉 `v` 后的版本号、`Cargo.toml` 中的 package version 一致。

## Homebrew 预留

Homebrew tap 预留路径：

- Tap repository：`iiwish/homebrew-skillrun`
- Formula：`skillrun.rb`

后续启用 Homebrew 时，应在 `cargo-dist` 配置中增加 `homebrew` installer，并先确认 tap repository、token 权限和 release checklist。启用前不应让当前 GitHub Release workflow 依赖不存在的 tap。

## npm wrapper 边界

npm package 名称可后续评估为 `skillrun` 或 scoped package。无论名称如何，npm wrapper 只允许：

- 根据 `process.platform` / `process.arch` 选择 GitHub Release artifact。
- 下载、缓存或引用原生 `skillrun` binary。
- 透传 CLI argv 和 exit code。

npm wrapper 不允许：

- 用 Node 重新实现 `skillrun` CLI。
- 成为唯一分发入口。
- 改变 Manifest、runtime、adapter protocol 或 Desktop contract。

## 维护者验证

分发相关变更至少运行：

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --version
dist plan
```

本地只能验证当前 host 能构建的 artifact。跨平台 binary 和 installer 以 GitHub Actions release matrix 为准。
