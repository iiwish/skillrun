# WeCom Team Notice Skill

## Purpose

This SkillRun capsule turns a team notice request into a preview or a WeCom
group robot message. It is safe for an agent to call when the agent needs a
policy-bound team notification, not when it should freely send arbitrary text.

## SOP

1. Prefer `dry_run=true` before sending any real WeCom message.
2. Confirm the notice has a non-empty `title`, `summary`, supported `audience`,
   supported `urgency`, and explicit `dry_run` value.
3. Reject high-risk content before sending, including obvious secret-like
   strings such as API keys, private keys, passwords, tokens, and webhook URLs.
4. Require `approval_id` for `urgency=high`, `urgency=critical`, or
   `audience=all_hands`.
5. When `dry_run=false`, require the declared `WECOM_WEBHOOK_URL` environment
   variable.
6. Produce a markdown notice artifact for preview, sent, and blocked outcomes.
7. Never treat stdout as the business result.

## Approval Boundary

- `dry_run=true` is allowed without `approval_id` unless the content violates
  prohibited-content rules.
- `urgency=normal` for `team`, `project`, or `incident` can be sent when the
  webhook is configured.
- `urgency=high`, `urgency=critical`, and `audience=all_hands` require
  `approval_id` before a real send or preview approval can continue.
- The capsule only sends to the configured WeCom group robot webhook. It does
  not manage WeCom users, contacts, departments, apps, approvals, or chats.

## Required Context

- `title`: short notice title.
- `summary`: notice body.
- `audience`: one of `team`, `project`, `incident`, or `all_hands`.
- `urgency`: one of `normal`, `high`, or `critical`.
- `dry_run`: whether to preview only.
- `approval_id`: required for high-risk sends.
- `mentioned_mobile_list`: optional WeCom mobile mentions.

## Recovery Guidance

If the action returns `PolicyViolation`, remove prohibited content or provide
the required approval before retrying. If it returns `DependencyError`, configure
`WECOM_WEBHOOK_URL` for real sends or retry with `dry_run=true`.

## Prohibited Behavior

- Do not send secrets, tokens, private keys, passwords, or webhook URLs.
- Do not bypass approval for high, critical, or all-hands notices.
- Do not use this capsule as a general WeCom API client.
- Do not ask the agent to manually run `skillrun run`; configure this capsule as
  an MCP server and call the Manifest-derived tool.

