# T015 Evidence Summary

Task ID: T015
Executor: Codex direct execute fallback
Branch / worktree context: `main` in `D:\data\skillrunv2`
Status: Accepted

## Files Changed

- `src/mcp.rs`
- `src/runtime.rs`
- `src/cli.rs`
- `tests/mcp_server.rs`
- `.ai-platform/specs/v0.2/tasks.md`

## Commands Run

- `cargo test --test mcp_server mcp_stdio_lists_and_calls_manifest_tool`
  - RED result: failed as expected before implementation.
  - GREEN result: passed after implementation.
- `cargo test --test mcp_server`
  - Result: passed.
- `cargo test --test runtime`
  - Result: passed.
- `cargo test`
  - Result: passed.

## TDD Evidence

- RED: Unignored the staged `tools/list` and `tools/call` MCP contract test; it failed against lifecycle-only server.
- GREEN: Added Manifest-derived `tools/list` and runtime-backed `tools/call`.
- REFACTOR: Added a narrow `runtime::run_with_json_input` API so MCP does not duplicate Python adapter execution logic.

## Diff Summary

- Added `runtime::run_with_json_input` and refactored runtime execution to accept JSON input values while preserving CLI file-input behavior.
- Passed `capsule_dir` into `mcp::serve_stdio` so `tools/call` can invoke the existing runtime.
- Implemented MCP `tools/list` from Manifest `tool` and `schemas.input` / `schemas.output`.
- Implemented MCP `tools/call` parameter validation, tool-name matching and runtime invocation.
- Mapped SkillRun success envelope to MCP tool result with `isError: false`.
- Mapped SkillRun structured error envelope to MCP tool result with `isError: true`.
- Added run record assertion proving `tools/call` creates a SkillRun run record with mode `mcp`.
- Left resources contract test ignored for T016.
- Moved T015 to `Needs_Review` in the v0.2 work graph.

## Spec Compliance Review

Status: Passed.

- Satisfies `US-203`, `FR-204`, `FR-205` and `NFR-203`.
- Tool schema is Manifest-derived.
- Tool call does not import action for metadata.
- Tool call reuses SkillRun runtime and preserves run records, declared env injection, artifact validation and envelope validation.

## Bug / Code Quality Review

Status: Passed.

- Existing CLI `run` and `test` behavior still passes runtime tests.
- Full test suite passed.
- MCP content avoids exposing run directory or record paths in the user-facing tool result.
- Resources remain unimplemented and dependency-gated for T016.

## QA Acceptance Review

Status: Accepted by user request to review, commit, and continue to T016.

- MCP clients can list the single Manifest-derived `refund` tool.
- MCP clients can call the tool successfully and receive a text result containing the structured output.
- Policy errors are returned as MCP tool results with `isError: true`.

## Residual Risks

- `resources/list` and `resources/read` are still not implemented.
- The MCP result currently uses text content only; structured MCP output may be refined later if needed.
