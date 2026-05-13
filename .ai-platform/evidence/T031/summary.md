# T031 Evidence Summary

Task ID: T031
Executor: Codex direct execute fallback
Branch: `codex/v0.4-integration`

## Files Changed

- `src/main.rs`
- `src/cli.rs`
- `src/doctor.rs`
- `src/check.rs`
- `src/readiness.rs`
- `tests/cli.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`
- `.ai-platform/specs/v0.4/tasks.md`
- `.ai-platform/evidence/T031/summary.md`
- `.ai-platform/evidence/T031/test-results.md`
- `.ai-platform/evidence/T031/review.md`
- `.ai-platform/evidence/T031/diff.patch`

## Commands Run

- RED: `cargo test --test cli --test consumer_guards --test instruction_only` failed because `check` was missing from CLI help.
- GREEN: `cargo test --test cli --test consumer_guards --test instruction_only` passed.
- Format: `cargo fmt --check` passed.
- Full validation: `cargo test` passed.
- Lint: `cargo clippy --all-targets -- -D warnings` passed.

## TDD Evidence

RED added tests for:

- CLI help listing `check`.
- Valid Python and JS capsules reporting deterministic `SkillRun Check` output without language flags.
- Stale Manifest reporting failure without creating run records.
- JS Consumer Mode no-import guard.
- Instruction-only and unsupported TypeScript diagnostics.

GREEN added a shared readiness engine, the `skillrun check --cwd <capsule>` command, and changed `doctor` to render from the same readiness findings.

## Diff Summary

- Added `src/readiness.rs` as the shared static readiness engine.
- Added `src/check.rs` as the automation-grade `check` command surface.
- Updated CLI parsing/help for `check`.
- Refactored `doctor` to reuse readiness evaluation while keeping human-friendly rendering.
- Added targeted Consumer Mode and instruction-only tests for `check`.
- Moved T031 to `Needs_Review`.

## Review Status

Spec compliance review: Passed. `check` performs static Consumer Mode readiness evaluation from Manifest, files, hashes, examples and declared requirements without running or importing action source.

Engineering review: Passed. `doctor` now reuses the readiness engine instead of maintaining duplicate file/hash logic, and `check` remains a thin CLI surface over the shared model.

QA acceptance: Passed after user-requested review.

## Residual Risk

T031 intentionally does not probe executable or package versions. Host dependency probing belongs to T032.
