# SkillRun

> Turn one SOP and one action into a tested MCP skill package with a Rust CLI/Core.

[简体中文](README.zh-CN.md)

SkillRun is a Rust-first local CLI/Core for building **SOP-backed Skill Capsules**. It turns a human-readable operating procedure, an explicit action, schemas, examples, and permissions into a manifest-driven skill package that can be inspected, tested, run, exposed to MCP clients, and distributed.

SkillRun is not another "wrap a function as a tool" layer. It is for teams that need the business context, recovery rules, audit trail, and runtime contract to travel with the action.

## Status

SkillRun is in the v0.1.0 MVP buildout.

- Current implementation: v0.1 MVP behavior through `.skr` packaging, with release-level T011 validation ready for review.
- Available today: `skillrun --help`, `skillrun --version`, `skillrun init <name> --python`, `skillrun manifest --cwd <capsule>`, `skillrun inspect --cwd <capsule>`, `skillrun test --cwd <capsule>`, `skillrun run --cwd <capsule> --input <file>`, `skillrun serve --mcp --cwd <capsule> --dry-run`, `skillrun pack --cwd <capsule>`, structured error envelopes, artifact validation, declared env injection, stale Manifest guards, instruction-only guards, Manifest-derived MCP contract inspection, `.skr` package generation, and contract tests for the skeleton/init/manifest/inspect/runtime/error/artifact/permission/consumer-guard/MCP/pack paths.
- Long-running `serve --mcp` server mode is not implemented in v0.1; `serve --mcp --dry-run` verifies the Manifest-derived MCP contract.
- The SkillRun core, CLI, Manifest, IPC, MCP exposure, and packaging path are implemented in Rust.
- Python `action.py` is the first planned action adapter target. It is the user action language, not the SkillRun implementation language.

## Why SkillRun

Most agent tool systems start with a callable function. SkillRun starts with a business capability:

```text
Skill Capsule = SOP + action code + schema + examples + permissions
Manifest      = compiled runtime contract
Core          = Rust manifest-driven runtime and MCP server
Adapter       = language bridge for user actions
Package       = immutable .skr distribution artifact
```

Use SkillRun when you want an agent-callable capability to carry more than a function signature:

- A `SKILL.md` cognitive contract that explains the SOP.
- Typed input and output schemas.
- Preflight checks for policy and approval boundaries.
- Structured success and error envelopes.
- Artifacts that are recorded as first-class outputs.
- Run records that preserve hashes, logs, and evidence.
- Manifest-driven MCP exposure that does not re-import source code in consumer mode.

If you only need to expose a Python function as an MCP tool, a lighter tool such as FastMCP may be the better fit. SkillRun is designed for SOP-backed capabilities that must be inspectable, testable, and distributable.

## The Core Flow

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
  schema
  permissions
  adapter
  tool description
  source hashes

        |
        +-- skillrun inspect
        +-- skillrun test
        +-- skillrun run --input examples/default.input.json
        +-- skillrun serve --mcp --dry-run
        +-- skillrun pack
```

The generated Manifest is the runtime contract. Author mode can regenerate it from local sources. Consumer mode reads it, validates source hashes, and refuses to guess when the Manifest is missing or stale.

## Planned MVP Workflow

```bash
skillrun init refund --python
cd refund
# edit SKILL.md
# edit action.py
skillrun manifest
skillrun inspect
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp --dry-run
skillrun pack
```

The first hero example is `refund`: a refund decision capsule with policy limits, approval boundaries, typed inputs, structured `PolicyViolation` errors, and auditable run records.

## What Works Today

The repository currently contains the Rust CLI skeleton, `init --python` capsule generator, Manifest generator, inspect renderer, test/run success path, MCP dry-run contract renderer, `.skr` package generation, and the B001 `refund` hero example:

```bash
cargo test
cargo run -- --help
cargo run -- --version
cargo run -- init refund --python --output tmp/e2e-init
cargo run -- manifest --cwd tmp/e2e-init/refund
cargo run -- inspect --cwd tmp/e2e-init/refund
cargo run -- test --cwd tmp/e2e-init/refund
cargo run -- run --cwd tmp/e2e-init/refund --input examples/default.input.json
cargo run -- serve --mcp --cwd tmp/e2e-init/refund --dry-run
cargo run -- pack --cwd tmp/e2e-init/refund
```

Example output:

```text
skillrun 0.1.0
```

Long-running MCP server mode intentionally fails with `command not implemented yet`; the v0.1 MCP path is dry-run contract exposure.

The `.skr` package is a source/Manifest distribution archive. It does not vendor dependencies or provide a reproducible runtime image.

## Security Model

SkillRun is honest about trust boundaries:

- `stdout` and `stderr` are logs only. Structured results must come from output or error envelopes.
- Consumer mode must not dynamically import untrusted source code to extract metadata.
- Stale or missing Manifests fail closed.
- Declared environment variables and artifact paths are part of the runtime contract.
- v0.1 does not claim to be a full OS sandbox. Running a third-party action still means executing third-party code.

The MVP goal is a small, hard boundary: no implicit execution of instruction-only skills, no stdout success fallback, and no source-code metadata import in consumer mode.

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

## Classic Business Examples

SkillRun's v0.1 business proof is intentionally narrow:

- `B001: Refund Decision` is implemented in `examples/refund` and tested end-to-end with success, `PolicyViolation`, `ValidationError`, run records, MCP dry-run exposure, and `.skr` packaging.
- `B002: Support Triage` is a docs-level example showing stable routing labels and missing-context recovery.
- `B003: Access Request Approval` is a docs-level example showing approval boundaries, declared environment, and audit notes.
- `B004: Vendor Risk Review` is a docs-level example showing artifact-first review summaries and package distribution without dependency vendoring.

The v0.1 MVP only implements the refund capsule. The other examples explain where the same SOP + action + Manifest pattern is valuable without expanding the runtime scope.

## Documentation

- [MVP contract](docs/mvp.md)
- [Architecture SSOT](docs/ssot.md)
- [Business examples](docs/business-examples.md)
- [Test strategy](.ai-platform/docs/test-strategy.md)

Project governance documents are primarily written in Chinese so future agents can parse and maintain the approved product contract consistently.

## Contributing

SkillRun is intentionally narrow at v0.1. Contributions should preserve these project rules:

- Use `SkillRun` for the project name and `skillrun` for the CLI, crate, commands, and code identifiers.
- Keep SkillRun core behavior in Rust.
- Treat Python as the first action adapter target only.
- Do not execute instruction-only skills implicitly.
- Do not infer structured success from stdout.
- Keep README and docs clear about what is implemented now versus planned.

Run the baseline checks before submitting changes:

```bash
cargo test
```

## License

SkillRun is licensed under the [Apache License, Version 2.0](LICENSE).
