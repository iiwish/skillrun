# T017 Evidence Summary

Task ID: T017
Executor: Codex direct execute fallback
Branch / worktree context: `main` in `D:\data\skillrunv2`
Status: Accepted

## Files Changed

- `tests/fixtures/mcp_stdio.rs`
- `tests/mcp_server.rs`
- `tests/e2e_matrix.rs`
- `.ai-platform/specs/v0.2/analysis.md`
- `.ai-platform/specs/v0.2/tasks.md`

## Commands Run

- `cargo test --test e2e_matrix a014_mcp_stdio_release_matrix_exercises_full_client_flow`
  - RED result: failed as expected with an explicit release-matrix coverage gap.
  - GREEN result: passed after adding the full MCP stdio release flow.
- `cargo test --test mcp_server`
  - Result: passed.
- `cargo test --test e2e_matrix`
  - Result: passed.
- `cargo fmt --check`
  - Result: passed.
- `cargo test`
  - Result: passed.

## TDD Evidence

- RED: Added `a014_mcp_stdio_release_matrix_exercises_full_client_flow` as a failing release-matrix placeholder to prove v0.2 had no single release-level MCP stdio client flow.
- GREEN: Implemented the release-level flow using a shared `ScriptedMcpClient` fixture.
- REFACTOR: Moved the scripted MCP client out of `tests/mcp_server.rs` into `tests/fixtures/mcp_stdio.rs` so protocol and release tests use the same bounded subprocess harness.

## Diff Summary

- Added shared MCP stdio test fixture with bounded stdout reads and a helper that fails if an operation writes unexpected stdout.
- Refactored `tests/mcp_server.rs` to use the shared fixture without weakening existing lifecycle, tools or resources tests.
- Added `a014_mcp_stdio_release_matrix_exercises_full_client_flow` to the release matrix.
- The release matrix now exercises `initialize`, `notifications/initialized`, `tools/list`, successful `tools/call`, structured-error `tools/call`, `resources/list`, `resources/read` for `SKILL.md` and `resources/read` for example JSON.
- Injected action stdout noise during the release-level MCP tool call and asserted the noise is captured in `stdout.log`, not emitted into MCP JSON-RPC stdout.
- Updated v0.2 analysis to record the T017 release-matrix coverage and preserve zero Critical/High findings.
- Moved T017 to `Accepted` in the v0.2 work graph.
- Moved T018 to `Ready` because T012 and T017 are now accepted.

## Spec Compliance Review

Status: Passed.

- Satisfies `US-202`, `US-203`, `US-204`, `FR-208`, `NFR-201` and `NFR-204`.
- Release matrix covers all v0.2 MCP P0 behavior.
- stdout discipline is explicitly tested at the MCP subprocess boundary.
- No runtime feature or protocol behavior was changed.

## Bug / Code Quality Review

Status: Passed.

- Shared fixture keeps timeouts bounded and kills the child process on drop.
- No broad unrelated test churn.
- Full test suite passed.
- Analysis records no Critical or High findings.

## QA Acceptance Review

Status: Accepted by user request to review, commit, and continue to T018.

- T017 is complete and accepted.
- T018 is ready for release-candidate preparation.

## Residual Risks

- The scripted fixture is sufficient for the v0.2 release gate, but it is not a replacement for optional manual validation against a named MCP client.
- The stdout discipline helper checks for unexpected newline-delimited stdout after selected operations; it does not attempt to model every future MCP notification type.
