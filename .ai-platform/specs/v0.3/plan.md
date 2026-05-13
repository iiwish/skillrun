# SkillRun v0.3 Implementation Plan

Version: v0.3
Status: Confirmed
Source spec: `.ai-platform/specs/v0.3/spec.md`
Last updated: 2026-05-13
Review: Approved by user on 2026-05-13 after plan/tasks review request

## 一句话判断

v0.3 的实现计划应先抽出 adapter boundary，再用最小 `action.mjs` 路径验证 JS alpha，最后补齐 adapter-aware diagnostics 和文档边界。

## Decision Summary

### D001: Adapter Boundary First

SkillRun Core 继续使用 Rust 实现，但 metadata extraction 和 runtime execution 必须通过 adapter dispatch 进入 Python 或 Node adapter。

Implication:
- `manifest` 不再直接调用 `python::extract_schemas`。
- `runtime` 不再直接调用 `python::run_action`。
- Python path 作为 regression baseline，不改变用户行为。

### D002: JS Alpha Uses Canonical ESM

v0.3 只支持 `action.mjs`，通过 Node ESM import 加载 action module。

Implication:
- 不支持 `action.js` 的 CJS/ESM 歧义。
- 不支持 `action.ts` 直接运行。
- Node 缺失时给出清晰错误，而不是自动安装。

### D003: JS Schema Is Explicit JSON Schema

JS action 必须导出 `inputSchema` 和 `outputSchema`。

Implication:
- 不做 TypeScript type-to-schema。
- 不从 examples 猜 schema。
- 后续 SDK 可以提供 Zod / TypeBox sugar，但 v0.3 不把它作为 runtime contract。

### D004: Runtime Envelope Remains Shared

Python 和 JS adapter 都必须写入 `SKILLRUN_OUTPUT_JSON`，并复用现有 envelope、artifact、permission、run record 校验。

Implication:
- stdout/stderr 仍只作为日志。
- JS adapter 不绕过 artifact containment。
- MCP `tools/call` 不需要知道 action language。

### D005: Diagnostics Must Be Adapter-aware But Non-executing

`doctor` / `validate` 只检查 capsule structure、Manifest freshness、schema/example consistency 和 release-facing warnings。

Implication:
- 不执行业务 `run`。
- 不把缺失 Node dependency 包装成 SkillRun 自动解决事项。
- 对 `action.ts` 给出诚实的 out-of-scope 恢复建议。

### D006: Language Flags Are Authoring-time Only

Language selection belongs to authoring setup, not runtime consumption.

Implication:
- `skillrun init refund --python` remains the README main Quickstart.
- `skillrun init refund --py` is a short alias for `--python`.
- `skillrun init refund --js` creates the JS alpha capsule.
- `skillrun init refund` has no implicit default language.
- `skillrun manifest` resolves adapter by `skillrun.config.json` first, then deterministic action-file convention.
- `skillrun test`、`skillrun run`、`skillrun serve --mcp`、`skillrun pack` do not accept language selection flags.

## Constitution Check

- Product identity: Satisfied. Docs keep `SkillRun` for project name and `skillrun` for CLI/crate/code identifiers.
- Rust Core boundary: Satisfied. JS support is an adapter path; SkillRun 本体仍由 Rust 实现。
- Manifest as runtime IR: Satisfied. JS alpha strengthens the Manifest-driven boundary instead of replacing it.
- Consumer Mode fail closed: Satisfied. Consumer Mode still validates static Manifest source hashes and does not import source for metadata.
- Honest security narrative: Satisfied. Plan explicitly excludes sandbox, dependency vendoring, runtime image and signed package claims.
- MVP-only Python wording: Not a violation. Constitution says Node is post-MVP; v0.3 is post-v0.2 planning. SSOT now narrows Node/JS to `action.mjs` alpha.

## Architecture Plan

### Current State

- `src/manifest.rs` directly requires `action.py` and calls `python::extract_schemas`.
- `src/runtime.rs` reads `runtime.adapter` but only accepts `python`, then calls `python::run_action`.
- `src/init.rs` only creates Python templates and currently requires `--python`.
- Consumer, MCP and pack are already Manifest-driven and should remain language-neutral.

### Target Shape

```text
src/adapters/
  mod.rs       adapter dispatch and shared request/output structs
  python.rs    existing Python metadata/run implementation
  node.rs      JS alpha metadata/run implementation

templates/
  python/
  js/
```

Adapter dispatch should expose two operations:

- `extract_schemas(adapter, capsule_dir, entrypoint) -> Schemas`
- `run_action(adapter, ActionRunRequest) -> ActionRunOutput`

The exact Rust type shape can be a trait or a narrow match-based dispatcher. The important boundary is behavioral: Core chooses by Manifest/config adapter, not by hard-coded Python assumptions.

### CLI Language Semantics

```text
init       explicit template flag: --python / --py / --js
manifest   Author Mode adapter resolution: config first, then unique action-file convention
test       Manifest only
run        Manifest only
serve      Manifest only
pack       Manifest only
```

`--py` must produce exactly the same capsule as `--python`. It is a DX alias, not a separate language path.

`manifest` convention rules:

- `skillrun.config.json` with `runtime.adapter` and `runtime.entrypoint` wins.
- If config does not specify runtime, exactly one known action file may select adapter: `action.py` or `action.mjs`.
- Multiple known action files are ambiguous and must fail closed.
- `action.ts` is unsupported in v0.3 and should suggest compiling to `action.mjs`.

### JS Action Contract

Canonical v0.3 JS action:

```javascript
export const inputSchema = { type: "object", properties: {}, required: [] };
export const outputSchema = { type: "object", properties: {}, required: [] };

export function preflight(input, ctx) {
  // optional
}

export async function run(input, ctx) {
  return { /* output */ };
}
```

Allowed return shapes should mirror Python:

- direct output object
- `{ output, artifacts, display }`

Structured failures should map to the existing envelope categories where feasible:

- input validation or schema mismatch -> `ValidationError`
- `preflight` / business policy failure -> `PolicyViolation`
- malformed adapter output -> `ProtocolViolation`
- unexpected thrown error -> `RuntimeError`

## Delivery Slices

### Slice 1: Safety Net And Adapter Boundary

Goal:
Preserve Python behavior while removing Python-only coupling from Core.

Includes:
- Characterization tests for current Python path.
- Adapter dispatcher.
- Manifest/runtime refactor.

### Slice 2: JS Author Path

Goal:
Create and compile a JS alpha capsule into a Manifest.

Includes:
- `init --js`.
- `init --py` as an alias for `init --python`.
- JS templates.
- Node metadata extraction from explicit JSON Schema exports.

### Slice 3: JS Runtime Path

Goal:
Execute JS alpha capsule through the same run, artifact, evidence and error envelope path.

Includes:
- Node runtime adapter.
- Sync/async `run`.
- `preflight`.
- runtime tests and E2E matrix.

### Slice 4: Manifest-derived Surfaces

Goal:
Prove JS alpha does not create a parallel product surface.

Includes:
- `inspect`.
- `serve --mcp` dry-run and stdio client matrix.
- `pack`.

### Slice 5: Author Quality And Docs

Goal:
Make the new language path understandable without expanding claims.

Includes:
- adapter-aware diagnostics.
- README updates.
- TypeScript boundary notes.
- Capsule quality checklist.

## Test Strategy

- Unit tests: adapter selection, timeout handling, schema extraction error mapping, JSON Schema shape validation.
- Contract tests: CLI usage, `--py` alias behavior, generated Manifest fields, source hashes, Consumer Mode stale guards.
- Runtime tests: JS success, validation failure, policy failure, protocol violation, artifact containment.
- E2E tests: `init --js -> manifest -> inspect -> test -> run -> serve --mcp --dry-run -> pack`.
- MCP tests: existing stdio client matrix must remain language-neutral.
- Regression tests: Python path must pass the existing release matrix unchanged.

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| JS alpha is perceived as full TypeScript support | Scope explosion | Docs and errors state `action.mjs` only; `action.ts` is out of scope |
| Adapter refactor breaks Python path | Release regression | Characterization tests before refactor; full `cargo test` after each slice |
| Node availability varies by machine | Fragile tests | Tests detect clear Node error; CI/release matrix records Node requirement |
| Schema export format becomes future debt | Manifest instability | Use plain JSON Schema as adapter contract; SDK sugar deferred |
| JS runtime bypasses envelope/artifact checks | Security narrative drift | Reuse existing output envelope and artifact validation in Rust runtime |
| Language flags leak into Consumer Mode | Manifest boundary erosion | Tests reject language flags on `test`、`run`、`serve --mcp` and `pack` |

## Out Of Scope For This Plan

- Full TypeScript support.
- `action.ts` direct runtime.
- TypeScript type-to-schema.
- CJS support.
- implicit default language for `skillrun init refund`.
- language flags on Consumer Mode commands.
- package manager install.
- dependency vendoring.
- sandbox.
- HTTP/SSE/Streamable HTTP MCP transport.
- registry/marketplace/install flow.

## User Review Gate

- Approval: Approved on 2026-05-13.
- Reviewer notes: Plan reviewed with no blocking findings. Work graph may be confirmed, execution packets may be created, and T019 may move to `Ready`.
