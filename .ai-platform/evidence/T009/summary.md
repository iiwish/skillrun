# T009 Evidence Summary

Task: T009 - Implement Manifest-driven MCP Tool Exposure
Status: Accepted
Date: 2026-05-12
Execution mode: Direct Execute fallback

## Direct Execute Reason

本轮用户要求继续推进当前 governed task，但没有显式要求使用 sub-agent；当前环境规则也要求只有用户明确请求 sub-agents 时才可以派生 agent。因此 T009 采用 direct execute fallback，并按 packet 记录真实命令、diff 和复审证据。

## Changed Files

- `.ai-platform/docs/release-report.md`
- `.ai-platform/docs/tasks.md`
- `.ai-platform/specs/mvp/analysis.md`
- `.ai-platform/specs/mvp/tasks.md`
- `.ai-platform/specs/mvp/packets/T009.yaml`
- `README.md`
- `README.zh-CN.md`
- `src/cli.rs`
- `src/main.rs`
- `src/mcp.rs`
- `tests/consumer_guards.rs`
- `tests/mcp_server.rs`

## Implementation Summary

- Added `src/mcp.rs` as an isolated Manifest-to-MCP dry-run contract builder.
- Wired `skillrun serve --mcp --dry-run` through the existing Consumer Mode guard before building any contract.
- Dry-run JSON now exposes Manifest-derived tool name, description, input schema, output schema, result envelope contract, Manifest hash, and `SKILL.md` resource content.
- Preserved fail-closed stale Manifest behavior for `serve --mcp --dry-run`.
- Kept long-running MCP server mode explicitly unimplemented with a clear message.
- Updated Consumer guard tests so valid dry-run succeeds while `pack` remains unimplemented.
- Updated README files to distinguish implemented MCP dry-run exposure from planned long-running MCP server and `.skr` packaging.

## Diff Summary

- CLI: parse and honor `--dry-run`; route valid dry-run requests to the MCP contract builder.
- MCP: derive all public MCP contract fields from Manifest and `SKILL.md`; no Consumer Mode metadata import from `action.py`.
- Tests: added MCP dry-run schema/resource, no-import, and stale Manifest cases.
- Governance: created T009 packet, moved T009 to `Needs_Review`, updated analysis and release ledger.

## Review Notes

- Spec compliance: PASS. T009 implements Manifest-driven MCP dry-run exposure and leaves T010 packaging untouched.
- Bug/code quality: PASS. MCP code is isolated, Consumer Mode guard order is preserved, and stale Manifest errors remain command-specific.
- QA acceptance: PASS. Targeted, full, formatting, whitespace, and end-to-end dry-run validation passed.
- User acceptance: PASS. User requested T009 rereview, commit, and continuation to T010 on 2026-05-12.

## Residual Risks

- T009 intentionally implements dry-run contract verification only; the long-running stdio MCP server loop remains future work.
- MCP resource URI uses the Manifest skill name directly; future packaging/install work may need URI escaping if broader name characters are allowed.
- `.skr` packaging is still unimplemented and remains T010 scope.
