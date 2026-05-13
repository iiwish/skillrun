# T028 Test Results

Date: 2026-05-13
Final rerun: after T028 acceptance status update

## Commands

### `cargo test`

Exit code: 0

Result: passed.

Relevant coverage included:
- Python release matrix.
- MCP stdio release matrix.
- JS alpha `init -> manifest -> inspect -> test -> run -> serve --mcp --dry-run -> pack`.
- JS MCP dry-run and stdio tool calls.
- JS `.skr` packaging without dependencies or run history.
- `--py` alias behavior.
- Adapter-aware `doctor` checks.
- Consumer Mode stale Manifest and language-boundary guards.

### `cargo run -- --version`

Exit code: 0

Output:

```text
skillrun 0.2.0
```

This confirms T028 did not perform the release-version bump.

### `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`

Exit code: 0

Output:

```text
ok: delivery artifacts passed lightweight validation
```

### `git diff --check`

Exit code: 0

Result: passed before evidence diff generation and after T028 acceptance status updates.

## Conclusion

T028 release-facing docs and governance updates are accepted and ready for the explicit release handoff decision.
