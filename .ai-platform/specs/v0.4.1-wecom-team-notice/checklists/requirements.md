# Requirements Checklist: v0.4.1 WeCom Team Notice

Version: v0.4.1
Status: Confirmed
Created: 2026-05-13

## Scope Quality

- [x] The example is framed as a Skill Capsule, not as a WeCom adapter.
- [x] The example can be tested without a real WeCom webhook.
- [x] The real send path is opt-in and requires explicit `dry_run=false`.
- [x] No Core runtime change is required.
- [x] No bash adapter, OpenAPI import, HTTP server or install flow is introduced.

## SOP Boundary

- [x] `SKILL.md` defines purpose, SOP, prohibited behavior, required context and recovery guidance.
- [x] High/critical urgency requires `approval_id`.
- [x] All-hands audience requires `approval_id`.
- [x] Obvious secret patterns are blocked before sending.
- [x] Tool description and docs make clear that this skill can send messages only after the policy boundary passes.

## Contract Quality

- [x] Input schema has stable fields and enums.
- [x] Output schema separates `preview`, `sent` and `blocked` decisions.
- [x] `DependencyError` is used for missing webhook or external send dependency.
- [x] `PolicyViolation` is used for approval and content policy failures.
- [x] Artifact output is declared and validated.

## Agent/MCP Quality

- [x] Docs explain that Agent should call the MCP tool, not manually run `skillrun run`.
- [x] `serve --mcp --dry-run` exposes a clear tool schema.
- [x] MCP tool description includes SOP summary, prohibited behavior and recovery guidance.
- [x] Example prompt tells Agent to run dry-run first and wait for confirmation before real send.

## Release Quality

- [x] README and README.zh-CN mention `wecom_team_notice` without making it the whole project identity.
- [x] `docs/business-examples.md` adds WeCom as a v0.4.1 official example.
- [x] Release notes for v0.4.1 distinguish the example-led scope from the required Python adapter process-environment fix.
- [x] Tests cover dry-run, policy violation, dependency error, inspect/check/MCP/pack.
