# SkillRun

> SOP-backed skills for AI agents. No Manifest contract, no execution.

[Simplified Chinese](README.zh-CN.md)

FastMCP turns functions into MCP tools. SkillRun turns SOP-backed capabilities into **Skill Capsules**.

SkillRun is a Rust runtime and CLI for packaging one SOP and one action into an inspectable, testable, runnable, distributable, and MCP-callable Agent skill. It is not a general Agent framework, not a marketplace, and not an OS sandbox.

## Why It Exists

Most agent tool systems start from a callable function. That is enough when the action is small and low-risk. It is not enough when the agent is touching a real business process.

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

Use FastMCP when you only need to expose a function. Use SkillRun when the SOP matters as much as the code.

## What Works Today

Current public release: `v0.5.8`.

Current binary/crate version:

```bash
skillrun --version
# skillrun 0.5.8
```

Available today:

- Python `action.py` stable adapter target.
- JS `action.mjs` alpha adapter target.
- Level 0 `command` adapter for explicit argv processes that obey SkillRun IPC.
- Manifest generation with source hashes and runtime contract fields.
- `inspect`, `check`, and `doctor` human and JSON surfaces.
- `test` and `run` with structured output/error envelopes.
- MCP stdio server from Manifest-derived tools and resources.
- `.skr` source + Manifest packaging.
- Local capsule `registry` and `switchboard`.
- Local MCP Router for one-click mounting:
  - `skillrun router serve --mcp`
  - `skillrun router serve --mcp --dry-run`
- Headless consumer JSON surfaces for future Desktop and Router consumers:
  - `skillrun consumer inventory --json`
  - `skillrun consumer exposure --json`
  - `skillrun consumer runs list --json`
  - `skillrun consumer mount plan --client <id> --json`

v0.5.8 intentionally does not add Desktop, Tauri, `skillrun ui`, a daemon API, real MCP client config mutation, signed package trust, dependency installation, marketplace behavior, or OS sandboxing.

## Quickstart

Run the golden path from the repository root:

```bash
cargo run -- init refund --python --output tmp/quickstart
cargo run -- manifest --cwd tmp/quickstart/refund
cargo run -- inspect --cwd tmp/quickstart/refund
cargo run -- check --cwd tmp/quickstart/refund
cargo run -- doctor --cwd tmp/quickstart/refund
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
        +-- registry / switchboard
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

Desktop is a separate project. It should consume SkillRun Core through stable headless surfaces, not by reading `.skillrun/` internals or parsing MCP text.

The intended boundary is:

```text
skillrun
  Rust CLI/Core, Manifest, Consumer Mode, Adapter Protocol, runtime, pack,
  registry/switchboard, headless JSON surfaces, Router MVP

skillrun-desktop
  Tauri shell, Capsule Switchboard, MCP Mount Manager, Envelope Explorer,
  official pack browser
```

The key rule for one-click mounting is: mount the future SkillRun Router, not individual `.skr` files or capsule folders. `.skr` is an import/distribution artifact. Router is the MCP runtime entry.

## Version Layers

SkillRun uses separate version layers:

- `Cargo.toml` and `skillrun --version` identify the binary/crate version.
- Git tags such as `v0.5.8` identify public release boundaries.
- Milestone names such as v0.5.4, v0.5.5, v0.5.6, v0.5.7, and v0.5.8 describe delivery scope.
- Manifest `manifest_version` identifies the Manifest IR schema.
- IPC / Adapter `protocol_version` identifies the Core-to-adapter file protocol.

The current generated Manifest IR and IPC protocol versions remain `0.1.0`. v0.5.8 adds a local MCP Router runtime MVP without changing those protocol versions.

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
- [v0.6 Consumer Era vision](docs/v0.6-consumer-era-vision.md)
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
