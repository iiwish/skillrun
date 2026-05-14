# T043 Evidence: Business Example Tests

Status: Accepted
Commit: ade5553

## Summary

Added `wecom_team_notice_example_runs_locally_without_real_webhook` to `tests/business_examples.rs`. The test covers manifest, inspect, check, test, dry-run, approval policy violation, secret policy violation, missing-webhook dependency error, MCP dry-run, pack, unpack inspect, and unpack check.

## Review Notes

- The test removes `WECOM_WEBHOOK_URL` from the child process environment.
- No test requires network or a real WeCom webhook.
