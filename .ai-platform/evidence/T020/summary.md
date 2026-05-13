# T020 Evidence Summary

Task: T020 Introduce Adapter Dispatch Boundary
Status: Accepted
Date: 2026-05-13

## Scope

Introduced the first adapter dispatch boundary while preserving the existing Python path.

## Changed Files

- `src/adapters/mod.rs`
- `src/adapters/python.rs`
- `src/main.rs`
- `src/manifest.rs`
- `src/runtime.rs`

## What Changed

- Added `src/adapters/mod.rs` as the adapter registry and shared adapter contract.
- Moved `ActionRunRequest` and `ActionRunOutput` out of `python.rs` into the adapter boundary.
- Updated `manifest` generation to call `adapters::extract_schemas`.
- Updated runtime execution to validate and dispatch through `adapters::ensure_runtime_adapter` / `adapters::run_action`.
- Kept Python behavior as the only implemented adapter.

## Validation

- `cargo fmt`: passed.
- `cargo test --test manifest --test runtime --test e2e_matrix`: passed.
- `cargo test`: passed.

## Review Notes

- No JS metadata or runtime implementation was added.
- Unsupported adapter behavior remains deterministic and still fails before run records are created.
- Consumer, MCP and pack surfaces remain Manifest-driven.

## Residual Risk

- `manifest` now reads runtime config before schema extraction, which is the intended adapter-boundary direction. Follow-up T022 must harden config-first convention and ambiguous action behavior.

## Review Decision

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
