# SkillRun v0.4 Implementation Plan

Version: v0.4
Status: Ready_For_User_Review
Source spec: `.ai-platform/specs/v0.4/spec.md`
Last updated: 2026-05-13
Review: Drafted after assistant review; waiting for user confirmation before tasks may move to Ready.

## 一句话判断

v0.4 的实现顺序应先把 dependency contract 写进 Manifest 语义，再建立静态 readiness engine，最后把 runtime/MCP 依赖失败统一收敛到 `DependencyError`。

## Decision Summary

### D001: `check` Is The Automation-grade Readiness Command

`check` 是 v0.4 的核心命令，面向 CI、Agent 和严肃 Consumer Mode。

Implication:
- `check` 输出必须稳定、可测试。
- `doctor` 可保持人类友好，不作为唯一自动化接口。
- `inspect` 不承担 runtime dependency 判断。

### D002: Readiness Comes From Manifest Plus Adapter Defaults

v0.4 不为了 check 导入 action source。依赖信息来自 Manifest `runtime` 字段、source hashes、examples 和 adapter defaults。

Implication:
- Freshness check 先于 dependency check。
- Legacy Manifest 缺少 requirements 时可使用 adapter defaults，但需要清楚提示。
- requirements 是诊断合同，不是安装合同。

### D003: Dependency Failures Are First-class Errors

缺 executable、版本不满足、缺 adapter package 都归入 `DependencyError`。

Implication:
- `errors.rs` 必须允许 `DependencyError`。
- `runtime` 需要在创建 run record 后返回结构化 envelope，或在无法创建 run 前给出 CLI-level 诊断；优先保持 run evidence。
- MCP tool call 必须把该错误映射成 tool-level error result。

### D004: No Installer In v0.4

SkillRun 不调用 pip、uv、npm 或 nvm。v0.4 只诊断和解释。

Implication:
- Docs 和 CLI 文案可以给修复建议，但不自动执行。
- `.skr` 仍不包含依赖。

### D005: Hostile Environment Tests Are Release-critical

v0.4 的质量不由 happy path 证明，而由缺依赖场景证明。

Implication:
- 测试需要模拟缺 Python、缺 Node、缺 Pydantic、版本不满足。
- MCP server survival 是 release gate。

## Architecture Plan

### Current State

- `doctor` 已能检查文件、Manifest freshness 和 source hash，且不 import action source。
- Python/Node adapter runtime 仍会把底层 spawn/import failure 表现为泛化 runtime failure 或 adapter-specific error text。
- `errors.rs` 尚未把 `DependencyError` 列入合法 error envelope。
- Manifest 尚无明确 `runtime.requirements` contract。

### Target Shape

```text
src/
  check.rs or readiness.rs    static capsule readiness engine
  doctor.rs                   human-friendly rendering over readiness
  errors.rs                   DependencyError envelope support
  manifest.rs                 runtime requirements generation
  adapters/
    python.rs                 Python discovery/package probe
    node.rs                   Node discovery probe
```

Readiness engine responsibilities:

- Load and validate capsule shape.
- Load Manifest without executing source.
- Validate source freshness.
- Resolve adapter requirements.
- Probe host executable and required packages.
- Produce structured readiness findings.

Runtime responsibilities:

- Before adapter execution, probe minimal runtime readiness.
- Convert dependency failure into `DependencyError`.
- Preserve stdout/stderr log discipline.

MCP responsibilities:

- Convert runtime `DependencyError` envelope into MCP `isError: true`.
- Continue serving after dependency failure.

## Delivery Slices

### Slice 1: Contract And Characterization

Goal:
Lock v0.4 requirements and current failure behavior before changing runtime.

Includes:
- Tests for current missing dependency behavior.
- Manifest requirements shape tests.
- Error envelope code tests.

### Slice 2: Readiness Engine And `check`

Goal:
Create static Consumer Mode readiness diagnostics.

Includes:
- `check` CLI parser/help.
- Readiness model.
- Source freshness and example checks.
- Python/Node executable probes.

### Slice 3: Adapter Dependency Probes

Goal:
Check adapter-specific packages without importing action source.

Includes:
- Python Pydantic v2 probe.
- Version requirement comparison.
- Clear remediation hints.

### Slice 4: Runtime And MCP DependencyError

Goal:
Make dependency failures structured and Agent-safe.

Includes:
- Runtime pre-adapter readiness check.
- `DependencyError` envelope.
- MCP tool-call survival tests.

### Slice 5: Portable `.skr` And Docs

Goal:
Show distributed capsules are diagnosable without becoming runtime images.

Includes:
- `.skr` unpack check tests.
- README/SSOT/testing/release docs.
- Release matrix.

## Test Strategy

- Contract tests: CLI help, `check` output, Manifest requirements fields.
- Negative tests: no Python, no Node, missing Pydantic, unsupported versions.
- Consumer guard tests: no action import during `check` and `doctor`.
- Runtime tests: CLI `DependencyError` envelope.
- MCP tests: `tools/call` dependency failure and server survival.
- Pack tests: unpacked `.skr` inspect/check.

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| `check` duplicates `doctor` confusingly | User mental model weakens | Define `check` as automation-grade readiness; `doctor` as human-friendly diagnostic |
| dependency probes accidentally import action source | Consumer Mode trust boundary breaks | Dedicated no-import marker tests for Python and JS |
| version parsing becomes package manager scope | Scope creep | Support only minimal semver-like comparisons needed by adapter defaults |
| missing dependency errors bypass run evidence | Audit trail weakens | Prefer creating run record before returning runtime `DependencyError` when possible |
| MCP server exits on dependency failure | Agent runtime unreliable | Scripted MCP client regression test after forced dependency failure |
| `.skr` is perceived as runtime image | Security/distribution overclaim | Docs and pack output keep dependency-free boundary explicit |

## Out Of Scope For This Plan

- HTTP transport.
- package installation.
- dependency vendoring.
- registry/signed package.
- sandbox.
- full TypeScript support.
- package-manager lockfile interpretation.

## User Review Gate

- Approval: Pending.
- Reviewer notes: Plan is ready for review together with `tasks.md`; no task should move to Ready until user confirms scope and task breakdown.
