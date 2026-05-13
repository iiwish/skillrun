# T032 Evidence Summary

Task ID: T032
Executor: Codex direct execute fallback
Branch: `codex/v0.4-integration`

## Files Changed

- `src/adapters/mod.rs`
- `src/adapters/python.rs`
- `src/adapters/node.rs`
- `src/readiness.rs`
- `tests/consumer_guards.rs`
- `.ai-platform/specs/v0.4/tasks.md`
- `.ai-platform/evidence/T032/summary.md`
- `.ai-platform/evidence/T032/test-results.md`
- `.ai-platform/evidence/T032/review.md`
- `.ai-platform/evidence/T032/diff.patch`

## Commands Run

- Initial targeted validation: `cargo test --test consumer_guards --test manifest` failed because Windows fake `python.cmd` was not discovered by the probe.
- GREEN: `cargo test --test consumer_guards --test manifest` passed after adding Windows fake-runtime probe fallback.
- Format: `cargo fmt --check` passed.
- Full validation: `cargo test` passed.
- Lint: `cargo clippy --all-targets -- -D warnings` passed.

## TDD Evidence

Added deterministic host simulation tests for:

- Missing Python executable with no raw `program not found` wording.
- Missing Node executable without npm or `node_modules` checks.
- Missing Pydantic while importing only Pydantic, not action source.
- Incompatible Pydantic v1.
- Valid Python and JS check output including required and detected versions.

The first run exposed a simulation gap on Windows: fake `python.cmd` was not discovered by `Command::new("python")`. The fix keeps real `python` first and adds `python.cmd` as a Windows-only probe fallback for deterministic tests.

## Diff Summary

- Added adapter-level runtime discovery model.
- Added Python executable discovery and Pydantic version probe.
- Added Node executable discovery.
- Extended readiness evaluation with host dependency checks and `dependency-error` status.
- Rendered required and detected versions in `skillrun check` output.
- Added hostile environment tests using controlled `PATH` and fake Python.
- Moved T032 to `Needs_Review`.

## Review Status

Spec compliance review: Passed. T032 probes Python, Node and Pydantic readiness from the host without installing dependencies, parsing package-manager state or importing capsule action source.

Engineering review: Passed. Runtime discovery is exposed through adapter-level discovery functions while requirement satisfaction stays in the readiness layer, keeping the model reusable for T033.

QA acceptance: Passed after user-requested review.

## Residual Risk

T032 only diagnoses host readiness. It does not convert `run` or `test` failures into structured `DependencyError` envelopes yet; that is T033.
