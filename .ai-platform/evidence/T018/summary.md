# T018 Evidence Summary

Task ID: T018
Executor: Codex direct execute fallback
Branch / worktree context: `main` in `D:\data\skillrunv2`
Status: Accepted

## Files Changed

- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.2/analysis.md`
- `.ai-platform/specs/v0.2/tasks.md`
- `docs/ssot.md`
- `docs/mvp.md`
- `tests/cli.rs`
- `tests/business_examples.rs`
- `tests/e2e_matrix.rs`
- `tests/consumer_guards.rs`
- `tests/pack.rs`
- `tests/manifest.rs`

## Scope Note

`tests/consumer_guards.rs`, `tests/pack.rs` and `tests/manifest.rs` were updated only for version-string assertions caused by the `0.2.0` package version bump. No runtime behavior was changed.

## Commands Run

- `cargo test --test cli version_uses_approved_project_name`
  - RED result: failed as expected before the package version update.
  - GREEN result: passed after updating package/version metadata.
- `cargo test --test cli`
  - Result: passed.
- `cargo test --test business_examples`
  - Result: passed.
- `cargo test --test e2e_matrix`
  - Result: passed.
- `cargo test --test pack`
  - Result: passed.
- `cargo test --test consumer_guards valid_capsule_reaches_serve_dry_run_and_pack_success`
  - Result: passed.
- `cargo run -- --version`
  - Result: passed with `skillrun 0.2.0`.
- `cargo test --test e2e_matrix a014_mcp_stdio_release_matrix_exercises_full_client_flow`
  - Result: passed.
- `cargo fmt --check`
  - Result: passed.
- `cargo test`
  - First full run failed on a stale manifest test assertion expecting `skillrun@0.1.0`.
  - Final full run passed after updating the assertion to `skillrun@0.2.0`.

## TDD Evidence

- RED: Updated the CLI version assertion to expect `skillrun 0.2.0`; it failed while `Cargo.toml` still declared `0.1.0`.
- GREEN: Updated package version metadata, generated package filename assertions, README state, release report and release-candidate docs.
- REFACTOR: Removed stale README language that described real MCP stdio serving as a future target or unimplemented behavior.

## Diff Summary

- Bumped crate/package version from `0.1.0` to `0.2.0`.
- Updated version-sensitive tests and `.skr` archive expectations to `0.2.0`.
- Updated README and Chinese README from "v0.2 target" language to "v0.2.0 ready for user review" language.
- Documented real `serve --mcp` stdio behavior while preserving `serve --mcp --dry-run` as contract inspection.
- Added explicit release-candidate limitations for stdio-only MCP, one primary tool, Python-only adapter, no sandbox and no signed/dependency-bundled `.skr`.
- Replaced the old v0.1 release report with a v0.2.0 `Ready_For_User_Review` report.
- Added v0.1-not-published / v0.2-RC notes to `docs/ssot.md` and `docs/mvp.md`.
- Updated v0.2 analysis and moved T018 to `Needs_Review`.
- Reviewed T018 and moved it to `Accepted`.

## Spec Compliance Review

Status: Passed.

- Satisfies `US-205`, `FR-209`, `NFR-202`, `NFR-205` and `NFR-206`.
- Release report only claims accepted T012-T017 capabilities with evidence.
- Known limitations explicitly avoid sandbox, registry, signed package and dependency isolation overclaims.
- No tag or public release artifact was created.

## Bug / Code Quality Review

Status: Passed.

- Version output is consistent with `0.2.0`.
- `.skr` archive name assertions now match the crate version.
- Full test suite passed.
- The only allowed-file expansion was version assertion maintenance in existing tests.

## QA Acceptance Review

Status: Accepted by user request to review, commit, and continue.

- v0.2.0 release candidate is ready for maintainer release decision.
- The next decision remains publish, hold, or revise.

## Residual Risks

- Manual validation against a named MCP client remains optional and has not been performed.
- Release report is ready for review but not accepted; no tag should be created until the maintainer explicitly approves.
