# SkillRun v0.4 Feature Spec

Version: v0.4
Status: Ready_For_User_Review
Created: 2026-05-13
Updated: 2026-05-13
Source: `docs/v0.4-portable-consumer-checks.md`
Review: Drafted after assistant review; waiting for user confirmation before implementation planning is treated as approved.

## 一句话判断

v0.4 要验证 SkillRun 的第二个核心假设：

> Skill Capsule 可以被分发、检查和解释，即使当前机器不能运行它。

v0.3 证明 adapter extensibility；v0.4 要证明 portable capsule readiness。重点不是让报错更漂亮，而是让 Consumer Mode 在不执行未信任源码的前提下解释 `.skr` 或 capsule 当前能否被消费、为什么不能运行、下一步应该怎么修复。

## 北极星

**Portable Consumer Checks**

让用户拿到一个 `.skr` 或 capsule 文件夹后，可以先运行：

```bash
skillrun inspect --cwd <capsule>
skillrun check --cwd <capsule>
```

即使当前机器缺 Python、Node 或 Pydantic，用户和 Agent 也能知道：

- 这个 capsule 是什么。
- Manifest 是否 fresh。
- entrypoint 和 adapter 是什么。
- 当前 host 缺什么 runtime dependency。
- 为什么不能直接 `run` 或通过 MCP `tools/call` 调用。

## 设计原则

- Consumer Mode 继续只信静态 Manifest，不为 dependency check 动态 import `action.py` / `action.mjs`。
- `inspect` 看 Manifest contract；`check` 看当前 host readiness；`doctor` 是人类友好诊断入口。
- dependency check 只诊断，不自动安装。
- `.skr` 仍不是 runtime image、dependency bundle、registry package 或 secure install format。
- 缺 dependency 是稳定的一等错误：`DependencyError`。
- stale Manifest 优先级高于 dependency failure；source mismatch 必须先 fail closed。
- MCP server 不能因为一次 tool call 的 dependency failure 崩溃。
- HTTP、registry、signed package、vendoring、sandbox、package-manager 管理都不进入 v0.4。

## Requirement Map

### Functional Requirements

- FR-001: `skillrun check --cwd <capsule>` must statically diagnose capsule files, Manifest freshness, source hashes, entrypoint presence, example presence and runtime requirements without importing action source.
- FR-002: SkillRun must record or infer a minimal runtime dependency contract from Manifest adapter defaults, including executable name/version and required adapter packages.
- FR-003: Python readiness must report executable presence, detected Python version, required Python version and Pydantic v2 package status.
- FR-004: Node readiness must report executable presence, detected Node version and required Node version for JS Alpha capsules.
- FR-005: `test` and `run` must return structured `DependencyError` envelopes when adapter runtime dependencies are missing or incompatible.
- FR-006: MCP `tools/call` must surface dependency failures as tool errors while keeping the MCP server process alive.
- FR-007: `.skr` archives must remain dependency-free but unpacked capsules must be inspectable and checkable.
- FR-008: `doctor` must reuse or align with the `check` readiness model while remaining human-friendly and non-executing.
- FR-009: Documentation must state the inspect/check/doctor boundary and avoid install, sandbox or runtime-image claims.

### Non-functional Requirements

- NFR-001: v0.4 must preserve the honest security model: dependency checking is not sandboxing.
- NFR-002: v0.4 must not introduce automatic dependency installation, package-manager orchestration or vendoring.
- NFR-003: Consumer Mode diagnostics must be deterministic and suitable for CI usage.
- NFR-004: Hostile environment scenarios must have automated tests or explicitly documented exceptions.
- NFR-005: Existing Python stable and JS Alpha command paths must remain compatible.

## In Scope

### P0: `check` Command And Readiness Engine

- Add `skillrun check --cwd <capsule>`.
- Use Manifest and static files only.
- Report status, reason, required values, detected values and next step.
- Keep `doctor` as a human-friendly wrapper or aligned diagnostic surface.

### P0: Manifest Runtime Requirements

- Add a minimal `runtime.requirements` contract or equivalent adapter-default fallback.
- Python default: executable `python`, version `>=3.10`, package `pydantic>=2,<3`.
- Node default: executable `node`, version `>=18`, no package-manager dependency for JS Alpha.
- Preserve legacy Manifest behavior with clear `check` notes.

### P0: DependencyError

- Promote `DependencyError` to a valid error envelope code.
- Convert missing executable, unsupported executable version and missing adapter package into `DependencyError`.
- Preserve logs for low-level details without leaking stack traces into display markdown.

### P0: MCP Resilience

- Map `DependencyError` into MCP tool error results.
- Ensure the MCP stdio server keeps serving after one dependency failure.

### P1: Portable `.skr` Diagnosis

- Ensure unpacked `.skr` can run `inspect` and `check`.
- Keep `.skr` dependency-free.
- Document the difference between package diagnosability and runtime reproducibility.

### P1: Documentation And Release Matrix

- Update README, SSOT, testing docs and release notes.
- Add release-level matrix evidence for hostile host environments.

## Out Of Scope

- HTTP transport.
- Custom REST API.
- Marketplace / registry.
- Signed package.
- Dependency vendoring.
- Automatic install for Python, Node, Pydantic or npm packages.
- virtualenv、uv、pip、poetry、npm、pnpm、yarn management.
- Full TypeScript runtime support.
- OS sandbox.
- Multi-action orchestration.

## Success Criteria

- A user can inspect and check an unpacked `.skr` on a machine that cannot run it.
- Missing Python, missing Node and missing Pydantic no longer surface as raw spawn/import errors.
- `DependencyError` is stable across CLI and MCP tool calls.
- MCP server survives dependency failures.
- Docs make clear that SkillRun diagnoses dependencies but does not install them.

## Review Gate

- Approval: Pending.
- Reviewer notes: This spec is ready for user review. It does not authorize implementation until the user confirms the v0.4 scope and work graph.
