# T041 Evidence: WeCom Capsule Skeleton And SOP

Status: Accepted
Commit: ade5553

## Summary

Created `examples/wecom_team_notice` capsule skeleton with `SKILL.md`, `skillrun.config.json`, and example inputs. The SOP defines dry-run-first behavior, approval boundaries, prohibited secret-like content, and Agent/MCP usage guidance.

## Review Notes

- The example is a Skill Capsule, not a WeCom adapter.
- The config declares `WECOM_WEBHOOK_URL` and outbound `qyapi.weixin.qq.com`.
- Dry-run input does not require real env or network.
