# T045 Manual Real-send Evidence Summary

Status: Accepted
Date: 2026-05-14

## Scope

Maintainer validated the `examples/wecom_team_notice` real-send path with a local `WECOM_WEBHOOK_URL`.

## Result

- `skillrun run --cwd examples/wecom_team_notice --input examples/send.input.json` returned `ok=true`.
- Output decision was `sent`.
- WeCom response returned `errcode=0` and `errmsg=ok`.
- The run produced the `wecom_notice` markdown artifact.

## Redaction

The webhook URL and key are not recorded in evidence. CI and automated tests continue to avoid real webhook credentials.
