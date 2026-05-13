# SkillRun Release Notes

## v0.3.0 Draft

Status: Release_Candidate_Accepted
Prepared on: 2026-05-13
Previous public handoff: v0.2.0 local release candidate
Publication: not tagged, not pushed, and not published by this release-candidate handoff

### Headline

SkillRun keeps the Python capsule path stable while proving the Core is a Manifest-driven multi-adapter runtime through a narrow JS Action Alpha.

### What Is Included

- Stable Python author path: `skillrun init <name> --python`.
- Python shorthand: `skillrun init <name> --py` produces the same Python capsule shape as `--python`.
- JS alpha author path: `skillrun init <name> --js` generates a runnable `action.mjs` capsule.
- Adapter-aware Manifest generation through config-first adapter resolution and deterministic action-file convention fallback.
- Node adapter metadata extraction from explicit `inputSchema` and `outputSchema` exports in `action.mjs`.
- JS runtime execution for `preflight(input, ctx)` and sync or async `run(input, ctx)`.
- Shared output envelope, run record, artifact containment, permission checks, stale Manifest guards, MCP exposure, and `.skr` packaging across Python and JS alpha capsules.
- `skillrun doctor` for adapter-aware, non-executing capsule diagnostics.
- Release-level tests covering Python baseline, MCP stdio, JS alpha local commands, JS MCP surfaces, JS `.skr` packaging, `--py` alias behavior, and Consumer Mode language-boundary guards.

### Release Matrix

| Area | Evidence | Status |
| --- | --- | --- |
| Python baseline | `cargo test`; `tests/e2e_matrix.rs::a001_to_a013_release_matrix_has_fresh_command_evidence` | Green in T028 validation |
| MCP stdio | `cargo test`; `tests/e2e_matrix.rs::a014_mcp_stdio_release_matrix_exercises_full_client_flow` | Green in T028 validation |
| JS alpha local path | `cargo test`; `tests/e2e_matrix.rs::js_alpha_local_command_matrix_covers_init_manifest_inspect_test_and_run` | Green in T028 validation |
| JS MCP and package path | `cargo test`; `tests/mcp_server.rs`; `tests/pack.rs` | Green in T028 validation |
| `--py` alias | `cargo test`; `tests/e2e_matrix.rs::py_alias_manifest_smoke_uses_python_adapter_identity` | Green in T028 validation |
| Diagnostics | `cargo test`; `tests/consumer_guards.rs` doctor coverage | Green in T028 validation |
| Governance artifacts | Delivery artifact validator | Green in T028 validation |

### Boundaries

- JS support is alpha and only recognizes canonical ESM `action.mjs`.
- `action.ts` is not a runtime entrypoint in v0.3. Authors may compile TypeScript to `action.mjs` outside SkillRun.
- SkillRun v0.3 does not run `ts-node`, `tsx`, source maps, CJS/ESM compatibility matrices, or package-manager install flows.
- JS schemas must be explicit JSON Schema exports. SkillRun does not infer schemas from TypeScript types, JSDoc, Zod, TypeBox, examples, or package metadata.
- Runtime commands are Manifest-only. `skillrun test`, `skillrun run`, `skillrun serve --mcp`, and `skillrun pack` do not accept `--python`, `--py`, or `--js`.
- `.skr` remains a source + Manifest archive. It is not signed, not a registry package, not a dependency bundle, and not a reproducible runtime image.
- SkillRun is not an OS sandbox. Running third-party actions still means executing third-party code.
- Version bump, tag creation, remote push, and package publication require a separate explicit release handoff.

### Validation

- `cargo test`
- `cargo run -- --version`
- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`

## v0.2.0

Status: Released_Local

SkillRun v0.2.0 was the first public release handoff.

### Headline

Turn one SOP and one action into a manifest-driven Agent skill capsule with real MCP stdio serving.

### What Is Included

- Real `skillrun serve --mcp` long-running MCP stdio server.
- MCP lifecycle support for `initialize` and `notifications/initialized`.
- Manifest-derived `tools/list`.
- Runtime-backed `tools/call` that preserves SkillRun run records and structured error behavior.
- Manifest-derived `resources/list` and `resources/read` for `SKILL.md` and example inputs.
- Release-level scripted MCP client fixture covering lifecycle, tools, resources and stdout discipline.
- `.skr` packaging as a source + Manifest archive.
- README and release report updated for v0.2.0.

### Boundaries

- MCP transport is stdio only.
- Python `action.py` is the only blessed v0.2 action adapter target.
- One primary Manifest-derived tool per capsule.
- `.skr` is not signed, not a registry package, not a dependency bundle and not a runtime image.
- SkillRun is not an OS sandbox. Running third-party actions still means executing third-party code.

### Validation

- `cargo test`
- `cargo run -- --version`
- `cargo test --test e2e_matrix a014_mcp_stdio_release_matrix_exercises_full_client_flow`
- Delivery artifact validator
