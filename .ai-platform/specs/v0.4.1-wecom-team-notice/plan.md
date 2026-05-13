# SkillRun v0.4.1 Plan: WeCom Team Notice Example

Version: v0.4.1
Status: Confirmed
Source spec: `.ai-platform/specs/v0.4.1-wecom-team-notice/spec.md`
Created: 2026-05-13

## Technical Approach

Implement `wecom_team_notice` as an official example capsule using the existing Python stable adapter. The example must not require SkillRun Core changes. The action should use Pydantic for typed input/output, the existing SDK/error envelope behavior for structured errors, and Python standard library HTTP support for the real webhook send path.

## Architecture Boundary

```text
Human CLI / MCP Agent
  -> skillrun Core
  -> Python adapter
  -> examples/wecom_team_notice/action.py
  -> WeCom webhook when dry_run=false
```

The action must not call `skillrun` recursively. Agent usage must go through MCP `tools/call`; local human usage can use `skillrun run`.

## Implementation Strategy

1. Create the capsule source files and example inputs.
2. Implement dry-run success first, with markdown artifact output.
3. Add policy boundaries for approval and secret-like content.
4. Add real send path guarded by `dry_run=false` and `WECOM_WEBHOOK_URL`.
5. Add automated tests that avoid real network.
6. Add manual-send evidence as maintainer-only release evidence.
7. Update docs and v0.4.1 release notes.

## Validation Strategy

Automated validation:

- `cargo test --test business_examples`
- `cargo test`
- `skillrun manifest --cwd examples/wecom_team_notice`
- `skillrun test --cwd examples/wecom_team_notice`
- `skillrun serve --mcp --cwd examples/wecom_team_notice --dry-run`
- `skillrun pack --cwd examples/wecom_team_notice`

Manual validation:

- Set `WECOM_WEBHOOK_URL` locally.
- Run `skillrun run --cwd examples/wecom_team_notice --input examples/send.input.json`.
- Record redacted evidence in `.ai-platform/evidence/T045/`.

## Risk Controls

- Keep real send out of CI.
- Redact webhook URLs and response details from evidence.
- Treat secret detection as a heuristic policy check, not DLP.
- Do not market declared network permissions as sandbox enforcement.

## Review Gate

- Approval: Granted on 2026-05-13.
- Reviewer notes: Implementation path accepted. T041-T044 are complete; T045 requires maintainer-provided webhook credentials and must not be faked in CI.
