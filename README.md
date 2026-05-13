# SkillRun

> Turn one SOP and one action into a manifest-driven Agent skill capsule.

[简体中文](README.zh-CN.md)

FastMCP turns functions into MCP tools.
SkillRun turns SOP-backed capabilities into **Skill Capsules**.

A Skill Capsule carries what a function signature cannot: typed input/output, preflight checks, structured errors, artifacts, run evidence, declared permissions, and a Manifest-derived MCP contract.

SkillRun is for teams that need the business context, recovery rules, audit trail, and runtime contract to travel with the action. Use FastMCP when you only need to expose a function; use SkillRun when the SOP matters as much as the code.

## Status

SkillRun v0.3.0 is the current local release handoff. The `v0.3.0` tag is local; no remote tag or public package has been published by this handoff.

- Current implementation: v0.2 MCP stdio behavior plus v0.3 JS Action Alpha and adapter-aware diagnostics.
- Available today: `skillrun --help`, `skillrun --version`, `skillrun init <name> --python`, `skillrun init <name> --py`, `skillrun init <name> --js`, `skillrun manifest --cwd <capsule>`, `skillrun inspect --cwd <capsule>`, `skillrun doctor --cwd <capsule>`, `skillrun test --cwd <capsule>`, `skillrun run --cwd <capsule> --input <file>`, `skillrun serve --mcp --cwd <capsule>`, `skillrun serve --mcp --cwd <capsule> --dry-run`, `skillrun pack --cwd <capsule>`, structured error envelopes, artifact validation, declared env injection, stale Manifest guards, instruction-only guards, Manifest-derived MCP tools/resources, `.skr` package generation, and release tests for the skeleton/init/manifest/inspect/doctor/runtime/error/artifact/permission/consumer-guard/MCP/pack paths.
- v0.2 keeps `serve --mcp --dry-run` for contract inspection, but the normal `serve --mcp` path is now a long-running MCP stdio server.
- The SkillRun core, CLI, Manifest, IPC, MCP exposure, and packaging path are implemented in Rust.
- Python `action.py` is the stable action adapter target. JS `action.mjs` is an alpha adapter target. Both are user action languages, not the SkillRun implementation language.

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
        +-- skillrun test
        +-- skillrun run --input examples/default.input.json
        +-- skillrun serve --mcp             # MCP stdio server
        +-- skillrun serve --mcp --dry-run   # contract inspection
        +-- skillrun pack
```

The generated Manifest is the runtime contract. Author mode can regenerate it from local sources. Consumer mode reads it, validates source hashes, and refuses to guess when the Manifest is missing or stale.

In v0.2, `skillrun serve --mcp` starts a real MCP stdio server whose tools and resources are still derived from the Manifest.

## Release Candidate Workflow

```bash
skillrun init refund --python
cd refund
# edit SKILL.md
# edit action.py
skillrun manifest
skillrun inspect
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp
skillrun pack
```

`--py` is only a short alias for `--python`. Keep `--python` as the main Quickstart because Python is the stable path.

Language flags belong to `init` only. `manifest`, `inspect`, `doctor`, `test`, `run`, `serve --mcp`, and `pack` read the capsule and its generated Manifest; they do not accept `--python`, `--py`, or `--js`.

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
skillrun doctor
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp --dry-run
skillrun pack
```

The JS alpha contract is canonical ESM `action.mjs` with explicit `inputSchema`, `outputSchema`, optional `preflight`, and `run` exports. SkillRun does not infer schemas from TypeScript types, JSDoc, Zod, TypeBox, examples, or package metadata in v0.3.

`action.ts` is not a runtime entrypoint. Authors may compile TypeScript to `action.mjs` themselves, but SkillRun v0.3 does not run `ts-node`, `tsx`, source maps, CJS/ESM compatibility matrices, or package-manager install flows.

## Let an Agent Learn a Capsule Before Calling It

SkillRun capsules are designed to be learned before they are called. Give an AI assistant a URL or repository path that points directly to a capsule folder, not just a generic project homepage. The folder should include `SKILL.md`, `skillrun.config.json`, an action entrypoint such as `action.py` or `action.mjs`, and `examples/`.

```text
Learn this SkillRun Capsule before using it:
<capsule-folder-url-or-repo-path>

1. Read SKILL.md for purpose, SOP, prohibited behavior, required context, and recovery guidance.
2. Read skillrun.config.json and the generated Manifest, if present, to confirm adapter and entrypoint.
3. Read action.py or action.mjs only as the action contract for this capsule; do not infer unsupported languages or package-manager behavior.
4. Read examples/default.input.json to understand the expected input shape.
5. If you can access the workspace, run `skillrun inspect --cwd <capsule>`, `skillrun doctor --cwd <capsule>`, and `skillrun test --cwd <capsule>`.
6. When calling the MCP tool, do not infer success from stdout. Use the output/error envelope, artifacts, and run record.
```

Use a real capsule folder when publishing a skill. This keeps the model from treating the capsule as a loose function call: it learns the SOP, adapter entrypoint, example input, and failure behavior before it uses the MCP tool.

## What Works Today

The repository currently contains the Rust CLI, `init --python` and `init --py` Python capsule generator, `init --js` JS alpha capsule generator, Manifest generator, inspect renderer, doctor diagnostics, test/run path, MCP stdio server, MCP dry-run contract renderer, `.skr` package generation, and the B001 `refund` hero example:

```bash
cargo test
cargo run -- --help
cargo run -- --version
cargo run -- init refund --python --output tmp/e2e-init
cargo run -- manifest --cwd tmp/e2e-init/refund
cargo run -- inspect --cwd tmp/e2e-init/refund
cargo run -- doctor --cwd tmp/e2e-init/refund
cargo run -- test --cwd tmp/e2e-init/refund
cargo run -- run --cwd tmp/e2e-init/refund --input examples/default.input.json
cargo run -- serve --mcp --cwd tmp/e2e-init/refund --dry-run
cargo run -- pack --cwd tmp/e2e-init/refund
```

Example output:

```text
skillrun 0.3.0
```

The real `serve --mcp` command is a long-running stdio server and is validated by the scripted MCP client release matrix.

The `.skr` package is a source/Manifest archive. It is not signed, does not vendor dependencies, and does not provide a reproducible runtime image.

## Release Candidate Limits

v0.3.0 is intentionally narrow:

- MCP transport is stdio only.
- Each capsule exposes one primary Manifest-derived tool.
- Python `action.py` is the stable action adapter target.
- JS `action.mjs` is alpha only and is not full TypeScript support.
- `action.ts`, direct TypeScript runtime execution, `ts-node`, `tsx`, type-to-schema extraction, source maps, CJS compatibility, npm install flows, and dependency vendoring are out of scope.
- `.skr` is a source + Manifest archive, not a signed package, registry package, dependency bundle, or runtime image.
- SkillRun does not provide an OS sandbox. Running a third-party action still means executing third-party code.
- The `v0.3.0` tag is local in this handoff. Remote tag push and package publication are separate explicit decisions.

## Security Model

SkillRun is honest about trust boundaries:

- `stdout` and `stderr` are logs only. Structured results must come from output or error envelopes.
- Consumer mode must not dynamically import untrusted source code to extract metadata.
- Stale or missing Manifests fail closed.
- Declared environment variables and artifact paths are part of the runtime contract.
- SkillRun does not claim to be a full OS sandbox. Running a third-party action still means executing third-party code.
- `.skr` is not a secure install format, registry package, or dependency bundle.

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

## Classic Business Examples

SkillRun's business proof is intentionally narrow:

- `B001: Refund Decision` is implemented in `examples/refund` and tested end-to-end with success, `PolicyViolation`, `ValidationError`, run records, MCP dry-run exposure, and `.skr` packaging.
- `B002: Support Triage` is a docs-level example showing stable routing labels and missing-context recovery.
- `B003: Access Request Approval` is a docs-level example showing approval boundaries, declared environment, and audit notes.
- `B004: Vendor Risk Review` is a docs-level example showing artifact-first review summaries and package distribution without dependency vendoring.

The current runtime only implements the refund capsule. The other examples explain where the same SOP + action + Manifest pattern is valuable without expanding the runtime scope.

## Documentation

- [MVP contract](docs/mvp.md)
- [Architecture SSOT](docs/ssot.md)
- [Business examples](docs/business-examples.md)
- [Test strategy](.ai-platform/docs/test-strategy.md)

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
