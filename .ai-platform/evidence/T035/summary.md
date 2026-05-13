# T035 Evidence Summary

Task ID: T035
Executor: Codex direct execute fallback
Branch: `codex/v0.4-integration`

## Files Changed

- `tests/pack.rs`
- `.ai-platform/specs/v0.4/tasks.md`
- `.ai-platform/evidence/T035/summary.md`
- `.ai-platform/evidence/T035/test-results.md`
- `.ai-platform/evidence/T035/review.md`
- `.ai-platform/evidence/T035/diff.patch`

## Commands Run

- Target validation: `cargo test --test pack --test e2e_matrix` passed.
- Format: `cargo fmt --check` passed.
- Full validation: `cargo test` passed.
- Diff hygiene: `git diff --check` passed.
- Lint: `cargo clippy --all-targets -- -D warnings` passed.
- Artifact smoke: `validate_delivery_artifacts.py --task-id T035` passed with 0 errors and expected warnings for older spec directories.

## TDD Evidence

The added `.skr` check matrix passed without production code changes. This is characterization coverage for the v0.4 boundary already introduced by T031 and T032.

The tests now prove:

- Unpacked Python `.skr` capsules can run `inspect` and `check`.
- Unpacked JS `.skr` capsules can run `inspect` and `check`.
- Missing Python or Node produces `status: dependency-error`.
- Dependency failure still reports `manifest freshness: fresh` and the action source as fresh.
- JS `.skr` checks do not mention `npm` or `node_modules`.

## Diff Summary

- Added pack-test helpers for controlled empty `PATH` execution and stdout assertions.
- Extended Python and JS pack/unpack tests to run `check` after `inspect`.
- Added dependency-failure assertions for unpacked capsules without changing `.skr` archive contents or vendoring behavior.
- Moved T035 to `Needs_Review`.

## Review Status

Spec compliance review: Passed. Unpacked Python and JS `.skr` archives can run `inspect` and `check`, and dependency failure does not affect source freshness reporting.

Engineering review: Passed. No production code changed; tests stay focused on package diagnosability and do not introduce vendoring or package-manager behavior.

QA acceptance: Passed after user-requested review.

## Residual Risk

The successful `check` path depends on the local test host having the expected Python/Pydantic and Node runtimes available, consistent with the existing test suite. Hostile missing-runtime behavior is deterministic through empty `PATH`.
