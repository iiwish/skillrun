# SkillRun

> Package one SOP and one action into a portable Agent skill.

[简体中文](README.zh-CN.md)

FastMCP turns functions into MCP tools.
SkillRun turns SOP-backed capabilities into **Skill Capsules**.

A Skill Capsule carries what a function signature cannot: typed input/output, preflight checks, structured errors, artifacts, run evidence, declared permissions, and a Manifest-derived MCP contract.

SkillRun is for teams that need the business context, recovery rules, audit trail, and runtime contract to travel with the action. Use FastMCP when you only need to expose a function; use SkillRun when the SOP matters as much as the code.

## Status

SkillRun v0.5.5 is the current release binary/crate version. v0.5.0 defines the language-agnostic Adapter Protocol and proves it with a Level 0 command adapter. The v0.5.2 integration line adds the Consumer JSON Surface; v0.5.3 adds local capsule registry and switchboard state for future Router/Desktop consumers; v0.5.4 stabilizes Core contracts; v0.5.5 hardens Manifest-driven Consumer Mode before a separate Desktop project consumes it.

- Current implementation: v0.2 MCP stdio behavior, v0.3 JS Action Alpha, v0.4 Portable Consumer Checks, v0.4.1 WeCom Team Notice, v0.4.2 official reference capsules, v0.4.3 CI/runtime error stabilization, v0.5 Adapter Protocol with Level 0 command adapter runtime, v0.5.2 Consumer JSON Surface, v0.5.3 local registry/switchboard state, v0.5.4 Core stabilization, and v0.5.5 Manifest-driven Consumer Mode hardening.
- Available today: `skillrun --help`, `skillrun --version`, `skillrun init <name> --python`, `skillrun init <name> --py`, `skillrun init <name> --js`, `skillrun manifest --cwd <capsule>`, `skillrun inspect --cwd <capsule>`, `skillrun inspect --json --cwd <capsule>`, `skillrun check --cwd <capsule>`, `skillrun check --json --cwd <capsule>`, `skillrun doctor --cwd <capsule>`, `skillrun doctor --json --cwd <capsule>`, `skillrun registry add/list/inspect/remove`, `skillrun switchboard list/enable/disable`, `skillrun test --cwd <capsule>`, `skillrun run --cwd <capsule> --input <file>`, `skillrun serve --mcp --cwd <capsule>`, `skillrun serve --mcp --cwd <capsule> --dry-run`, `skillrun pack --cwd <capsule>`, structured error envelopes, `DependencyError`, artifact validation, declared env injection, stale Manifest guards, instruction-only guards, Manifest-derived MCP tools/resources, `.skr` package generation, and release tests for the skeleton/init/manifest/inspect/check/doctor/registry/switchboard/runtime/error/artifact/permission/consumer-guard/MCP/pack paths.
- v0.2 keeps `serve --mcp --dry-run` for contract inspection, but the normal `serve --mcp` path is now a long-running MCP stdio server.
- The SkillRun core, CLI, Manifest, IPC, MCP exposure, and packaging path are implemented in Rust.
- Python `action.py` is the stable action adapter target. JS `action.mjs` is an alpha adapter target. Both are user action languages, not the SkillRun implementation language.
- Level 0 `runtime.adapter = "command"` runs explicit argv commands that obey SkillRun IPC and envelope contracts. It is a protocol-native escape hatch, not a new blessed language SDK.

## Version Layers

SkillRun uses separate version layers:

- `Cargo.toml` and `skillrun --version` identify the binary/crate version.
- Git tags such as `v0.4.3` identify public release boundaries.
- Milestone names such as v0.5.2, v0.5.3, v0.5.4, and v0.5.5 describe integration scope before a release decision.
- Manifest `manifest_version` identifies the Manifest IR schema.
- IPC / Adapter `protocol_version` identifies the Core-to-adapter file protocol.

The current local binary reports `skillrun 0.5.5`; the current generated Manifest IR and IPC protocol versions remain `0.1.0`. v0.5.5 hardens Manifest-driven runtime behavior and Consumer Mode contracts without changing those protocol versions.

## Why SkillRun

Most agent tool systems start with a callable function. SkillRun starts with a business capability:

```text
Skill Capsule = SOP + action code + schema + examples + permissions
Manifest      = compiled runtime contract
Core          = Rust manifest-driven runtime
Adapter       = language bridge for user actions
Package       = .skr source + Manifest archive
```

Use SkillRun when you want an agent-callable capability to carry more than a function signature:

- A `SKILL.md` cognitive contract that explains the SOP.
- Typed input and output schemas.
- Preflight checks for policy and approval boundaries.
- Structured success and error envelopes.
- Artifacts that are recorded as first-class outputs.
- Run records that preserve hashes, logs, and evidence.
- Manifest-derived MCP exposure that does not re-import source code in consumer mode.

If you only need to expose a Python function as an MCP tool, a lighter tool such as FastMCP may be the better fit. SkillRun is designed for SOP-backed capabilities that must be inspectable, testable, and distributable.

## The Core Flow

```text
refund/
  SKILL.md
  action.py                  # Python stable path
  # action.mjs               # JS alpha path
  examples/
    default.input.json
  skillrun.config.json
  .skillrun/
    manifest.generated.yaml

        |
        | skillrun manifest
        v

Manifest-driven contract
  schema
  permissions
  adapter
  tool description
  source hashes

        |
        +-- skillrun inspect
        +-- skillrun inspect --json
        +-- skillrun check
        +-- skillrun check --json
        +-- skillrun doctor
        +-- skillrun doctor --json
        +-- skillrun registry add/list/inspect/remove
        +-- skillrun switchboard list/enable/disable
        +-- skillrun test
        +-- skillrun run --input examples/default.input.json
        +-- skillrun serve --mcp             # MCP stdio server
        +-- skillrun serve --mcp --dry-run   # contract inspection
        +-- skillrun pack
```

The generated Manifest is the runtime contract. Author mode can regenerate it from local sources. Consumer mode reads it, validates source hashes, and refuses to guess when the Manifest is missing or stale.

In v0.2, `skillrun serve --mcp` starts a real MCP stdio server whose tools and resources are still derived from the Manifest.

In v0.4, `skillrun check` is the automation-grade readiness command. It reads the Manifest, source hashes, entrypoint, examples, and runtime requirements to explain whether the current host can consume or run a capsule. It does not import `action.py` or `action.mjs`, and it does not install dependencies.

In v0.5, the Adapter Protocol makes the southbound runtime boundary explicit. Core still reads Manifest, creates IPC paths, validates envelopes and exposes MCP; adapters bridge user action ecosystems back into that contract.

In v0.5.2, `inspect`, `check`, and `doctor` also expose `--json` for Desktop, Router, and automation consumers. `test` and `run` are unchanged because they already emit standard output/error envelope JSON.

In v0.5.3, `registry` records local capsule inventory and `switchboard` records future exposure intent. It does not import `.skr`, expose MCP tools, mount clients, install dependencies, certify trust, or provide sandboxing.

In v0.5.4, Core hardening makes command readiness non-executing, enforces Manifest schemas at runtime, isolates bad registry entries, and freezes Desktop-facing JSON fixtures. Desktop should be a separate project that consumes these stable Core surfaces; it should not redefine Manifest schema, execute actions directly, or parse MCP text as audit data.

In v0.5.5, Consumer Mode uses a shared Manifest static contract before execution, MCP exposure, and `.skr` distribution. Invalid runtime fields or schema contracts fail closed before run records, MCP tool contracts, or package archives are created.

The v0.5.6 release-polish line adds `skillrun consumer inventory --json` and `skillrun consumer exposure --json` as explicit headless consumer control-plane surfaces. They reuse registry readiness semantics and do not add Desktop, daemon, sandboxing, signed package trust, or marketplace behavior.

## Release Candidate Workflow

```bash
skillrun init refund --python
cd refund
# edit SKILL.md
# edit action.py
skillrun manifest
skillrun inspect
skillrun check
skillrun doctor
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp
skillrun pack
```

`--py` is only a short alias for `--python`. Keep `--python` as the main Quickstart because Python is the stable path.

Language flags belong to `init` only. `manifest`, `inspect`, `check`, `doctor`, `test`, `run`, `serve --mcp`, and `pack` read the capsule and its generated Manifest; they do not accept `--python`, `--py`, or `--js`.

`inspect`, `check`, and `doctor` have different jobs:

- `inspect` shows the Manifest contract: SOP summary, schemas, permissions, adapter, entrypoint, examples, and source hashes.
- `check` diagnoses current-host readiness from static capsule data and runtime probes.
- `doctor` is the human-friendly diagnostic view aligned with the same Consumer Mode boundary.

The first hero example is `refund`: a refund decision capsule with policy limits, approval boundaries, typed inputs, structured `PolicyViolation` errors, and auditable run records.

## JS Action Alpha

JS support in v0.3 is intentionally narrow:

```bash
skillrun init refund-js --js
cd refund-js
# edit SKILL.md
# edit action.mjs
skillrun manifest
skillrun inspect
skillrun check
skillrun doctor
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp --dry-run
skillrun pack
```

The JS alpha contract is canonical ESM `action.mjs` with explicit `inputSchema`, `outputSchema`, optional `preflight`, and `run` exports. SkillRun does not infer schemas from TypeScript types, JSDoc, Zod, TypeBox, examples, or package metadata in v0.3.

`action.ts` is not a runtime entrypoint. Authors may compile TypeScript to `action.mjs` themselves, but SkillRun v0.3 does not run `ts-node`, `tsx`, source maps, CJS/ESM compatibility matrices, or package-manager install flows.

## Level 0 Command Adapter

The v0.5 command adapter proves language agnosticism without blessing another language ecosystem:

```json
{
  "runtime": {
    "adapter": "command",
    "command": ["python", "action.py"],
    "timeout": "10s"
  },
  "input_schema": { "type": "object" },
  "output_schema": { "type": "object" }
}
```

Core starts the explicit argv command, injects `SKILLRUN_INPUT_JSON`, `SKILLRUN_CONTEXT_JSON`, `SKILLRUN_OUTPUT_JSON`, and `SKILLRUN_ARTIFACT_DIR`, then validates the output envelope and artifacts. stdout/stderr remain logs only.

See `examples/command_hello` for a minimal SDK-free command capsule. It uses `python action.py` only as a portable command process; it is not the Python adapter and does not use Pydantic metadata extraction.

## Let an Agent Learn a Capsule Before Calling It

SkillRun capsules are designed to be learned before they are called. Give an AI assistant a URL or repository path that points directly to a capsule folder, not just a generic project homepage. The folder should include `SKILL.md`, `skillrun.config.json`, an action entrypoint such as `action.py` or `action.mjs`, and `examples/`.

```text
Learn this SkillRun Capsule before using it:
<capsule-folder-url-or-repo-path>

1. Read SKILL.md for purpose, SOP, prohibited behavior, required context, and recovery guidance.
2. Read skillrun.config.json and the generated Manifest, if present, to confirm adapter and entrypoint.
3. Read action.py or action.mjs only as the action contract for this capsule; do not infer unsupported languages or package-manager behavior.
4. Read examples/default.input.json to understand the expected input shape.
5. If you can access the workspace, run `skillrun inspect --cwd <capsule>`, `skillrun check --cwd <capsule>`, `skillrun doctor --cwd <capsule>`, and `skillrun test --cwd <capsule>`.
6. When calling the MCP tool, do not infer success from stdout. Use the output/error envelope, artifacts, and run record.
```

Use a real capsule folder when publishing a skill. This keeps the model from treating the capsule as a loose function call: it learns the SOP, adapter entrypoint, example input, and failure behavior before it uses the MCP tool.

## What Works Today

The repository currently contains the Rust CLI, `init --python` and `init --py` Python capsule generator, `init --js` JS alpha capsule generator, Manifest generator, inspect renderer, dependency-aware `check`, doctor diagnostics, test/run path, MCP stdio server, MCP dry-run contract renderer, `.skr` package generation, and the B001 `refund` hero example:

```bash
cargo test
cargo run -- --help
cargo run -- --version
cargo run -- init refund --python --output tmp/e2e-init
cargo run -- manifest --cwd tmp/e2e-init/refund
cargo run -- inspect --cwd tmp/e2e-init/refund
cargo run -- inspect --json --cwd tmp/e2e-init/refund
cargo run -- check --cwd tmp/e2e-init/refund
cargo run -- check --json --cwd tmp/e2e-init/refund
cargo run -- doctor --cwd tmp/e2e-init/refund
cargo run -- doctor --json --cwd tmp/e2e-init/refund
cargo run -- registry add --cwd tmp/e2e-init/refund
cargo run -- registry list --json
cargo run -- switchboard enable refund
cargo run -- switchboard list --json
cargo run -- test --cwd tmp/e2e-init/refund
cargo run -- run --cwd tmp/e2e-init/refund --input examples/default.input.json
cargo run -- serve --mcp --cwd tmp/e2e-init/refund --dry-run
cargo run -- pack --cwd tmp/e2e-init/refund
```

Current local binary output:

```text
skillrun 0.5.5
```

The real `serve --mcp` command is a long-running stdio server and is validated by the scripted MCP client release matrix.

The `.skr` package is a source/Manifest archive. It is not signed, does not vendor dependencies, and does not provide a reproducible runtime image. After unpacking, a consumer can still run `inspect` and `check` to understand the capsule and diagnose missing Python, Node, Pydantic, or command executable dependencies without executing action source for metadata.

## Release Candidate Limits

The current integration scope is intentionally narrow:

- MCP transport is stdio only.
- Each capsule exposes one primary Manifest-derived tool.
- Python `action.py` is the stable action adapter target.
- JS `action.mjs` is alpha only and is not full TypeScript support.
- `runtime.adapter = "command"` is Level 0 protocol execution with explicit argv and static schema. It is not shell-string execution, package installation, sandboxing, or a newly blessed language adapter.
- `action.ts`, direct TypeScript runtime execution, `ts-node`, `tsx`, type-to-schema extraction, source maps, CJS compatibility, shell-string commands, npm install flows, and dependency vendoring are out of scope.
- `.skr` is a source + Manifest archive, not a signed package, registry package, dependency bundle, or runtime image.
- `registry` is local inventory, not a marketplace, trust registry, or install source.
- `switchboard enabled=true` is future Router exposure intent, not trust, sandboxing, dependency installation, or MCP client mounting.
- Desktop is a separate future project. It should consume CLI JSON contracts, registry/switchboard state, run records, and a future Core API; it should not bypass `check`, read `.skillrun/` internals as a stable API, redefine Manifest semantics, or parse MCP text content as audit evidence.
- `check` diagnoses dependency readiness; it does not install Python, Node, Pydantic, command executables, npm packages, or create virtual environments.
- Missing runtime dependencies are reported as structured `DependencyError` results for CLI runtime paths and MCP tool calls.
- SkillRun does not provide an OS sandbox. Running a third-party action still means executing third-party code.
- The v0.5.5 release handoff is explicit: merge, tag creation, remote push, and GitHub Release publication are release actions; package publication remains a separate explicit decision.

## Security Model

SkillRun is honest about trust boundaries:

- "No guardrail, no execution" means Manifest contracts, input/output schemas, preflight checks, structured envelopes, artifact containment, run evidence, and Consumer Mode static checks. It does not mean OS sandboxing.
- `stdout` and `stderr` are logs only. Structured results must come from output or error envelopes.
- Consumer mode must not dynamically import untrusted source code to extract metadata.
- `skillrun check` and `skillrun doctor` are Consumer Mode diagnostics; they do not import action source for metadata.
- Stale or missing Manifests fail closed.
- Declared environment variables and artifact paths are part of the runtime contract.
- SkillRun does not claim to be a full OS sandbox. Running a third-party action still means executing third-party code.
- `.skr` is not a secure install format, registry package, or dependency bundle.
- Dependency readiness is not sandboxing, vendoring, or reproducible environment creation.
- Command adapter execution is argv-only and still executes host code. It is not an OS sandbox.

The goal is a small, hard boundary: no implicit execution of instruction-only skills, no stdout success fallback, and no source-code metadata import in consumer mode.

## Roadmap

| Milestone | Focus |
| --- | --- |
| `T001` | Rust CLI skeleton, help, version, unsupported command behavior |
| `T002` | `init --python` capsule skeleton |
| `T003` | Manifest generation, Pydantic v2 schema extraction, source hashes |
| `T004` | `inspect` and instruction-only status |
| `T005` | IPC runtime, output envelopes, run records |
| `T006` | Structured errors and failure discipline |
| `T007` | Artifact containment and declared environment handling |
| `T008` | Consumer guards and stale Manifest behavior |
| `T009` | Manifest-driven MCP exposure |
| `T010` | `.skr` packaging |
| `T011` | End-to-end acceptance matrix and business examples |
| `v0.2` | Real MCP stdio server and public release candidate readiness |
| `v0.3` | Adapter boundary, JS Action Alpha via `action.mjs`, `doctor`, and explicit TypeScript boundary |
| `v0.4` | Portable Consumer Checks, dependency-aware Consumer Mode, `check`, and structured `DependencyError` |
| `v0.5` | Language-agnostic Adapter Protocol and Level 0 command adapter |
| `v0.5.2` | Consumer JSON Surface for `inspect`, `check`, and `doctor` |
| `v0.5.3` | Local Capsule Registry and Switchboard exposure intent |
| `v0.5.4` | Core Stabilization Audit before Desktop |
| `v0.5.5` | Manifest-driven Consumer Mode contract hardening |

## Classic Business Examples

SkillRun's business proof is intentionally narrow:

- `B001: Refund Decision` is implemented in `examples/refund` and tested end-to-end with success, `PolicyViolation`, `ValidationError`, run records, MCP dry-run exposure, and `.skr` packaging.
- `B002: Support Triage` is a docs-level example showing stable routing labels and missing-context recovery.
- `B003: Access Request Approval` is a docs-level example showing approval boundaries, declared environment, and audit notes.
- `B004: Vendor Risk Review` is a docs-level example showing artifact-first review summaries and package distribution without dependency vendoring.
- `B005: WeCom Team Notice` is implemented in `examples/wecom_team_notice` as a v0.4.1 official runnable example. It shows dry-run previews, approval boundaries, declared `WECOM_WEBHOOK_URL`, structured `DependencyError`, markdown artifacts, and MCP usage for a real local notification workflow.
- `B006: Commit Message Gate` is implemented in `examples/commit_message_gate` as a v0.4.2 official reference capsule. It validates concise Conventional Commits subjects without auto-staging files.
- `B007: Bounded File Patcher` is implemented in `examples/bounded_file_patcher` as a v0.4.2 official reference capsule. It applies one exact text replacement inside declared project directories and records a patch artifact.
- `B008: Read-only Diagnostics Runner` is implemented in `examples/readonly_diagnostics_runner` as a v0.4.2 official reference capsule. It runs only named, allowlisted diagnostics without accepting arbitrary shell strings.
- `B009: Command Hello` is implemented in `examples/command_hello` as a v0.5.0 Level 0 command adapter reference capsule. It demonstrates static schema, standard IPC env vars, output envelope writing, artifact output, and stdout-as-log behavior without using a SkillRun SDK.

The runnable examples are intentionally narrow. `refund` proves safety and audit boundaries; `wecom_team_notice` proves a closer day-to-day local workflow without turning SkillRun into a WeCom adapter or API wrapper. The v0.4.2 reference capsules show reusable preflight patterns without claiming OS sandboxing, registry trust, or a general-purpose shell. `command_hello` proves Level 0 adapter protocol execution without making SkillRun a language framework.

## Documentation

- [Documentation index](docs/README.md)
- [MVP contract](docs/mvp.md)
- [Architecture SSOT](docs/ssot.md)
- [Adapter Protocol](docs/adapter-protocol.md)
- [Positioning](docs/positioning.md)
- [Vision](docs/vision.md)
- [Trust model](docs/trust-model.md)
- [v0.4 Portable Consumer Checks](docs/v0.4-portable-consumer-checks.md)
- [v0.4.2 official example capsules](docs/v0.4.2-official-capsules.md)
- [v0.4.3 CI and runtime error stabilization](docs/v0.4.3-ci-stabilization.md)
- [v0.5 Adapter Protocol plan](docs/v0.5-adapter-protocol.md)
- [v0.5.1 Contract Stabilization](docs/v0.5.1-contract-stabilization.md)
- [v0.5.2 Consumer JSON Surface](docs/v0.5.2-consumer-json-surface.md)
- [v0.5.3 Capsule Registry + Switchboard](docs/v0.5.3-capsule-registry-switchboard.md)
- [v0.5.4 Core Stabilization Audit](docs/v0.5.4-core-stabilization-audit.md)
- [v0.5.5 Core Contract Hardening](docs/v0.5.5-core-contract-hardening.md)
- [v0.5.5 Release Gate Review](docs/v0.5.5-release-gate-review.md)
- [v0.6 Consumer Era vision](docs/v0.6-consumer-era-vision.md)
- [Business examples](docs/business-examples.md)
- [Test strategy](docs/testing.md)
- [Release policy](docs/release-policy.md)
- [Branch protection guidance](docs/branch-protection.md)
- [Contributing guide](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Code of conduct](CODE_OF_CONDUCT.md)

Project governance documents are primarily written in Chinese so future agents can parse and maintain the approved product contract consistently.

## Contributing

SkillRun is intentionally narrow. Contributions should preserve these project rules:

- Use `SkillRun` for the project name and `skillrun` for the CLI, crate, commands, and code identifiers.
- Keep SkillRun core behavior in Rust.
- Treat Python as the stable action adapter target and JS `action.mjs` as a narrow alpha adapter path.
- Do not execute instruction-only skills implicitly.
- Do not infer structured success from stdout.
- Do not expand JS alpha into full TypeScript support, package-manager ownership, dependency vendoring, registry behavior, or sandbox claims.
- Keep README and docs clear about what is implemented now versus planned.

Run the baseline checks before submitting changes:

```bash
cargo test
```

## License

SkillRun is licensed under the [Apache License, Version 2.0](LICENSE).
