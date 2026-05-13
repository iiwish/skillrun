# T024 Evidence Summary

Task: T024 Add JS Alpha End-to-end Command Matrix
Status: Accepted
Date: 2026-05-13

## Scope

Added JS alpha end-to-end coverage for the complete local command chain and tightened `inspect` wording around Manifest-derived runtime adapter and entrypoint.

## Changed Files

- `tests/e2e_matrix.rs`
- `tests/inspect.rs`
- `src/inspect.rs`

## What Changed

- Added a JS alpha command matrix covering `init --js -> manifest -> inspect -> test -> run`.
- Verified JS init generates `action.mjs`, default example, config and no package manager files.
- Verified JS Manifest records `runtime.adapter: node`, `runtime.entrypoint: action.mjs` and `sources.action.path: action.mjs`.
- Verified JS inspect reports the Manifest-derived adapter/entrypoint runtime contract and detects JS preflight.
- Verified JS `test` and `run` return successful output envelopes.
- Verified stale Manifest behavior fails closed for changed `action.mjs`.
- Added a `--py -> manifest` smoke test proving the alias keeps Python adapter identity.
- Kept MCP and pack behavior out of scope for T024.

## Validation

- RED: `cargo test --test e2e_matrix --test inspect --test runtime --test manifest` failed before implementation because `inspect` did not yet include the Manifest adapter/entrypoint contract wording.
- `cargo fmt`: passed.
- GREEN: `cargo test --test e2e_matrix --test inspect --test runtime --test manifest` passed.
- Full suite: `cargo test` passed.

## Review Notes

- T024 stays within local command-chain coverage and does not change runtime, manifest generation, MCP, pack, package manager, TypeScript, sandbox, registry or HTTP transport behavior.
- The implementation change is limited to `inspect` rendering and preflight detection.

## Residual Risk

- JS alpha E2E assumes a local `node` binary is available. Missing Node behavior is already covered by T022 metadata tests.

## Review Decision

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
