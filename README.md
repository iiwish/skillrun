# SkillRun

> SOP-backed skills for AI agents. No Manifest contract, no execution.

[Simplified Chinese](README.zh-CN.md)

Agent Skills made skills portable. SkillRun makes executable skills dependable.

SkillRun is a Rust runtime and CLI for packaging one SOP and one action into an inspectable, testable, runnable, distributable, and MCP-callable **Skill Capsule**. It is not an alternative Agent Skills standard, not a general Agent framework, not a marketplace, and not an OS sandbox.

## Why It Exists

Most agent tool systems start from a callable function. That is enough when the action is small and low-risk. It is not enough when the agent is touching a real business process.

Agent Skills are becoming the standard mental model for giving agents capabilities: a `SKILL.md` plus scripts, references, and assets that agents discover and load on demand. SkillRun should be compatible with that layer rather than reinvent it.

SkillRun starts from a business capability:

```text
Skill Capsule = SOP + action code + schema + examples + permissions
Manifest      = compiled runtime contract
Core          = Rust Manifest-driven runtime
Adapter       = language bridge for user actions
Package       = .skr source + Manifest archive
```

A Skill Capsule carries what a function signature cannot:

- A `SKILL.md` SOP that tells the agent what the capability is for and when it must not run.
- Typed input and output schemas.
- Preflight checks for approval, policy, missing context, and recovery boundaries.
- Structured success and error envelopes.
- Artifacts recorded as first-class outputs.
- Run records with hashes, timing, logs, and evidence.
- Manifest-derived MCP exposure that does not re-import source code in Consumer Mode.

The relationship is:

```text
Agent Skills = how agents discover and learn a capability
MCP          = how agents invoke external capabilities
SkillRun    = how executable capabilities are checked, run, packaged, evidenced, and mounted
```

Use plain Agent Skills when the capability is mostly instructions, references, templates, or lightweight helper scripts. Use FastMCP when you only need to expose a function. Use SkillRun when SOP, code, schemas, preflight checks, run evidence, and consumer-side checks matter together.

See [Agent Skills Compatibility](docs/agent-skills-compatibility.md) for the full boundary.

## What Works Today

Current development line: `v0.6.2`.

Latest public release: `v0.6.2`.

Current binary/crate version:

```bash
skillrun --version
# skillrun 0.6.2
```

## Install

Starting with releases that publish native distribution artifacts, normal users do not need a Rust toolchain. macOS / Linux users can use the shell installer:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/iiwish/skillrun/releases/latest/download/skillrun-installer.sh | sh
skillrun --version
```

Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/iiwish/skillrun/releases/latest/download/skillrun-installer.ps1 | iex"
skillrun --version
```

GitHub Releases also include platform-named archives and checksums, such as `skillrun-x86_64-unknown-linux-gnu.tar.xz` and `sha256.sum`. See [Native Binary Distribution](docs/native-distribution.md) for artifact naming and package-manager boundaries.

Available today:

- Python `action.py` stable adapter target.
- JS `action.mjs` alpha adapter target.
- Level 0 `command` adapter for explicit argv processes that obey SkillRun IPC.
- Manifest generation with source hashes and runtime contract fields.
- `inspect`, `check`, and `doctor` human and JSON surfaces.
- `doctor` host diagnostics for declared Python, Node, command adapter, `PATH`, and package requirements; diagnostics do not use Docker, install dependencies, or execute action business logic.
- `validate` author loop for Manifest freshness, readiness diagnostics, the default example test, and the next explicit `run` step; it does not install dependencies or use Docker.
- `host status --json` for Desktop/tray host readiness and the `desktop.alpha` contract set.
- `test` and `run` with structured output/error envelopes.
- MCP stdio server from Manifest-derived tools and resources.
- `.skr` source + Manifest packaging.
- Local `.skr` import into the capsule registry:
  - `skillrun import <package.skr> --json`
  - `skillrun import <package.skr> --replace --json`
- Local capsule `registry` and `switchboard`.
- Local MCP Router for one-click mounting:
  - `skillrun router serve --mcp`
  - `skillrun router serve --mcp --dry-run`
- Reversible Claude Desktop MCP config mounting:
  - `skillrun mount plan --client <id> --json`
  - `skillrun mount apply --client claude-desktop --json`
  - `skillrun consumer mount apply --client claude-desktop --json`
  - `skillrun consumer mount rollback --client claude-desktop --backup <path> --json`
- Headless consumer JSON surfaces for Desktop, Router checks, and automation consumers:
  - `skillrun consumer inventory --json`
  - `skillrun consumer exposure --json`
  - `skillrun consumer runs list --json [--capsule <id>] [--status <status>] [--mode <mode>]`
  - `skillrun consumer runs inspect <run-id> --json`
  - `skillrun consumer mount plan --client <id> --json`

v0.5.15 freezes the Desktop alpha contract set and makes `import --json` runtime failures machine-readable. It intentionally does not add Desktop, Tauri, `skillrun ui`, a daemon API, Router hot reload, Router process management, Cursor apply, multi-client mount adapters, signed package trust, dependency installation, package update/reinstall, import from URL, marketplace behavior, `--include-input`, artifact content reads, log content reads, global run indexing, or OS sandboxing.

## Quickstart

Run the golden path from the repository root:

```bash
cargo run -- init refund --python --output tmp/quickstart
cargo run -- manifest --cwd tmp/quickstart/refund
cargo run -- inspect --cwd tmp/quickstart/refund
cargo run -- check --cwd tmp/quickstart/refund
cargo run -- doctor --cwd tmp/quickstart/refund
cargo run -- validate --cwd tmp/quickstart/refund
cargo run -- test --cwd tmp/quickstart/refund
cargo run -- run --cwd tmp/quickstart/refund --input examples/default.input.json
cargo run -- serve --mcp --cwd tmp/quickstart/refund --dry-run
cargo run -- pack --cwd tmp/quickstart/refund
```

For a real MCP stdio server:

```bash
cargo run -- serve --mcp --cwd tmp/quickstart/refund
```

`serve --mcp` is long-running stdio. Use `serve --mcp --dry-run` when you only want to inspect the derived MCP contract.

## v0.6 CLI Command Navigation

The v0.6 CLI information architecture is organized around four paths. Existing top-level commands stay stable; this pass freezes navigation and compatibility policy without changing CLI JSON contracts.

| Path | User question | Stable entrypoints | v0.6 compatibility policy |
| --- | --- | --- | --- |
| Author | How do I create, check, test, and package a Skill Capsule? | `init`, `manifest`, `inspect`, `check`, `doctor`, `validate`, `test`, `run`, `serve --mcp --dry-run`, `pack` | Keep these as top-level commands. `init --py` is the existing `init --python` alias; `init --js` remains an alpha adapter target. |
| Consumer | How do I import a `.skr` from someone else and decide whether to enable it? | `import`, `registry`, `switchboard`, `consumer inventory`, `consumer exposure` | `registry` is local inventory. `switchboard enabled=true` is local exposure intent, not proof of trust or sandboxing. |
| Router | How do I expose enabled capsules to an MCP client? | `router serve --mcp`, `router serve --mcp --dry-run`, `router status --json`, `mount plan/apply/rollback`, `consumer mount plan/apply/rollback` | MCP clients mount the SkillRun Router. `mount` is the short CLI facade; `consumer mount` remains the stable headless JSON surface. `serve --mcp` remains compatible as a single-capsule / author-debug entrypoint. |
| Ops | How do I inspect host readiness, diagnostics, mount previews, and run evidence? | `host status --json`, `router status --json`, `doctor`, `check`, `consumer mount plan --json`, `consumer runs list/inspect --json` | Headless JSON surfaces remain machine-readable; field additions must stay compatible with existing consumers. |

The five-minute core path:

```bash
# Author: create, generate the Manifest, check, and package
skillrun init refund --python --output tmp/quickstart
skillrun manifest --cwd tmp/quickstart/refund
skillrun check --cwd tmp/quickstart/refund
skillrun validate --cwd tmp/quickstart/refund
skillrun pack --cwd tmp/quickstart/refund

# Consumer: import the distribution artifact and enable local exposure intent
skillrun import <package.skr> --id refund --json
skillrun switchboard enable refund
skillrun consumer inventory --json
skillrun consumer exposure --json

# Router: preview or mount the MCP runtime entry
skillrun router serve --mcp --dry-run
skillrun router status --json
skillrun mount plan --client claude-desktop --json
skillrun mount apply --client claude-desktop --json
```

`.skr` is the distribution artifact: a source + Manifest archive. `import` validates it and places it in the local registry, but it is not the MCP runtime entry. MCP clients should mount `skillrun router serve --mcp`; the Router then exposes Manifest-derived tools for capsules enabled through `switchboard`.

Use `skillrun import <package.skr> --replace` to reinstall or update an already imported `.skr` with the same registry id. Replacement validates the new package before swapping files, preserves the existing `switchboard` enabled state, and is limited to capsules whose registry `source_type` is `imported_skr`; it does not overwrite local-path registry entries or install runtime dependencies.

Use `skillrun registry remove <id>` to remove a capsule from the local registry without deleting capsule files. If the entry came from `.skr` import, `skillrun registry remove <id> --delete-files` also deletes the imported copy after staging it for rollback while the registry is saved; this flag is refused for local-path entries.

`skillrun mount ...` is a short facade over `skillrun consumer mount ...`; JSON output keeps the existing `consumer.mount_*` schema versions.

Router short-running machine-readable contracts:

- `skillrun router serve --mcp --dry-run` emits `router.mcp.v1` with `ok`, `router.snapshot`, `tools`, `resources`, and `error.code` / `error.message` on failure.
- `skillrun router status --json` emits `router.status.v1` so Desktop / agents can inspect the route snapshot without starting the long-running MCP stdio server.
- JSON Schemas live at `docs/contracts/router-mcp.schema.json` and `docs/contracts/router-status.schema.json`.

This task does not introduce Desktop UI, marketplace behavior, daemon behavior, OS sandboxing, dependency installation, runtime images, or package-manager ownership. Future grouping commands or aliases must preserve the stable entrypoints above and leave a compatibility window for JSON consumers.

## Core Flow

```text
refund/
  SKILL.md
  action.py
  examples/
    default.input.json
  skillrun.config.json
  .skillrun/
    manifest.generated.yaml

        |
        | skillrun manifest
        v

Manifest-driven contract
  schemas
  permissions
  adapter
  tool description
  source hashes

        |
        +-- inspect / check / doctor
        +-- validate
        +-- import / registry / switchboard
        +-- consumer inventory / exposure / runs / mount plan
        +-- test / run
        +-- serve --mcp
        +-- pack
```

Author Mode can regenerate the Manifest from local source. Consumer Mode reads the Manifest, validates source hashes and runtime contract fields, and fails closed when the Manifest is missing, stale, or invalid.

## Trust Model

SkillRun is honest about the boundary.

"No Manifest contract, no execution" means SkillRun requires Manifest contracts, input/output schemas, preflight checks, structured envelopes, artifact containment, run evidence, and Consumer Mode static checks before it treats a capsule as runnable or exposable.

It does not mean:

- OS-level sandboxing.
- Network egress isolation.
- Dependency installation.
- Signed package trust.
- Reproducible runtime images.
- Safe execution of arbitrary third-party code.

Running a third-party action still means executing third-party code. SkillRun reduces blind agent execution by making the SOP, runtime contract, evidence, and failure behavior explicit.

Important rules:

- `stdout` and `stderr` are logs only. Structured results must come from output/error envelopes.
- Consumer Mode does not dynamically import untrusted source code for metadata extraction.
- Stale or missing Manifests fail closed.
- `.skr` is a source + Manifest archive, not a secure install format.
- `registry` is local inventory, not a trust store.
- `switchboard enabled=true` is future exposure intent, not proof of trust or sandboxing.

## Desktop Direction

Desktop is a separate tray-first project. It should consume SkillRun Core through stable headless surfaces, not by reading `.skillrun/` internals or parsing MCP text.

The intended boundary is:

```text
skillrun
  Rust CLI/Core, Manifest, Consumer Mode, Adapter Protocol, runtime, pack,
  registry/switchboard, headless JSON surfaces, Router MVP

skillrun-desktop
  Tauri tray shell, Capsule Switchboard, MCP Mount Manager,
  Envelope Explorer, official pack browser
```

The key rule for one-click mounting is: mount the SkillRun Router, not individual `.skr` files or capsule folders. `.skr` is an import/distribution artifact. Router is the MCP runtime entry.

The tray should use `skillrun host status --json` for Core handshake and short-running JSON commands for refresh. It should bind to `desktop.alpha` contract version `1`, not infer compatibility from human text. It should not run `skillrun router serve --mcp` as a hidden daemon; MCP clients start the Router through their mounted config.

## Version Layers

SkillRun uses separate version layers:

- `Cargo.toml` and `skillrun --version` identify the binary/crate version.
- Git tags such as `v0.5.14` identify public release boundaries.
- Milestone names such as v0.5.4, v0.5.5, v0.5.6, v0.5.7, v0.5.8, v0.5.9, v0.5.10, v0.5.11, v0.5.12, v0.5.13, v0.5.14, and v0.5.15 describe delivery scope.
- Manifest `manifest_version` identifies the Manifest IR schema.
- IPC / Adapter `protocol_version` identifies the Core-to-adapter file protocol.

The current generated Manifest IR and IPC protocol versions remain `0.1.0`. v0.5.15 freezes the Desktop alpha contract set without changing those protocol versions.

## Roadmap

| Milestone | Focus |
| --- | --- |
| `v0.2` | Real MCP stdio server and public release candidate readiness |
| `v0.3` | JS Action Alpha via `action.mjs` and explicit TypeScript boundary |
| `v0.4` | Portable Consumer Checks and dependency-aware Consumer Mode |
| `v0.5` | Language-agnostic Adapter Protocol and Level 0 command adapter |
| `v0.5.4` | Core Stabilization Audit before Desktop |
| `v0.5.5` | Manifest-driven Consumer Mode contract hardening |
| `v0.5.6` | Headless consumer JSON contracts before Desktop |
| `v0.5.7` | Public narrative and contract-surface polish before Desktop |
| `v0.5.8` | Router runtime MVP for real one-click mounting |
| `v0.5.9` | Safe Mount Apply for reversible MCP client config changes |
| `v0.5.10` | Consumer Contract Hardening before Desktop |
| `v0.5.11` | Runs Inspect for Desktop Envelope Explorer |
| `v0.5.12` | Capsule Import for Desktop-ready local inventory |
| `v0.5.13` | Import-to-Router contract hardening before Desktop |
| `v0.5.14` | Desktop Host Readiness for tray-first Core handshake |
| `v0.5.15` | Desktop Contract Freeze for `desktop.alpha` and import JSON errors |
| `v0.6` | Proposed Consumer Era Desktop and local control plane |

## Examples

The runnable examples are intentionally narrow. They prove SkillRun boundaries without turning the project into a general API wrapper.

- `examples/refund`: refund decision with policy limits, approval boundaries, typed inputs, structured `PolicyViolation`, artifacts, run records, MCP exposure, and `.skr` packaging.
- `examples/wecom_team_notice`: local notification workflow with dry-run preview, approval boundary, declared `WECOM_WEBHOOK_URL`, structured `DependencyError`, and markdown artifacts.
- `examples/commit_message_gate`: Conventional Commits validation without auto-staging files.
- `examples/bounded_file_patcher`: exact text replacement inside declared directories with patch artifacts.
- `examples/readonly_diagnostics_runner`: named allowlist diagnostics without arbitrary shell strings.
- `examples/command_hello`: Level 0 command adapter contract without a SkillRun SDK.

Docs-level business patterns remain part of the narrative without expanding current runtime scope: Support Triage, Access Request Approval, and Vendor Risk Review show how a portable Agent skill can carry stable routing labels, approval boundaries, and artifact-backed review evidence.

## Documentation

- [Documentation index](docs/README.md)
- [Architecture SSOT](docs/ssot.md)
- [Positioning](docs/positioning.md)
- [Trust model](docs/trust-model.md)
- [Adapter Protocol](docs/adapter-protocol.md)
- [v0.5.6 Headless Consumer Contract](docs/v0.5.6-headless-consumer-contract.md)
- [v0.5.6 Run History Contract Review](docs/v0.5.6-run-history-contract-review.md)
- [v0.5.6 Mount Plan Contract Review](docs/v0.5.6-mount-plan-contract-review.md)
- [v0.5.8 Router MVP](docs/v0.5.8-router-mvp.md)
- [v0.5.9 Safe Mount Apply](docs/v0.5.9-safe-mount-apply.md)
- [v0.5.10 Consumer Contract Hardening](docs/v0.5.10-consumer-contract-hardening.md)
- [v0.5.11 Runs Inspect](docs/v0.5.11-runs-inspect.md)
- [v0.5.12 Capsule Import](docs/v0.5.12-capsule-import.md)
- [v0.5.13 Import Router Contract](docs/v0.5.13-import-router-contract.md)
- [v0.5.14 Desktop Host Readiness](docs/v0.5.14-desktop-host-readiness.md)
- [v0.5.15 Desktop Contract Freeze](docs/v0.5.15-desktop-contract-freeze.md)
- [v0.6 Consumer Era vision](docs/v0.6-consumer-era-vision.md)
- [v0.6 Skill Capsule Contract](docs/v0.6-skill-capsule-contract.md)
- [Router MCP JSON Schema](docs/contracts/router-mcp.schema.json)
- [Router Status JSON Schema](docs/contracts/router-status.schema.json)
- [Business examples](docs/business-examples.md)
- [Test strategy](docs/testing.md)
- [Release policy](docs/release-policy.md)
- [Release checklist](docs/release-checklist.md)
- [Contributing guide](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

Project governance documents are primarily written in Chinese so future agents can parse and maintain the approved product contract consistently.

## Contributing

SkillRun is intentionally narrow. Contributions should preserve these rules:

- Use `SkillRun` for the project name and `skillrun` for the CLI, crate, commands, and code identifiers.
- Keep SkillRun core behavior in Rust.
- Treat Python as the stable action adapter target and JS `action.mjs` as a narrow alpha path.
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
