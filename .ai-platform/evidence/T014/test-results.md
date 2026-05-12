# T014 Test Results

Task ID: T014
Date: 2026-05-12
Executor: Codex direct execute fallback

## RED Baseline

Source: T013 accepted RED contract.

Command:

```powershell
cargo test --test mcp_server
```

Result before T014: Failed as expected.

Observed failure:

- `mcp_stdio_initializes_with_2025_11_25_protocol` timed out waiting for initialize response.
- Cause: non-dry-run `serve --mcp` exited instead of running a stdio MCP server.

## GREEN

Command:

```powershell
cargo test --test mcp_server
```

Result: Passed.

Observed:

- `5 passed; 0 failed; 2 ignored`
- Active stdio lifecycle tests pass.
- Existing dry-run tests still pass.
- Future tools/resources contract tests remain ignored for T015/T016.

## CLI Validation

Command:

```powershell
cargo test --test cli
```

Result: Passed.

Observed:

- `3 passed; 0 failed`

## Full Validation

Command:

```powershell
cargo test
```

Result: Passed.

Observed:

- All integration test suites passed.
- `mcp_server` reports `5 passed; 0 failed; 2 ignored`.

