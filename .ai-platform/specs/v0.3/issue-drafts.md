# SkillRun v0.3 Curated Issue Drafts

Version: v0.3
Status: Ready_For_User_Review
Created: 2026-05-12
Updated: 2026-05-13

这些 issue draft 用于发布 v0.2.0 后引导社区贡献。目标是主动限制项目方向：接受 authoring quality、adapter boundary 和 JS Action Alpha 相关贡献，避免被拉向通用 Agent framework、HTTP server、marketplace、sandbox 或完整 TypeScript 工具链。

## Label Set

- `scope:v0.3`
- `scope:post-v0.3`
- `type:docs`
- `type:author-dx`
- `type:adapter`
- `type:js-action`
- `type:typescript`
- `type:mcp-compat`
- `type:diagnostics`
- `good first issue`
- `needs-design`

## Curated Issues

### 1. Improve 10-minute Quickstart

Labels: `scope:v0.3`, `type:docs`, `type:author-dx`, `good first issue`

Goal:
让 README 中的新作者路径更像真实使用，而不是命令列表。Python 仍是主路径；JS alpha path 只作为紧凑补充。

Acceptance:
- Covers Python `init -> manifest -> inspect -> test -> serve --mcp --dry-run -> serve --mcp`.
- Keeps `--python` as the README main Quickstart and mentions `--py` only as an alias/reference detail.
- Adds a compact JS alpha path only after the Python path is clear.
- Explains that `init` requires explicit template selection, while runtime commands read Manifest.
- Explains when to use `--dry-run`.
- Does not claim sandbox, registry, dependency bundling, runtime image or full TypeScript support.

### 2. Extract Adapter Boundary From Python-only Runtime

Labels: `scope:v0.3`, `type:adapter`, `needs-design`

Goal:
把当前 Python-only 直接调用路径收敛成清晰 adapter boundary，让 Core 按 Manifest dispatch adapter。

Acceptance:
- Python adapter keeps current behavior.
- Runtime execution dispatches by `runtime.adapter`.
- Manifest generation dispatches metadata extraction by adapter.
- `manifest` resolves adapter by config-first deterministic convention when generating Manifest in Author Mode.
- Consumer Mode still validates Manifest before runtime execution.
- No behavior change to MCP contract shape.

### 3. Implement JS Action Alpha With `action.mjs`

Labels: `scope:v0.3`, `type:adapter`, `type:js-action`

Goal:
提供最小 JS Action path，证明 SkillRun 支持第二语言 adapter，同时不接管 Node/TypeScript 工具链。

Acceptance:
- `skillrun init refund --js` creates `action.mjs`, `SKILL.md`, `examples/default.input.json`, and `skillrun.config.json`.
- `skillrun.config.json` sets `runtime.adapter` to `node` and `runtime.entrypoint` to `action.mjs`.
- `action.mjs` exports `inputSchema`, `outputSchema`, optional `preflight`, and `run`.
- `skillrun init refund` without a language flag remains invalid.
- JS capsule passes `manifest`, `inspect`, `test`, `run`, `serve --mcp --dry-run`, real `serve --mcp` client matrix if feasible, and `pack`.
- No npm install, dependency vendoring, `ts-node`, `tsx`, CJS support, or network dependency.

### 4. Document TypeScript Boundary

Labels: `scope:v0.3`, `type:typescript`, `type:docs`

Goal:
明确 v0.3 不提供完整 TypeScript support，避免社区把 JS alpha 解读成 SkillRun 要接管 TS build pipeline。

Acceptance:
- States that v0.3 stable path is `action.mjs`.
- States that `action.ts` direct runtime, type-to-schema extraction, `ts-node`, `tsx`, source maps and package manager integration are out of scope.
- Optionally documents that authors may compile TypeScript to `action.mjs` themselves.
- Does not introduce an experimental CLI flag unless a separate approved task defines it.

### 5. Design Adapter-aware `skillrun doctor` / `skillrun validate`

Labels: `scope:v0.3`, `type:author-dx`, `type:diagnostics`, `type:adapter`, `needs-design`

Goal:
设计一个不执行业务 action 的 capsule 诊断入口，并让诊断能解释 Python 与 JS capsule。

Acceptance:
- Checks Manifest presence/freshness.
- Checks required capsule files for Python and JS paths.
- Checks examples exist and are schema-compatible if feasible without running action.
- Reports instruction-only status clearly.
- Reports unsupported TypeScript path honestly.
- Does not dynamically import action in Consumer Mode.

### 6. Improve Stale Manifest Recovery Messages

Labels: `scope:v0.3`, `type:author-dx`, `type:adapter`, `good first issue`

Goal:
当 `SKILL.md`、`action.py`、`action.mjs` 或 config stale 时，错误信息直接告诉作者下一步怎么恢复。

Acceptance:
- Error identifies stale source path.
- Error suggests `skillrun manifest` when appropriate.
- Error explains whether the capsule appears Python, JS alpha, or instruction-only.
- Error does not expose irrelevant stack trace details.

### 7. Explain Manifest-derived MCP Contract In Inspect Output

Labels: `scope:v0.3`, `type:author-dx`, `type:mcp-compat`, `type:adapter`

Goal:
让 `skillrun inspect` 更清楚地解释 MCP tool/resource 来自 Manifest，而不是来自 live source import。

Acceptance:
- Shows primary tool name.
- Shows adapter and entrypoint.
- Shows resource exposure summary.
- Mentions Consumer Mode no metadata import.
- Keeps output compact enough for terminal use.

### 8. Improve Python And JS Action Templates

Labels: `scope:v0.3`, `type:author-dx`, `type:js-action`, `good first issue`

Goal:
让 `init --python` / `init --py` 和 `init --js` 生成的 action 更像可学习的 SkillRun action，而不是普通脚本。

Acceptance:
- `--py` is documented and tested as an alias for `--python`, not a separate template.
- Python template explains `Input`, `Output`, `preflight`, `run`.
- JS template explains `inputSchema`, `outputSchema`, `preflight`, `run`.
- Shows where structured errors belong.
- Avoids real external API calls, package dependencies or secrets.

### 9. Add Capsule Quality Checklist

Labels: `scope:v0.3`, `type:docs`, `type:diagnostics`

Goal:
为作者提供一个人工 checklist，用来判断一个 Skill Capsule 是否适合给 Agent 调用。

Acceptance:
- Covers SOP clarity, schema quality, example coverage, error recoverability, artifact behavior and permission honesty.
- Covers adapter-specific quality questions without making Python or JS feel like separate products.
- Explicitly states natural-language SOP is not a hard policy engine.

### 10. Track Post-v0.3 Expansion Ideas Without Implementing Them

Labels: `scope:post-v0.3`, `needs-design`

Goal:
为 full TypeScript support、HTTP transport、registry、sandbox、signed package、dependency vendoring 等方向建立停车场，防止它们进入 v0.3。

Acceptance:
- Each idea states why it is out of v0.3.
- Each idea names the trust, protocol or packaging question that must be solved first.
- No implementation work is attached to this issue.

## Maintainer Guidance

优先接受能降低作者摩擦、提升诊断质量、澄清 Manifest/adapter 边界、跑通 JS Action Alpha 的 PR。

暂缓接受会扩大 remote runtime surface、改变 Manifest IR、引入远程 transport、承诺安全隔离、要求包生态治理、或把 v0.3 扩成完整 TypeScript 工具链的 PR。
