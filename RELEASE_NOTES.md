# SkillRun Release Notes

## v0.5.6

Status: Ready_For_Release_Decision
Prepared on: 2026-05-17
Publication: no v0.5.6 tag, main merge, GitHub Release, package publication, registry entry, or artifact publication has been performed

### Headline

SkillRun adds release polish and the first explicit headless consumer control-plane contracts before Desktop: `skillrun consumer inventory --json` and `skillrun consumer exposure --json`.

### What Is Included

- Maintainer-oriented release checklist documentation.
- CI failure diagnostics review for `cargo test` GitHub annotations.
- Headless consumer contract documentation for future Desktop, Router, mount planning, exposure, and run history surfaces.
- Run history contract review defining registry-scoped list semantics, input privacy boundaries, and why `runs inspect` should not be bundled into v0.5.6 by default.
- Mount plan contract review defining Router-only mounting, plan-first output, and why apply/rollback should not be bundled into v0.5.6 by default.
- `skillrun consumer inventory --json` as a stable capsule inventory surface for Desktop, Router, and automation consumers.
- `skillrun consumer exposure --json` as a read-only Manifest-derived tool exposure plan for future Router consumers.
- `skillrun consumer runs list --json` as a registry-scoped run evidence summary for future Envelope Explorer consumers.
- `skillrun consumer mount plan --client <id> --json` as a plan-only MCP client configuration preview.
- Contract fixture coverage for enabled consumer inventory output.
- Registry degradation coverage showing consumer inventory tolerates missing capsule paths and invalid Manifest entries without failing the whole list.
- Exposure coverage showing disabled capsules and no-longer-ready enabled capsules are not exposed.
- Run list coverage showing summary output omits full input/envelope/log content and degrades invalid run records without failing the whole list.
- Mount plan coverage showing missing configs, existing configs, unsupported clients, and unparseable configs do not mutate real client files.

### Boundaries

- v0.5.6 does not add Desktop, Tauri, `skillrun ui`, a daemon, Router runtime, MCP client config mutation, `.skr import`, marketplace, signed package trust, dependency installation, or OS sandboxing.
- `consumer inventory --json` and `consumer exposure --json` are read-only control-plane surfaces over local registry readiness semantics.
- `consumer runs list --json` is a read-only summary over registered capsules only; it is not a global run database and does not include full input, artifact content, log content, or `runs inspect`.
- `consumer mount plan --client <id> --json` is plan-only. v0.5.6 does not implement apply/rollback and does not modify real MCP client configuration.
- Mount plan targets the future SkillRun Router and emits a warning because v0.5.6 does not add Router runtime.
- Run history remains evidence-first; Desktop should consume Core JSON surfaces instead of reading `.skillrun/runs` directly.
- Registry remains inventory, not a trust store; `enabled=true` remains future exposure intent and does not mean trusted, sandboxed, installed, or runnable.
- `.skr` remains an import/distribution artifact, not a direct MCP runtime entry.

### Validation

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`
- `cargo test --test registry`
- `cargo test --test mount_plan`
- `cargo test --test consumer_json_contracts`
- docs relative links check
- Remote CI passed on `codex/v0.5.6-release-polish`: https://github.com/iiwish/skillrun/actions/runs/25991247448

## v0.5.5

Status: Released
Prepared on: 2026-05-17
Publication: v0.5.5 main merge, remote push, tag, and GitHub Release publication completed; no package registry publication was performed

### Headline

SkillRun hardens the Manifest-driven Consumer Mode contract before Desktop: execution, MCP exposure, and `.skr` distribution now share the same static Manifest validation boundary.

### Version Layers

- Binary/crate version is `0.5.5`.
- Manifest IR `manifest_version` remains `0.1.0`.
- IPC / Adapter `protocol_version` remains `0.1.0`.
- Adapter Protocol remains `adapter.v1` for Level 0 command adapters.

### What Is Included

- Shared `ManifestView` access helpers for frequently read Manifest fields.
- Consumer Mode static Manifest contract validation in `consumer::validate`.
- Runtime fail-closed behavior for missing `runtime.adapter` / `runtime.entrypoint`.
- Manifest schema contract validation before Manifest generation, readiness success, runtime execution, MCP dry-run contract output, and `.skr` archive creation.
- Core schema validator support for `minLength` and `minimum`.
- Rejection of unsupported or malformed schema `type` declarations.
- Adapter timeout handling moved into a shared process helper.
- Best-effort process-tree termination on timeout: Windows uses `taskkill /T /F`; Unix uses process groups with `TERM` / `KILL`.
- Release gate review documenting command-by-command Consumer Mode boundaries.

### Boundaries

- v0.5.5 does not add Desktop, Tauri, Router, daemon, MCP client mounting, marketplace, signed package verification, dependency installation, or OS sandboxing.
- Schema validation is a SkillRun-supported JSON Schema subset, not a full JSON Schema engine claim.
- `serve --mcp` validates the Manifest at server startup and uses that startup snapshot for the stdio server lifecycle; it does not hot-reload Manifest changes.
- Registry remains local inventory. `registry add` does not mean trust, enablement, install, or runnable certification.
- Adapter timeout cleanup is best-effort process lifecycle control, not a security sandbox.
- `.skr` remains a source + Manifest archive. It is not signed, not a dependency bundle, not a registry package, and not a reproducible runtime image.

### Validation

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`
- Remote CI passed on the release branch and main before tag publication.

## v0.5.4

Status: Ready_For_Release_Decision
Prepared on: 2026-05-15
Publication: no v0.5.4 tag, remote push, package publication, registry entry, or artifact publication has been performed

### Headline

SkillRun hardens Core contracts before Desktop: command readiness no longer executes arbitrary command adapter probes, Manifest schemas are enforced by Core, bad registry entries no longer poison inventory, and Desktop-facing JSON contracts are frozen as fixtures.

### Version Layers

- Binary/crate version is `0.5.4`.
- Manifest IR `manifest_version` remains `0.1.0`.
- IPC / Adapter `protocol_version` remains `0.1.0`.
- v0.5.4 is the stabilization milestone name for this integration line, not evidence that a public tag has been created.

### What Is Included

- Non-executing command adapter readiness: `check` and `switchboard enable` only resolve executable presence for Level 0 command capsules.
- Core runtime input schema validation before adapter launch.
- Core runtime output schema validation for successful adapter envelopes.
- `ValidationError` for invalid user input and `ProtocolViolation` for invalid adapter output.
- Registry/switchboard per-entry degradation for corrupt or unreadable Manifest entries.
- Consumer JSON contract fixtures for runnable `inspect/check/doctor`, instruction-only `inspect/check`, registry enabled, and switchboard enabled states.
- Documentation of JSON fixture compatibility rules and version-layer semantics.

### Boundaries

- v0.5.4 does not add Desktop, Tauri, Router, daemon, MCP client mounting, marketplace, signed package verification, dependency installation, or OS sandboxing.
- JSON fixtures normalize paths, timestamps, hashes, and local dependency versions; they freeze contract shape and state semantics, not a single machine's environment.
- Manifest schema enforcement is a pragmatic v0 subset and not a full JSON Schema engine claim.
- Desktop should be a separate project consuming CLI JSON contracts, registry/switchboard state, run records, and a future Core API. It should not redefine Manifest schema, execute actions directly, or parse MCP text as audit evidence.

### Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --test consumer_json_contracts`
- `cargo test --test registry`
- `cargo test --test runtime --test errors --test mcp_server`

## v0.5.3

Status: Ready_For_Release_Decision
Prepared on: 2026-05-15
Publication: no v0.5.3 tag, remote push, package publication, registry entry, or artifact publication has been performed

### Headline

SkillRun adds a local Capsule Registry and Switchboard: users and future Router/Desktop consumers can see registered local capsules and explicitly enable or disable future exposure intent.

### What Is Included

- `skillrun registry add --cwd <capsule> [--id <id>]` for local capsule inventory.
- `skillrun registry list --json` and `skillrun registry inspect <id> --json` for machine-readable inventory and readiness views.
- `skillrun registry remove <id>` for removing local registry state without deleting capsule files.
- `skillrun switchboard list --json` for enabled/disabled state.
- `skillrun switchboard enable <id>` with fail-closed readiness gates.
- `skillrun switchboard disable <id>` for turning off future exposure intent.
- Missing registered capsule paths are represented as per-capsule `readiness.status="missing-path"` instead of failing the whole inventory command.
- Tests for empty registry, duplicate ids, missing local paths, add/list/inspect/remove, enable/disable, stale Manifest, instruction-only, and dependency-error gates.

### Boundaries

- Registry is local inventory, not a marketplace, package index, trust registry, or install source.
- Switchboard `enabled=true` means future Router exposure intent. It does not mean trust, sandboxing, dependency installation, or MCP client mounting.
- v0.5.3 does not import or unpack `.skr`.
- v0.5.3 does not add SkillRun Router, daemon, Tauri/Desktop, MCP client config mutation, signed packages, dependency vendoring, dependency installation, or OS sandboxing.
- Enable gates use Consumer Mode readiness and do not import action source for metadata.

### Validation

- `cargo test --test registry`
- `cargo test --test registry --test consumer_guards --test instruction_only`
- `cargo test`
- `git diff --check`

## v0.5.2

Status: Ready_For_Release_Decision
Prepared on: 2026-05-15
Publication: no v0.5.2 tag, remote push, package publication, registry entry, or artifact publication has been performed

### Headline

SkillRun adds the Consumer JSON Surface: `inspect`, `check`, and `doctor` now have stable `--json` output for Desktop, Router, and automation consumers without changing human CLI output.

### What Is Included

- `skillrun inspect --json` for Manifest contract summaries across runnable, invalid-runnable, and instruction-only capsule states.
- `skillrun check --json` for readiness reports backed by the existing readiness engine.
- `skillrun doctor --json` using the same readiness JSON schema as `check --json`, differing by `command`.
- JSON contract tests for runnable, stale Manifest, dependency/readiness, and instruction-only states.
- Governance packets and evidence for T056, T057, and T058.
- README and v0.5.2 contract docs updated for the implemented surface.

### Boundaries

- Human text output remains the default.
- `skillrun test` and `skillrun run` are not wrapped; they already output standard output/error envelope JSON.
- Parser and filesystem errors remain stderr + non-zero exit code.
- This release does not introduce registry, router, daemon, Tauri/Desktop UI, MCP client config mutation, signed packages, dependency installation, or sandbox semantics.
- JSON readiness still reads Manifest, files, hashes, examples, and runtime probes; it does not import action source for metadata.

### Validation

- `cargo test --test inspect`
- `cargo test --test consumer_guards --test instruction_only --test cli`
- `cargo test`
- `git diff --check`

## v0.5.0

Status: Ready_For_Release_Decision
Prepared on: 2026-05-14
Publication: no v0.5.0 tag, remote push, package publication, registry entry, or artifact publication has been performed

### Headline

SkillRun defines the language-agnostic Adapter Protocol and proves it with a Level 0 command adapter: any explicit argv command can act as a SkillRun action process when it obeys the IPC and output envelope contract.

### What Is Included

- `docs/adapter-protocol.md` as the public Adapter Protocol contract.
- `docs/v0.5-adapter-protocol.md` as the v0.5 design and boundary document.
- Adapter conformance tests that map Python stable and JS alpha behavior to the protocol contract.
- Manifest generation for `runtime.adapter = "command"` with explicit argv command and static JSON schemas.
- Consumer/readiness diagnostics for command executable presence without importing action source for metadata.
- Runtime dispatch for Level 0 command adapter processes using standard SkillRun IPC environment variables.
- stdout/stderr discipline for command adapter processes: logs only, never structured result fallback.
- `examples/command_hello` as a runnable SDK-free command adapter reference capsule.
- README, business example catalog, release report and version metadata updates.

### Boundaries

- This is an Adapter Protocol and Level 0 command adapter release, not a broad language-support release.
- The command adapter uses explicit argv only. It does not accept shell strings.
- It does not install Python, Node, Ruby, PHP, npm packages, virtualenvs or any command dependency.
- It does not vendor dependencies into `.skr`.
- It does not introduce registry, marketplace, `skillrun install`, signed capsules or trusted download behavior.
- It does not turn SkillRun into an OS sandbox. Running a command adapter still executes host code.
- Python remains the stable action adapter target; JS remains alpha; command adapter is protocol-level execution, not a new blessed SDK.

### Validation

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`
- Delivery artifact validator for T055

## v0.4.2

Status: Ready_For_Release_Decision
Prepared on: 2026-05-14
Publication: no v0.4.2 tag, remote push, package publication, registry entry, or artifact publication has been performed

### Headline

SkillRun sharpens its public positioning and adds three official reference capsules that demonstrate reusable SOP-backed preflight patterns without expanding the runtime scope.

### What Is Included

- `docs/positioning.md` for the core project positioning: Manifest-driven runtime and packaging toolchain for SOP-backed agent skills.
- `docs/vision.md` for long-term direction, including staged trust evolution and v0.5 language-agnostic Adapter Protocol direction.
- `docs/trust-model.md` for the current trust boundary: manifest-bound execution and inspection, not OS sandboxing.
- `docs/v0.4.2-official-capsules.md` for the official reference capsule design.
- `examples/commit_message_gate` as a Python stable reference capsule for Conventional Commits validation.
- `examples/bounded_file_patcher` as a Python stable reference capsule for exact old-text/new-text replacement inside declared project directories.
- `examples/readonly_diagnostics_runner` as a Python stable reference capsule for named read-only diagnostics without arbitrary shell strings.
- README, docs index and business example catalog updates.

### Boundaries

- This is a positioning and example-led patch release.
- It does not introduce a registry, marketplace, `skillrun install`, signed package or trusted download story.
- It does not introduce a new language adapter or the v0.5 Adapter Protocol.
- The diagnostics runner is not a general-purpose shell.
- The file patcher is not an OS sandbox.
- The commit message gate does not stage files automatically.
- `.skr` remains a source + Manifest archive.

### Validation

- `cargo fmt --check`
- `cargo test --test business_examples`
- `cargo test`
- `git diff --check`

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
