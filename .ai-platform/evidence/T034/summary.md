# T034 Evidence Summary

Task ID: T034
Executor: Codex direct execute fallback
Branch: `codex/v0.4-integration`

## Files Changed

- `tests/mcp_server.rs`
- `tests/fixtures/mcp_stdio.rs`
- `.ai-platform/specs/v0.4/tasks.md`
- `.ai-platform/evidence/T034/summary.md`
- `.ai-platform/evidence/T034/test-results.md`
- `.ai-platform/evidence/T034/review.md`
- `.ai-platform/evidence/T034/diff.patch`

## Commands Run

- Characterization: `cargo test --test mcp_server` passed after adding the dependency-failure survival test.
- Full validation: `cargo test` passed.
- Format: `cargo fmt --check` passed.
- Lint: `cargo clippy --all-targets -- -D warnings` initially failed because `spawn_with_path` is unused in other test targets that include the shared fixture, then passed after adding a narrow `#[allow(dead_code)]`.

## TDD Evidence

The new MCP survival test did not produce a RED failure because T033's runtime `DependencyError` envelope already flowed through MCP's existing generic `ok:false` tool result mapping.

The test still adds the missing acceptance guard:

- Start an MCP stdio server with a controlled empty `PATH`.
- Call the JS tool and receive `isError: true`.
- Assert the tool text contains `DependencyError`, `node` and `missing`.
- Send a later `tools/list` request and receive a valid response, proving the server remains alive.

## Diff Summary

- Added `ScriptedMcpClient::spawn_with_path` for deterministic MCP hostile-environment tests.
- Added MCP dependency-failure survival coverage.
- Did not change `src/mcp.rs`; existing language-neutral tool result mapping already satisfies T034.
- Moved T034 to `Needs_Review`.

## Review Status

Spec compliance review: Passed. MCP `tools/call` dependency failure returns `isError: true`, includes `DependencyError` text and keeps the stdio server alive for a later `tools/list`.

Engineering review: Passed. The task adds a focused survival test and does not change MCP production code, HTTP transport, runtime envelope shape or language-specific MCP branches.

QA acceptance: Passed after user-requested review.

## Residual Risk

T034 covers stdio MCP behavior. HTTP transport remains explicitly out of scope for v0.4.
