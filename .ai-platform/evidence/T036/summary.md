# T036 Evidence Summary

Task ID: T036
Executor: Codex direct execute fallback
Branch: `codex/v0.4-integration`

## Files Changed

- `README.md`
- `RELEASE_NOTES.md`
- `docs/ssot.md`
- `docs/testing.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.4/analysis.md`
- `.ai-platform/specs/v0.4/tasks.md`
- `.ai-platform/evidence/T036/summary.md`
- `.ai-platform/evidence/T036/test-results.md`
- `.ai-platform/evidence/T036/review.md`
- `.ai-platform/evidence/T036/diff.patch`

## Commands Run

- Diff hygiene: `git diff --check` passed.
- Full validation: `cargo test` passed.
- Lint: `cargo clippy --all-targets -- -D warnings` passed.
- Artifact smoke: `validate_delivery_artifacts.py --task-id T036` passed with 0 errors and expected warnings for older spec directories.

## Diff Summary

- Updated README status and command narrative for v0.4 Portable Consumer Checks.
- Added explicit `inspect` / `check` / `doctor` command boundaries.
- Added a v0.4 release notes section with release matrix, validation and non-goals.
- Replaced the release report with v0.4 integration evidence and release decision checklist.
- Updated SSOT and testing docs to reflect implemented `check` behavior and hostile-environment coverage.
- Moved T036 to `Accepted`.

## Boundary Review

The documentation does not claim dependency installation, package-manager orchestration, dependency vendoring, sandboxing, signed package behavior, registry publication, HTTP transport or runtime-image support.

Version bump, tag creation, remote push and package publication remain explicit future handoff decisions.

## Review Status

Spec compliance review: Passed. The docs match FR-009 and keep the v0.4 narrative focused on Portable Consumer Checks.

Engineering review: Passed. No implementation files changed.

QA acceptance: Passed after user-requested review.

## Residual Risk

`README.zh-CN.md` already contains mojibake/encoding corruption unrelated to T036. This task avoided editing it to prevent widening the change into a documentation encoding repair.
