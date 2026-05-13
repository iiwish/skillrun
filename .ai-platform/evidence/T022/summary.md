# T022 Evidence Summary

Task: T022 Implement Node Metadata Extraction From Explicit JSON Schema
Status: Accepted
Date: 2026-05-13

## Scope

Implemented JS alpha metadata extraction for Manifest generation while preserving Python behavior and keeping JS runtime execution out of scope.

## Changed Files

- `src/adapters/mod.rs`
- `src/adapters/node.rs`
- `src/manifest.rs`
- `src/consumer.rs`
- `tests/manifest.rs`
- `tests/consumer_guards.rs`

## What Changed

- Added a `node` metadata adapter that imports `action.mjs` and reads explicit `inputSchema` / `outputSchema` exports.
- Validated JS schema exports as JSON Schema objects and failed clearly when exports are missing or not objects.
- Kept runtime dispatch unsupported for `node`; T023 owns JS execution.
- Added config-first Manifest behavior: `skillrun.config.json` runtime settings override action-file convention.
- Added no-config convention for `action.py` and `action.mjs`.
- Failed closed for ambiguous known action files when no config exists.
- Rejected `action.ts` with explicit guidance to compile to `action.mjs`.
- Updated Consumer Mode missing-Manifest messaging so JS capsules are not misreported as missing `action.py`.
- Added static stale-source guard coverage for `action.mjs`.

## Validation

- RED: `cargo test --test manifest --test consumer_guards` failed before implementation with `unsupported metadata adapter: node`.
- `cargo fmt`: passed.
- GREEN: `cargo test --test manifest --test consumer_guards` passed.
- Full suite: `cargo test` passed.

## Review Notes

- No TypeScript inference, Zod/TypeBox/JSDoc inference, package manager behavior, dependency vendoring, sandbox, registry, HTTP transport, MCP behavior, pack behavior, or JS runtime execution was added.
- Author Mode still dynamically imports source for metadata. This is not a sandbox and should not be described as one.
- Consumer Mode remains static and Manifest/hash based.

## Residual Risk

- Node metadata extraction depends on a local `node` binary. Missing Node fails clearly, but v0.3 still needs release docs to state that JS alpha requires Node for Author Mode.
- JS runtime remains unsupported until T023.

## Review Decision

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
