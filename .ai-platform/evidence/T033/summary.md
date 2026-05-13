# T033 Evidence Summary

Task ID: T033
Executor: Codex direct execute fallback
Branch: `codex/v0.4-integration`

## Files Changed

- `src/runtime.rs`
- `src/errors.rs`
- `tests/runtime.rs`
- `.ai-platform/specs/v0.4/tasks.md`
- `.ai-platform/evidence/T033/summary.md`
- `.ai-platform/evidence/T033/test-results.md`
- `.ai-platform/evidence/T033/review.md`
- `.ai-platform/evidence/T033/diff.patch`

## Commands Run

- RED: `cargo test --test runtime --test errors` failed because runtime dependency failures still returned `RuntimeError`.
- GREEN: `cargo test --test runtime --test errors` passed after wiring readiness dependency failures into `DependencyError`.
- Format: `cargo fmt --check` passed.
- Full validation: `cargo test` passed.
- Lint: `cargo clippy --all-targets -- -D warnings` passed.

## TDD Evidence

RED added runtime tests for:

- Missing Python returns `DependencyError`.
- Missing Node returns `DependencyError`.
- Missing Pydantic returns `DependencyError` without importing action source.
- Stale Manifest fails before dependency precheck.
- Dependency failures still create failed run records.

GREEN added a runtime dependency precheck after Manifest validation and run path creation, returning a structured `DependencyError` envelope through the existing `finish_run` path.

## Diff Summary

- Added `errors::dependency_error`.
- Added runtime dependency precheck using the readiness model.
- Preserved stale Manifest priority by running the precheck only after `consumer::validate`.
- Preserved audit trail by writing failed run records for dependency failures.
- Added deterministic runtime dependency tests with controlled `PATH` and fake Python.
- Moved T033 to `Needs_Review`.

## Review Status

Spec compliance review: Passed. Missing Python, missing Node and missing Pydantic now return structured `DependencyError` envelopes, and stale Manifest still fails before dependency probing.

Engineering review: Passed. Runtime dependency failures reuse the readiness model and flow through `finish_run`, preserving failed run records and existing envelope validation.

QA acceptance: Passed after user-requested review.

## Residual Risk

MCP tool-call behavior for `DependencyError` is not changed in T033. That is explicitly deferred to T034.
