# SkillRun Release Notes

## v0.4.1

Status: Ready_For_Release_Decision
Prepared on: 2026-05-14
Publication: no v0.4.1 tag, remote push, package publication, registry entry, or artifact publication has been performed

### Headline

SkillRun adds `wecom_team_notice`, an official runnable example that turns a local WeCom group notification workflow into a dry-run-first, approval-bound Skill Capsule, and hardens the Python adapter process environment needed for Windows network calls.

### What Is Included

- `examples/wecom_team_notice` as a Python stable Skill Capsule.
- Dry-run preview path that does not require a real webhook.
- Real send path guarded by `dry_run=false` and declared `WECOM_WEBHOOK_URL`.
- Approval boundary for high, critical and all-hands notices.
- Secret-like content blocking through `PolicyViolation`.
- Missing webhook behavior through structured `DependencyError`.
- Markdown notice artifact and run record evidence.
- Python adapter parity with the Node adapter for baseline process environment variables such as `SystemRoot`, `WINDIR`, temp directories and `PATH`, while still injecting business env vars only when declared in the Manifest.

### Boundaries

- This is an example-led patch release, not a WeCom adapter.
- It is not OpenAPI-to-MCP, a WeCom CLI wrapper, bash action support or hosted server behavior.
- Real webhook sending is manual and opt-in; CI uses dry-run and structured error paths.
- The adapter fix preserves host process basics for Windows runtime viability; it does not relax declared business env permissions or create a sandbox.

## v0.4.0

Status: Released
Prepared on: 2026-05-13
Previous local release handoff: v0.3.0
Publication: v0.4.0 tag and public release artifact publication completed; no registry entry was performed

### Headline

SkillRun adds Portable Consumer Checks: a distributed Skill Capsule can be inspected and dependency-checked from its Manifest without importing untrusted action source.

### What Is Included

- `skillrun check --cwd <capsule>` as the automation-grade readiness command.
- Manifest runtime requirements for Python stable and JS Action Alpha capsules.
- Python readiness checks for Python executable version and Pydantic v2.
- Node readiness checks for Node executable version without npm/package-manager checks.
- Structured `DependencyError` envelopes for missing or incompatible runtime dependencies.
- MCP stdio survival when a tool call returns `DependencyError`.
- `.skr` unpack coverage proving Python and JS capsules can run `inspect` and `check` after distribution.
- Hostile-environment tests for missing Python, missing Node, missing Pydantic, stale Manifest precedence, Consumer Mode no-import behavior, runtime dependency envelopes, MCP survival, and unpacked `.skr` diagnosis.

### Command Boundary

| Command | Responsibility |
| --- | --- |
| `inspect` | Read and display the Manifest contract. It does not judge host runtime readiness. |
| `check` | Diagnose capsule readiness from Manifest, source hashes, examples, entrypoint and runtime probes. |
| `doctor` | Human-friendly diagnostic view aligned with the same non-executing Consumer Mode boundary. |

### Release Matrix

| Area | Evidence | Status |
| --- | --- | --- |
| Error contract | `cargo test --test errors --test cli --test consumer_guards` | Green in T029 validation |
| Manifest requirements | `cargo test --test manifest --test pack` | Green in T030 validation |
| `check` readiness engine | `cargo test --test cli --test consumer_guards --test instruction_only` | Green in T031 validation |
| Runtime discovery | `cargo test --test consumer_guards --test manifest` | Green in T032 validation |
| Runtime `DependencyError` | `cargo test --test runtime --test errors` | Green in T033 validation |
| MCP dependency survival | `cargo test --test mcp_server` | Green in T034 validation |
| Portable `.skr` checks | `cargo test --test pack --test e2e_matrix` | Green in T035 validation |
| Full regression | `cargo test`; `cargo clippy --all-targets -- -D warnings` | Green in T035/T036 validation |

### Boundaries

- v0.4 diagnoses dependencies; it does not install Python, Node, Pydantic, npm packages, virtualenvs or runtime images.
- `.skr` remains a source + Manifest archive. It is not signed, not a registry package, not a dependency bundle, and not a reproducible runtime image.
- SkillRun is not an OS sandbox. Running third-party actions still means executing third-party code.
- HTTP, SSE, Streamable HTTP, hosted server modes, auth and registry behavior remain out of scope.
- JS support remains JS Action Alpha through canonical ESM `action.mjs`; `action.ts` and TypeScript runtime execution remain out of scope.

### Validation

- `cargo test --test pack --test e2e_matrix`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`
- Delivery artifact validator

## v0.3.0

Status: Released_Local
Prepared on: 2026-05-13
Previous public handoff: v0.2.0 local release candidate
Publication: local tag `v0.3.0` created; remote tag push and package publication were not performed

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
- Remote tag push and package publication require separate explicit approval.

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
