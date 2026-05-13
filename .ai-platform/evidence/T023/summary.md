# T023 Evidence Summary

Task: T023 Implement Node Runtime Adapter
Status: Accepted
Date: 2026-05-13

## Scope

Implemented the JS alpha runtime adapter for `action.mjs` while preserving Python behavior and existing Rust-side run evidence.

## Changed Files

- `src/adapters/mod.rs`
- `src/adapters/node.rs`
- `tests/runtime.rs`
- `tests/errors.rs`
- `tests/artifacts.rs`

## What Changed

- Enabled `node` as a runtime adapter in adapter dispatch.
- Added Node runtime execution for `action.mjs`.
- Passed the same runtime IPC paths to Node actions: context JSON, input JSON, output JSON and artifact directory.
- Supported async and sync `run(input, ctx)`.
- Supported optional `preflight(input, ctx)`.
- Validated JS input against explicit `inputSchema` and returned recoverable `ValidationError`.
- Validated JS output against explicit `outputSchema` and returned non-recoverable `ProtocolViolation` for malformed output.
- Mapped preflight and explicit business rejections to recoverable `PolicyViolation`.
- Mapped unexpected thrown errors to non-recoverable `RuntimeError` while keeping stack traces in stderr logs rather than display markdown.
- Kept stdout and stderr as logs only.
- Kept artifact validation in Rust via existing `permissions::validate_artifacts`.
- Did not add package-manager installation, dependency vendoring, TypeScript support, sandbox behavior, registry behavior, HTTP transport, MCP behavior or pack behavior.

## Validation

- RED: `cargo test --test runtime --test errors --test artifacts` failed before implementation because JS runtime output was not a JSON envelope.
- `cargo fmt`: passed.
- GREEN: `cargo test --test runtime --test errors --test artifacts` passed.
- Full suite: `cargo test` passed.

## Review Notes

- The JS runtime adapter intentionally uses only a narrow in-adapter JSON Schema subset for v0.3 alpha validation.
- The Rust runtime remains responsible for run directories, records, source hashes, stdout/stderr logs and artifact permission validation.
- Unknown runtime adapters still fail before run records are created.

## Residual Risk

- JS runtime depends on a local `node` binary.
- The JSON Schema validator is intentionally narrow for v0.3 alpha and is not a full JSON Schema implementation.

## Review Decision

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
