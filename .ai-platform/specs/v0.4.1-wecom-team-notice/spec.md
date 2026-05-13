# SkillRun v0.4.1 Feature Spec: WeCom Team Notice Example

Version: v0.4.1
Status: Confirmed
Created: 2026-05-13
Updated: 2026-05-13
Source: `docs/v0.4.1-wecom-team-notice.md`
Review: User approved the scope and requested implementation continuation on 2026-05-13.

## 一句话判断

v0.4.1 应该交付一个正式可用的 `wecom_team_notice` 示例，用来证明 SkillRun 可以把“团队通知发布 SOP”变成本地可运行、Agent 可调用、可 dry-run、可审批、可审计的 Skill Capsule。

## 北极星

**Run a real local WeCom notification skill without turning SkillRun into a WeCom wrapper.**

用户应该能通过两条路径完成同一个能力：

```text
Local CLI:
  skillrun run --cwd examples/wecom_team_notice --input examples/dry_run.input.json

Agent/MCP:
  skillrun serve --mcp --cwd examples/wecom_team_notice
```

SkillRun 的产品原子仍然是 Skill Capsule，而不是企业微信 API、MCP server 框架或 shell script。

## 目标用户

- 已经使用企业微信群机器人 webhook 的个人或团队。
- 想让本地 AI assistant 生成、预览、审批并发送团队通知。
- 需要通知 SOP、审批边界、禁发规则和审计记录随 action 一起分发。
- 能接受 Python stable action path 和 `WECOM_WEBHOOK_URL` 环境变量配置。

## Functional Requirements

- FR-041-001: Add `examples/wecom_team_notice` as a complete Python Skill Capsule.
- FR-041-002: The capsule must support dry-run preview without `WECOM_WEBHOOK_URL`.
- FR-041-003: The capsule must support real WeCom webhook send when `dry_run=false` and `WECOM_WEBHOOK_URL` is provided.
- FR-041-004: High or critical urgency and all-hands audience must require `approval_id`.
- FR-041-005: Obvious secrets must be blocked before sending.
- FR-041-006: The capsule must emit structured `PolicyViolation`, `DependencyError` and `ValidationError` behavior through existing SkillRun envelopes.
- FR-041-007: The capsule must generate a markdown notice artifact for dry-run and send paths.
- FR-041-008: `skillrun inspect`, `skillrun check`, `skillrun test`, `skillrun run`, `skillrun serve --mcp --dry-run` and `skillrun pack` must work for the example.
- FR-041-009: Documentation must explain local CLI usage and Agent/MCP usage without telling Agent to call `skillrun run` directly.

## Non-functional Requirements

- NFR-041-001: No SkillRun Core runtime expansion.
- NFR-041-002: No new adapter language; Python `action.py` remains the stable path.
- NFR-041-003: No bash action path in v0.4.1.
- NFR-041-004: No OpenAPI-to-MCP or enterprise API collection narrative.
- NFR-041-005: The example must be testable without real network or secret by default.
- NFR-041-006: Real send behavior must be opt-in and explicit.
- NFR-041-007: Docs must preserve the honest security model: declared permissions are not sandbox enforcement.

## In Scope

### P0: Capsule Example

- `examples/wecom_team_notice/SKILL.md`
- `examples/wecom_team_notice/action.py`
- `examples/wecom_team_notice/skillrun.config.json`
- `examples/wecom_team_notice/examples/*.input.json`

### P0: Business Example Tests

- Tests that prove dry-run success.
- Tests that prove approval boundary.
- Tests that prove missing webhook maps to structured `DependencyError`.
- Tests that prove MCP dry-run contract includes the tool.
- Tests that do not require a real WeCom webhook.

### P1: Documentation

- Update README / README.zh-CN business example section.
- Update `docs/business-examples.md`.
- Add or retain `docs/v0.4.1-wecom-team-notice.md`.
- Explain MCP client configuration at the concept level.

## Out Of Scope

- Enterprise WeCom adapter.
- WeCom CLI.
- bash adapter.
- OpenAPI import.
- HTTP MCP transport.
- Hosted server.
- Marketplace or install flow.
- Signed `.skr`.
- Network sandbox.
- Real webhook integration tests in CI.

## Success Criteria

- `cargo test` remains green.
- `skillrun manifest --cwd examples/wecom_team_notice` succeeds.
- `skillrun test --cwd examples/wecom_team_notice` succeeds using dry-run input.
- `skillrun run --cwd examples/wecom_team_notice --input examples/urgent_requires_approval.input.json` returns `PolicyViolation`.
- `skillrun run --cwd examples/wecom_team_notice --input examples/send.input.json` without `WECOM_WEBHOOK_URL` returns `DependencyError`.
- With a real `WECOM_WEBHOOK_URL`, a maintainer can send one local WeCom notification.
- Docs clearly explain that Agent usage should go through MCP `tools/call`, not ad hoc shell command guessing.

## Review Gate

- Approval: Granted on 2026-05-13.
- Reviewer notes: Scope approved for implementation. T041-T044 have been implemented and reviewed; T045 remains manual real-send validation because it requires a real `WECOM_WEBHOOK_URL`.
