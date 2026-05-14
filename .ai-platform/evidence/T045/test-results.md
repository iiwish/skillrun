# T045 Manual Real-send Test Results

Command:

```powershell
cargo run -- run --cwd examples/wecom_team_notice --input examples/send.input.json
```

Environment:

- `WECOM_WEBHOOK_URL` was set locally by the maintainer.
- The value is intentionally redacted.

Redacted result:

```json
{
  "ok": true,
  "output": {
    "decision": "sent",
    "audit_note": "WeCom webhook accepted the message request.",
    "wecom_response": {
      "errcode": 0,
      "errmsg": "ok"
    }
  },
  "artifacts": [
    {
      "kind": "markdown",
      "name": "wecom_notice",
      "path": "notice.md"
    }
  ]
}
```

Conclusion:

- Passed.
- No webhook secret is recorded.
