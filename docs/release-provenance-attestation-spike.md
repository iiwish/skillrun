# Release Provenance Attestation Spike

**文档状态**：Design spike
**最后更新**：2026-05-31

## 目标

本文审计 `skillrun` 当前 cargo-dist release workflow 中可接入 GitHub artifact attestations 的位置、最终 artifact path、验证命令和 release checklist 变更范围。

本 spike 不启用 attestation，不修改 `.github/workflows/release.yml`，不改变 release asset list，也不把 provenance 写成 code signing、publisher identity、notarization、package trust 或 sandbox。

## 当前 release workflow 事实

当前 release pipeline 由两个 workflow 配合：

- `.github/workflows/release-plz.yml` 在 `main` 上运行 release-plz，维护 release PR、tag / GitHub Release 一致性 guard。
- `.github/workflows/release.yml` 由 version tag 触发，使用 cargo-dist 生成 native artifacts，并上传到同 tag GitHub Release。

最近一次真实 release 核对：

- Release：`v0.6.5`
- Release workflow run：`26489037383`
- GitHub Release target commit：`7bab5cb0a6f1f2c75914245bc857a5e4ce583dc7`
- 公开 assets：两个 installer、五个平台 archive、一个 `sha256.sum`
- `gh release verify v0.6.5 --repo iiwish/skillrun --format json` 当前返回 no attestations，说明 release-level attestation 尚未启用。

## Artifact path 审计

`release.yml` 的 artifact 流向如下：

| Stage | Job / Step | Path | 说明 |
| --- | --- | --- | --- |
| Plan | `plan` / `dist host --steps=create` | `plan-dist-manifest.json` | 生成 release plan，并上传为 `artifacts-plan-dist-manifest`。 |
| Local build | `build-local-artifacts` / `dist build ...` | cargo-dist 输出的 archive 与 per-archive checksum | 每个 target job 通过 `dist print-upload-files-from-manifest` 取得待上传文件列表，再上传为 `artifacts-build-local-*` workflow artifact。 |
| Local manifest | `build-local-artifacts` / `Post-build` | `target/distrib/<targets>-dist-manifest.json` | 只用于内部 workflow artifact，不应进入 public release assets。 |
| Global build | `build-global-artifacts` / `dist build --artifacts=global` | `sha256.sum`、`skillrun-installer.sh`、`skillrun-installer.ps1` | 生成 unified checksum 与 installer，并上传为 `artifacts-build-global`。 |
| Host fetch | `host` / `Fetch artifacts` | `target/distrib/` | cargo-dist host step 使用 scratch artifacts 继续 upload / release。 |
| Host publish staging | `host` / `Download GitHub Artifacts` | `artifacts/` | 将所有 `artifacts-*` workflow artifacts 合并到单个 staging directory。 |
| Public cleanup | `host` / `Cleanup` | `artifacts/` | 删除内部 manifest、单个 `.sha256` 和 `source.tar.gz*`，保留公开 assets。 |
| Public upload | `host` / `Create GitHub Release` | `artifacts/*` | `gh release upload <tag> artifacts/* --clobber` 上传最终公开 assets。 |

结论：R2 的最小 attestation subject 应绑定 `host` job `Cleanup` 之后、`Create GitHub Release` 之前的 `artifacts/*`。这是当前 release 页面最终展示的同一批 bytes，避免 attesting 内部 manifest、per-archive `.sha256` 或被清理掉的 source tarball。

当前最终公开 asset set 应保持：

```text
sha256.sum
skillrun-aarch64-apple-darwin.tar.xz
skillrun-aarch64-unknown-linux-gnu.tar.xz
skillrun-installer.ps1
skillrun-installer.sh
skillrun-x86_64-apple-darwin.tar.xz
skillrun-x86_64-pc-windows-msvc.zip
skillrun-x86_64-unknown-linux-gnu.tar.xz
```

## GitHub artifact attestations 接入点

GitHub 当前官方路径是使用 `actions/attest@v4` 对 `subject-path` 指向的 artifact 生成 build provenance attestation。官方文档要求 workflow 具备 `id-token: write`、`contents: read`、`attestations: write` 权限；`gh attestation verify` 默认验证 `https://slsa.dev/provenance/v1` predicate。

推荐的 implementation shape：

```yaml
  host:
    permissions:
      contents: write
      id-token: write
      attestations: write
    steps:
      # existing checkout / dist / download / cleanup steps
      - name: Generate release asset attestations
        uses: actions/attest@v4
        with:
          subject-path: artifacts/*
      - name: Create GitHub Release
        run: |
          gh release upload "${{ needs.plan.outputs.tag }}" artifacts/* --clobber
          gh release edit "${{ needs.plan.outputs.tag }}" --target "$RELEASE_COMMIT" $PRERELEASE_FLAG --draft=false
```

接入理由：

- `artifacts/*` 已经是公开 release asset staging set。
- attestation 与 upload 使用同一批本地 bytes。
- attestation step 在 release upload 前失败时，release 仍保持 draft，避免公开一个缺少 provenance 的 release。
- 不把 attestations 作为 GitHub Release assets 上传，减少 release 页面噪音；默认依赖 GitHub attestation store 和 `gh attestation verify` 在线查询。

不推荐的接入点：

- 不在 `build-local-artifacts` 各矩阵 job 中直接 attesting 初始 archive，因为 installer 和 `sha256.sum` 由 `build-global-artifacts` 产生，且最终 public set 还会被 `host` job cleanup 改写。
- 不 attesting `target/distrib/*-dist-manifest.json`，它们不是用户下载入口。
- 不 attesting 单个 archive `.sha256`，它们当前被清理，不在 release asset list。
- 不先依赖 `gh release verify` 作为唯一 gate；当前 `v0.6.5` 没有 release-level attestation，且该命令验证的是 release-level attestation，不等价于逐个 asset 的 artifact provenance policy。

## Verification gate 设计

推荐在 `verify-release-assets` job 增加独立 verification step，放在 asset list 和 checksum manifest 校验之后、installer smoke 之前。

建议下载并逐个验证最终公开 assets：

```bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/release-attestations"

while read -r asset; do
  gh release download "$RELEASE_TAG" \
    --repo "$GITHUB_REPOSITORY" \
    --pattern "$asset" \
    --dir "$RUNNER_TEMP/release-attestations"

  gh attestation verify "$RUNNER_TEMP/release-attestations/$asset" \
    --repo "$GITHUB_REPOSITORY" \
    --signer-workflow "iiwish/skillrun/.github/workflows/release.yml" \
    --source-ref "refs/tags/$RELEASE_TAG" \
    --format json >/tmp/skillrun-attestation.json
done < expected-assets.txt
```

需要在 implementation PR 中用真实 run 校验的细节：

- `--source-ref "refs/tags/$RELEASE_TAG"` 是否匹配 tag-triggered release run 的 certificate/source metadata。
- `--signer-workflow "iiwish/skillrun/.github/workflows/release.yml"` 是否是当前 `gh` 版本接受的精确格式；必要时改用 `--cert-identity` 或 `--cert-identity-regex`。
- 是否需要在 `verify-release-assets` job 显式声明 `attestations: read`；当前 GitHub CLI 可读取 public repo attestation，但 workflow token 最小权限应在实现 PR 中实测。
- `subject-path: artifacts/*` 是否在当前 `actions/attest@v4` 版本下为每个 public asset 生成可由 `gh attestation verify <downloaded-file>` 找到的 subject。

## Release checklist 变更范围

如果用户批准进入 R2 implementation PR，`docs/release-checklist.md` 至少需要增加这些 gate：

1. Phase 1 本地 release validation：记录本机 `gh --version`，确保包含 `gh attestation verify`。
2. Phase 5 tag / draft release：提醒 release workflow 会在公开前生成 artifact attestations；如果 attestation step 失败，不应手动公开 draft release。
3. Phase 6 确认 GitHub Release：除 asset list、checksum、installer / archive smoke 外，增加 artifact attestation verification。
4. Phase 7 发布后核对：下载至少一个 installer、一个 archive 和 `sha256.sum`，用 `gh attestation verify` 验证 repo、signer workflow 和 tag/source ref。
5. 失败处理：attestation 缺失或 identity mismatch 时，优先修 workflow 并重跑 release；不得把 checksum 通过当成 provenance 通过。

Checklist 文案必须保留边界：

- provenance 只说明 artifact 与 GitHub Actions release workflow / source ref / builder 绑定。
- 不称为 code signing、signed package、verified publisher、notarization、sandbox 或 safe execution。
- 如果未来要把验证结果展示到官网或 Desktop，需要单独通过 trust wording review。

## 权限与风险

实现 R2 会改变 release workflow 权限和发布 gate，因此必须先由用户确认。

预计最小权限变化：

- `host` job：`contents: write`、`id-token: write`、`attestations: write`。
- `verify-release-assets` job：至少 `contents: read`，可能需要 `attestations: read`。

风险点：

- `release.yml` 是 cargo-dist generated workflow with local patches；修改后要继续保留 `allow-dirty = ["ci"]` 的真实意图。
- `subject-path: artifacts/*` 如果匹配了意外文件，会扩大 attestation subject；因此必须依赖既有 `expected-assets.txt` guard，并在 attestation 前后都复查 `artifacts/`。
- 如果 `actions/attest@v4` 或 GitHub attestation API 行为变化，release workflow 可能在 tag 后失败；失败时 release 应保持 draft。
- Attestation predicate 中部分 metadata 由 workflow 上下文产生；policy enforcement 应优先依赖 certificate、timestamp、repo、workflow、source ref 等身份字段。

## Open questions before implementation

- 是否只 attesting 8 个 public assets，还是也 attesting GitHub 自动生成的 source archives。建议第一阶段只 attesting SkillRun 上传的 8 个 public assets。
- 是否把 installer 纳入 `sha256.sum`。这是 Phase R1，不应混入 R2；R2 可以先 attesting installer，但 checksum manifest 仍保持当前边界。
- 是否要增加 offline bundle 下载文档。建议暂缓；第一阶段使用 GitHub attestation store 在线验证，避免 release asset 噪音。
- 是否改 release notes / 官网文案。建议 R2 implementation 只更新 release checklist 和 native distribution docs 的维护者验证部分，公开 marketing 文案等下一次真实 release 后再处理。

## Recommendation

推荐下一步不是直接合并 workflow 变更，而是向用户确认是否进入 R2 implementation PR。若确认，最小 PR 应只做：

1. 在 `host` job 为 cleaned `artifacts/*` 生成 artifact attestations。
2. 在 `verify-release-assets` job 下载最终 release assets 并逐个 `gh attestation verify`。
3. 更新 `docs/release-checklist.md` 和 `docs/native-distribution.md` 的维护者验证说明。
4. 用 release workflow PR run 验证 plan/build jobs 不受影响，再在下一次真实 tag release 中验证 publishing path。

该 PR 不应改版本号、tag、release-plz 策略、asset list、`sha256.sum` 范围、crates.io publishing、code signing、notarization、publisher identity、Homebrew、npm wrapper、Desktop 或官网 trust copy。

## 参考资料

- GitHub Docs: [Using artifact attestations to establish provenance for builds](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)
- GitHub CLI Manual: [gh attestation verify](https://cli.github.com/manual/gh_attestation_verify)
- GitHub CLI Manual: [gh release verify](https://cli.github.com/manual/gh_release_verify)
