# T042 Evidence: Python Action With Dry-run, Policy And Artifact

Status: Accepted
Commit: ade5553

## Summary

Implemented `examples/wecom_team_notice/action.py` using the Python stable adapter and Pydantic schemas. The action supports dry-run preview, policy checks, missing-webhook `DependencyError`, WeCom webhook send path, and markdown artifact generation.

## Review Notes

- `dry_run=true` does not call WeCom.
- `dry_run=false` requires `WECOM_WEBHOOK_URL`.
- High/critical and all-hands notices require `approval_id`.
- Secret-like content is blocked with `PolicyViolation`.
