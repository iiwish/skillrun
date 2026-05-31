# Provenance and Trust Design

**文档状态**：Design baseline
**最后更新**：2026-05-28

## 目标

本文定义 SkillRun 在 checksum、GitHub Release asset digest、signature、provenance、publisher identity、notarization 和 sandbox 文案之间的边界。

核心目标不是立刻启用签名系统，而是先把将来要做什么、现在不能承诺什么写清楚，避免 release notes、官网、Desktop 或 Team Library 把完整性校验误写成信任证明。

## 当前事实

当前 `skillrun` 公开 release 提供：

- GitHub Release 上的 installer 和平台 archive。
- GitHub Release asset digest。
- `sha256.sum`，当前只覆盖平台 archive。
- cargo-dist release archive smoke 和 installer smoke。
- `.skr` package 内的 Manifest、source hashes、Consumer Mode static checks、run evidence。
- Team Catalog 中 `.skr` source 的 `sha256` 完整性校验。

当前没有：

- signed release assets。
- signed `.skr` package。
- maintainer-provided build provenance attestation。
- publisher identity verification。
- Apple notarization / Windows code signing。
- package registry trust policy。
- OS sandbox、malicious-code detection 或 network egress isolation。

## 词汇边界

| Term | SkillRun 中允许表达的含义 | 不能表达成 |
| --- | --- | --- |
| checksum / `sha256` | 下载或复制后的 bytes 与预期 hash 一致 | 作者身份、可信发布者、无恶意代码、安全执行 |
| GitHub asset digest | GitHub 对 release asset 记录的 digest，可辅助核对 asset 完整性 | 独立于 GitHub 的签名、notarization、publisher identity |
| signature | 某个 identity 或 key 对 artifact bytes 的签名 | sandbox、漏洞扫描、依赖安全、runtime policy |
| provenance / attestation | artifact 如何由哪个 workflow / builder 从哪些输入产生的声明 | 代码正确性、安全性或业务可信 |
| publisher identity | 发布主体或团队身份的可验证声明 | 对 capsule 内容的安全背书，或用户无需审查 |
| notarization / code signing | 平台生态的分发信任和拦截提示改善 | 跨平台安全保证或恶意代码检测 |
| trust note | 人类可读的审核提示和来源说明 | 机器可执行的 allow/deny policy |
| sandbox | 系统级强隔离能力 | 当前 SkillRun Manifest、permissions 或 checksum |

## 分层模型

SkillRun 的 trust story 应按层推进，而不是用一个“trusted”状态覆盖所有风险。

```text
L0 Integrity
  bytes match expected hashes

L1 Provenance
  artifact is linked to a release workflow, source repo, commit, and builder

L2 Publisher Identity
  artifact is linked to an expected maintainer, org, workflow identity, or team catalog owner

L3 Policy Verification
  local policy can decide whether a provenance / publisher / version / source is allowed

L4 Runtime Isolation
  action execution is isolated by OS/container/sandbox primitives
```

当前 SkillRun 已具备一部分 L0，并在 runtime contract 层提供 Manifest / Consumer Mode / run evidence。它尚未实现 L1-L4。

重要原则：

- L0 不是 L1。
- L1 不是 L2。
- L2 不是 L3。
- L3 不是 L4。
- 即使未来实现 L1-L3，运行第三方 action 仍然意味着执行第三方代码，除非另行实现并验证 L4。

## Release Asset 路线

### Phase R0：当前边界

维持现状：

- installer 是普通用户主入口。
- platform archive 是手动安装、离线安装、包管理器维护和 CI 固定版本入口。
- `sha256.sum` 只表达 archive 完整性。
- release docs 不把 checksum 称为 signed trust、publisher identity、notarization 或 sandbox。

### Phase R1：补齐 installer hash 入口

可以考虑让公开 checksum manifest 覆盖 installer 和 archive，或者新增一个清晰命名的 digest manifest。

约束：

- 变更 release asset list、checksum 内容或 installer 验证文案前必须走 release gate。
- 如果 `sha256.sum` 从只覆盖 archive 改为覆盖 installer + archive，必须同步 `.github/workflows/release.yml` 的 verify-release-assets / checksum guard。
- 文档必须仍然说明 checksum 只证明 bytes integrity。

### Phase R2：build provenance attestation

候选路线：

- 使用 GitHub Actions artifact attestations 为 release artifacts 生成 provenance。
- 将 attestation 绑定到具体 artifact，而不是只绑定 release tag。
- 在 release checklist 中增加验证步骤，例如用 GitHub CLI 验证 artifact attestation。
- R2 设计 spike 详见 [Release Provenance Attestation Spike](release-provenance-attestation-spike.md)。

约束：

- 不把 provenance 称为 code signing。
- 不把 provenance 称为 package trust。
- 不把 GitHub Actions workflow identity 误写成人的审核身份。
- 不把 “来自官方 workflow” 写成 “安全执行任意第三方 capsule”。

### Phase R3：blob signing / sidecar bundles

候选路线：

- 用 Sigstore / cosign 对 release archive、installer 或 `.skr` blob 生成 sidecar bundle。
- bundle 文件名应与 artifact 一一对应，例如 `<artifact>.sigstore.json` 或类似命名。
- 验证说明必须包含 expected identity、issuer、repository、workflow 和 tag / digest 约束。

约束：

- signing bundle 增加 release asset 数量，会重新带来下载噪音；若启用，需要配套 docs 和 GitHub Release asset 分组说明。
- keyless signing 更适合 CI identity，但必须定义 expected identity；否则用户只知道“某个 OIDC identity 签过”，不知道是否符合 SkillRun policy。
- self-managed key / KMS / hardware token 是另一条路线，涉及 key custody、rotation、revocation 和 maintainer process，必须单独决策。

### Phase R4：platform signing and package managers

候选路线：

- Apple notarization / macOS code signing。
- Windows Authenticode signing。
- Homebrew tap / npm wrapper / future package manager distribution。

约束：

- 平台签名主要改善平台安装体验和来源提示，不替代 SkillRun runtime trust model。
- 包管理器发布需要独立确认 token、maintainer、formula/package ownership、provenance 和 rollback policy。
- `crates.io` 当前仍是 disabled boundary；不得因为 release signing 设计而默认启用 `cargo publish`。

## `.skr` 和 Team Catalog 路线

`.skr` 是 source + Manifest archive，不是 secure install format。Team Catalog 当前提供发现、展示和 guarded local install path，不提供 publisher identity 或 trust decision。

### Phase C0：当前边界

当前可说：

- Team Catalog item 的 `sha256` 只证明 `.skr` bytes 与 catalog 中记录的 hash 一致。
- Core 在 install apply 前校验 `sha256`。
- import 进入 local registry；registry 是 inventory，不是 trust store。
- switchboard enabled 是 exposure intent，不是 trust proof。

当前不可说：

- catalog publisher 已验证。
- `.skr` 已签名。
- `.skr` 可安全执行任意第三方代码。
- Desktop 做了 trust 判断。

### Phase C1：catalog metadata hygiene

可以先强化 display-only metadata，不引入验证：

- `publisher.name`
- `publisher.url`
- `repository`
- `homepage`
- `license`
- `trust_note`
- `release_notes`

约束：

- 所有字段都必须被描述为 metadata / review hints。
- Desktop 可以展示这些字段，但不得显示为 verified badge。
- Core 不得基于这些字段自动 allow execution。

### Phase C2：signed `.skr` / signed catalog

候选路线：

- 对 `.skr` blob 签名并随 catalog 记录 sidecar bundle URL / digest。
- 对 catalog 文件本身签名，证明 catalog 未被中间人替换。
- 同时签 `.skr` 和 catalog，分别解决 package bytes 和 catalog distribution integrity。

约束：

- signed catalog 不等于 signed `.skr`。
- signed `.skr` 不等于 trusted publisher policy。
- signed `.skr` 不等于 sandbox。
- 如果 Core 增加 verification，JSON surface 必须区分 `signature_present`、`signature_verified`、`identity_matched`、`policy_allowed` 和 `warnings`，不能压成一个 `trusted: true`。

### Phase C3：local trust policy

候选路线：

- 本地 policy 文件记录 allowed catalog id、repository、workflow identity、publisher identity 或 signing certificate identity。
- Team Library / Desktop 只展示 Core 计算出的 policy result。
- Core 提供 `team catalog verify` 或在 `install plan` 中增加兼容字段。

约束：

- 这会影响 CLI JSON contract 和 Desktop/Core 边界，必须单独设计和 review。
- 默认 policy 应 fail closed 或 warning-first，需要用户确认。
- 不应把 policy 结果与 runtime sandbox 混淆。

## 文案规则

允许：

- “checksum verifies integrity”
- “provenance links this artifact to a build workflow”
- “signature verifies this artifact was signed by the expected identity”
- “policy allowed by local trust configuration”
- “review before enabling”
- “not an OS sandbox”

禁止：

- “safe to run arbitrary third-party code”
- “trusted package” without explaining which policy established trust
- “verified publisher” unless identity verification is implemented
- “signed means safe”
- “checksum proves publisher”
- “Team Catalog marketplace”
- “sandboxed” for current Manifest / permissions / checksum behavior

## 推荐下一步

推荐先做 Phase R2 的设计 spike，而不是直接实现：

1. 在 release workflow 的 cargo-dist artifact upload 前后确认可 attested 的最终 artifact path。
2. 设计 attestation asset / GitHub hosted attestation 的验证命令。
3. 决定是否把 attestation 放在 GitHub Release asset list，或只依赖 GitHub attestation store。
4. 更新 release checklist 的 verification steps。
5. 再决定是否进入 implementation PR。

真正实现前必须先问用户，因为这会改变 release workflow、permissions、asset verification 和公开 trust wording。

## 参考资料

实现前应重新核对官方文档和当前工具版本。本设计参考了：

- GitHub Docs: [Using artifact attestations to establish provenance for builds](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)
- Sigstore Docs: [Cosign signing overview](https://docs.sigstore.dev/cosign/signing/overview/)
- Sigstore Docs: [Signing blobs](https://github.com/sigstore/docs/blob/main/content/en/cosign/signing/signing_with_blobs.md)
- SLSA: [Distributing provenance](https://slsa.dev/spec/v1.0/distributing-provenance)
