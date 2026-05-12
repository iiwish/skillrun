# T013 Evidence Summary

Task ID: T013
Executor: Codex direct execute fallback
Branch / worktree context: `main` in `D:\data\skillrunv2`
Status: Accepted

## Files Changed

- `tests/mcp_server.rs`
- `.ai-platform/specs/v0.2/tasks.md`

## Commands Run

- `cargo test --test mcp_server`
  - Result: failed as expected.

## TDD Evidence

- RED: Added a bounded scripted MCP stdio client fixture that spawns `skillrun serve --mcp`, writes newline-delimited JSON-RPC requests to stdin, and waits for stdout responses with a timeout.
- RED: Added active lifecycle test `mcp_stdio_initializes_with_2025_11_25_protocol`.
- RED staging: Added ignored future contract tests for `tools/list` / `tools/call` and `resources/list` / `resources/read`; these compile now and are intended to be enabled by T015 and T016.
- GREEN: Not part of T013. Production implementation is intentionally deferred to T014-T016.

## Diff Summary

- Added `ScriptedMcpClient` fixture with child process cleanup and bounded stdout response waits.
- Added MCP `initialize` contract test targeting protocol version `2025-11-25`.
- Added ignored tools contract test covering Manifest-derived schema, successful tool call, and structured policy error mapping.
- Added ignored resources contract test covering `resources/list`, `resources/read`, markdown contents, and traversal rejection.
- Moved T013 to `Needs_Review` in the v0.2 work graph.

## Spec Compliance Review

Status: Passed.

- Satisfies `US-202`, `US-203`, `US-204`, `FR-202`, `FR-203`, `FR-208`, `NFR-201` and `NFR-204` as a RED contract task.
- Does not modify production MCP, CLI or runtime implementation.
- Uses a bounded subprocess fixture to avoid hanging test processes.

## Bug / Code Quality Review

Status: Passed with expected RED failure.

- Fixture owns and kills the child process on drop.
- Future-stage tests are ignored to avoid making T014 impossible before T015/T016.
- Current failure is caused by missing server behavior, not by a harness compile/runtime bug.

## QA Acceptance Review

Status: Accepted by user request to review, commit, and continue to T014.

- T013 creates the protocol contract baseline needed by T014.
- The worktree intentionally contains a failing active test until T014 implements lifecycle.

## Residual Risks

- The active RED test means `cargo test --test mcp_server` fails until T014.
- T015/T016 must remember to unignore or replace the staged tools/resources tests when implementing those behaviors.
