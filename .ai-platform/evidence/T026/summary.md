# T026 Evidence Summary

Task: T026 Implement Adapter-aware doctor Or validate
Status: Accepted
Date: 2026-05-13

## Scope

Added `skillrun doctor [--cwd <dir>]` as a non-executing diagnostics command for capsule structure, Manifest freshness and adapter-specific recovery steps.

## Changed Files

- `src/doctor.rs`
- `src/cli.rs`
- `src/main.rs`
- `tests/cli.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`

## What Changed

- Added a `doctor` command and help text.
- Implemented read-only diagnostics for required files, Manifest presence, Manifest source hash freshness, runtime adapter/entrypoint and example presence.
- Added adapter-aware output for Python stable capsules and JS alpha capsules.
- Added unsupported TypeScript diagnostics that tell authors to compile to `action.mjs`.
- Added instruction-only diagnostics that explicitly refuse action inference from Markdown, scripts, references, assets or examples.
- Added a JS no-import guard proving `doctor` does not import `action.mjs` for metadata.
- Kept language flags scoped to `init`; `doctor` output does not suggest `--python`, `--py` or `--js`.

## Validation

- RED: `cargo test --test consumer_guards --test instruction_only --test cli` failed before implementation because help did not list `doctor`.
- `cargo fmt`: passed.
- GREEN: `cargo test --test consumer_guards --test instruction_only --test cli`: passed.
- Full suite: `cargo test`: passed.

## Review Notes

- T026 stays inside its allowed files.
- No adapter implementation, runtime dispatch, package manager, TypeScript runtime, sandbox, registry or HTTP transport behavior was added.
- `doctor` reads files and hashes only; it does not execute or import action source.

## Residual Risk

`doctor` is intentionally textual in v0.3. A structured JSON diagnostics mode could be considered later, but is outside this task.

## Review State

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
