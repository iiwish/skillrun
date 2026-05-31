# Lark Notice Sender

## Purpose

This SkillRun capsule sends text messages to Feishu (Lark) chats through the official `lark-cli`. It demonstrates how an external CLI tool with 200+ commands can be **contracted down to a single, bounded action** with explicit preflight guards, dry-run defaults, and audit artifacts.

This is not a general Feishu adapter. It does not expose calendars, docs, sheets, or meetings. It only sends messages, and only under strict conditions.

## SOP

1. Accept only `chat_id` values from an explicit allowlist.
2. Reject messages containing secrets, API keys, passwords, or private keys.
3. Default to `dry_run=true`: generate a preview artifact without sending.
4. Only send live messages when `dry_run=false` and all preflight checks pass.
5. Record every attempt (dry-run or live) as a markdown artifact for audit.
6. Do not silently fail: if `lark-cli` is missing or unauthenticated, return a structured error.

## Prerequisites

- `lark-cli` must be installed and authenticated (`lark-cli auth status` should succeed).
- The environment variable `LARK_NOTICE_ALLOWLIST` must be set to a comma-separated list of allowed chat IDs (e.g., `oc_xxx,oc_yyy`).
- The capsule requires outbound network access to `open.feishu.cn` or `open.larksuite.com`.

## Recovery Guidance

- If `lark-cli` is not found, install it: `npx @larksuite/cli@latest install` and run `lark-cli auth login`.
- If `chat_id` is rejected, check `LARK_NOTICE_ALLOWLIST`.
- If content is rejected for sensitive words, remove secrets or tokens from the message body.
