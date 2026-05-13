# Requirements Checklist: v0.4.1 WeCom Team Notice

Version: v0.4.1
Status: Ready_For_User_Review
Created: 2026-05-13

## Scope Quality

- [ ] The example is framed as a Skill Capsule, not as a WeCom adapter.
- [ ] The example can be tested without a real WeCom webhook.
- [ ] The real send path is opt-in and requires explicit `dry_run=false`.
- [ ] No Core runtime change is required.
- [ ] No bash adapter, OpenAPI import, HTTP server or install flow is introduced.

## SOP Boundary

- [ ] `SKILL.md` defines purpose, SOP, prohibited behavior, required context and recovery guidance.
- [ ] High/critical urgency requires `approval_id`.
- [ ] All-hands audience requires `approval_id`.
- [ ] Obvious secret patterns are blocked before sending.
- [ ] Tool description and docs make clear that this skill can send messages only after the policy boundary passes.

## Contract Quality

- [ ] Input schema has stable fields and enums.
- [ ] Output schema separates `preview`, `sent` and `blocked` decisions.
- [ ] `DependencyError` is used for missing webhook or external send dependency.
- [ ] `PolicyViolation` is used for approval and content policy failures.
- [ ] Artifact output is declared and validated.

## Agent/MCP Quality

- [ ] Docs explain that Agent should call the MCP tool, not manually run `skillrun run`.
- [ ] `serve --mcp --dry-run` exposes a clear tool schema.
- [ ] MCP tool description includes SOP summary, prohibited behavior and recovery guidance.
- [ ] Example prompt tells Agent to run dry-run first and wait for confirmation before real send.

## Release Quality

- [ ] README and README.zh-CN mention `wecom_team_notice` without making it the whole project identity.
- [ ] `docs/business-examples.md` adds WeCom as a v0.4.1 official example.
- [ ] Release notes for v0.4.1 distinguish example-only scope from runtime changes.
- [ ] Tests cover dry-run, policy violation, dependency error, inspect/check/MCP/pack.

