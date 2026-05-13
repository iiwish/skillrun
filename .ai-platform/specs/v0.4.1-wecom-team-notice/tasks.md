# SkillRun v0.4.1 Work Graph: WeCom Team Notice Example

Version: v0.4.1
Status: In_Progress
Source spec: `.ai-platform/specs/v0.4.1-wecom-team-notice/spec.md`
Last updated: 2026-05-13
Review: User approved the spec and requested implementation continuation on 2026-05-13. T041-T044 are accepted; T045 is waiting for maintainer-provided webhook credentials.

## Work Graph Summary

```text
T041 -> T042 -> T043 -> T044
          |       |
          v       v
         T045 -> T046
```

## Epic E005: Official Real-world Skill Capsule Example

Goal:
Ship `wecom_team_notice` as the first official real-world local notification example after v0.4.0.

### T041: Create WeCom Capsule Skeleton And SOP

Status: Accepted
Priority: P0
Depends on: v0.4.0 release
Blocks: T042
Story / Requirement: FR-041-001, FR-041-009, NFR-041-004
Parallel: No
Conflicts with: T042

Goal:
Create `examples/wecom_team_notice` with `SKILL.md`, `skillrun.config.json` and example input files.

Allowed files:
- `examples/wecom_team_notice/SKILL.md`
- `examples/wecom_team_notice/skillrun.config.json`
- `examples/wecom_team_notice/examples/*.input.json`

Validation commands:
- `skillrun inspect --cwd examples/wecom_team_notice`
- `skillrun check --cwd examples/wecom_team_notice`

Test targets:
- Manual CLI smoke only until T042 adds runnable action.

Deliverables:
- Capsule directory skeleton.
- SOP with dry-run, approval and prohibited-content boundaries.
- Config and example input files.

Acceptance criteria:
- SOP clearly defines dry-run first, approval boundary, prohibited secrets and recovery guidance.
- Config declares `WECOM_WEBHOOK_URL` and outbound `qyapi.weixin.qq.com`.
- Dry-run example needs no real env or network.

Definition of Done:
- Files exist in the expected capsule shape.
- `inspect` and `check` return understandable diagnostics.

TDD plan:
- RED/GREEN is deferred to T042 because T041 is skeleton and SOP setup.

Packet path:
- `.ai-platform/specs/v0.4.1-wecom-team-notice/packets/T041.yaml`

Evidence required:
- `.ai-platform/evidence/T041/summary.md`
- `.ai-platform/evidence/T041/test-results.md`
- `.ai-platform/evidence/T041/diff.patch`

### T042: Implement Python Action With Dry-run, Policy And Artifact

Status: Accepted
Priority: P0
Depends on: T041
Blocks: T043, T045
Story / Requirement: FR-041-002, FR-041-003, FR-041-004, FR-041-005, FR-041-006, FR-041-007
Parallel: No
Conflicts with: T041, T043

Goal:
Implement `action.py` using Python stable adapter and Pydantic schema.

Allowed files:
- `examples/wecom_team_notice/action.py`
- `examples/wecom_team_notice/examples/*.input.json`

Validation commands:
- `skillrun manifest --cwd examples/wecom_team_notice`
- `skillrun test --cwd examples/wecom_team_notice`
- `skillrun run --cwd examples/wecom_team_notice --input examples/urgent_requires_approval.input.json`
- `skillrun run --cwd examples/wecom_team_notice --input examples/send.input.json`

Test targets:
- `skillrun test --cwd examples/wecom_team_notice`
- `skillrun run --cwd examples/wecom_team_notice --input examples/dry_run.input.json`
- `skillrun run --cwd examples/wecom_team_notice --input examples/urgent_requires_approval.input.json`
- `skillrun run --cwd examples/wecom_team_notice --input examples/send.input.json`

Deliverables:
- Python action with Pydantic input/output.
- Dry-run preview path.
- Real send path using `WECOM_WEBHOOK_URL`.
- Markdown artifact generation.

Acceptance criteria:
- Dry-run returns `ok: true` and markdown artifact.
- Missing approval returns `PolicyViolation`.
- Missing webhook on real send returns `DependencyError`.
- Obvious secret patterns return `PolicyViolation`.

Definition of Done:
- All validation commands produce expected success or structured error results.
- No real webhook is required for automated validation.

TDD plan:
- RED: Add example inputs and expected command outcomes.
- GREEN: Implement minimal action logic.
- REFACTOR: Keep policy checks readable and local to the example.

Packet path:
- `.ai-platform/specs/v0.4.1-wecom-team-notice/packets/T042.yaml`

Evidence required:
- `.ai-platform/evidence/T042/summary.md`
- `.ai-platform/evidence/T042/test-results.md`
- `.ai-platform/evidence/T042/diff.patch`

### T043: Add Business Example Tests

Status: Accepted
Priority: P0
Depends on: T042
Blocks: T044, T046
Story / Requirement: FR-041-008, NFR-041-005
Parallel: No
Conflicts with: T042

Goal:
Add automated tests proving the example works without a real webhook and exposes a stable MCP contract.

Allowed files:
- `tests/business_examples.rs`
- `tests/e2e_matrix.rs` if release matrix coverage is needed

Validation commands:
- `cargo test --test business_examples`
- `cargo test`

Test targets:
- `cargo test --test business_examples`

Deliverables:
- Automated business example coverage for the WeCom capsule.
- No-network tests for dry-run, policy violation and missing webhook behavior.

Acceptance criteria:
- Tests cover manifest, inspect, check, test, run dry-run, run policy violation, run missing webhook and serve dry-run.
- No test requires real `WECOM_WEBHOOK_URL`.

Definition of Done:
- Targeted tests and full `cargo test` pass.

TDD plan:
- RED: Add failing business example tests.
- GREEN: Adjust example behavior if needed.
- REFACTOR: Keep helper duplication limited.

Packet path:
- `.ai-platform/specs/v0.4.1-wecom-team-notice/packets/T043.yaml`

Evidence required:
- `.ai-platform/evidence/T043/summary.md`
- `.ai-platform/evidence/T043/test-results.md`
- `.ai-platform/evidence/T043/diff.patch`

### T044: Add Docs And README Narrative

Status: Accepted
Priority: P1
Depends on: T043
Blocks: T046
Story / Requirement: FR-041-009, NFR-041-004, NFR-041-007
Parallel: Yes
Conflicts with: T046 if release notes are edited at the same time

Goal:
Document the example as an official v0.4.1 real-world capsule while preserving SkillRun's positioning.

Allowed files:
- `README.md`
- `README.zh-CN.md`
- `docs/business-examples.md`
- `docs/v0.4.1-wecom-team-notice.md`
- `docs/README.md`

Validation commands:
- `git diff --check`
- `cargo test --test business_examples`

Test targets:
- `cargo test --test business_examples`
- `git diff --check`

Deliverables:
- README and docs updates.
- MCP usage explanation.
- Updated business example catalog.

Acceptance criteria:
- Docs explain local CLI path and Agent/MCP path.
- Docs do not claim WeCom adapter, OpenAPI import, bash action support or sandboxing.
- README keeps SkillRun positioned as portable Agent skill packaging.

Definition of Done:
- Documentation validates with `git diff --check`.
- Business example docs tests pass.

TDD plan:
- RED: Add or update docs assertions if needed.
- GREEN: Update docs.
- REFACTOR: Keep README concise.

Packet path:
- `.ai-platform/specs/v0.4.1-wecom-team-notice/packets/T044.yaml`

Evidence required:
- `.ai-platform/evidence/T044/summary.md`
- `.ai-platform/evidence/T044/test-results.md`
- `.ai-platform/evidence/T044/diff.patch`

### T045: Manual Real-send Evidence

Status: Blocked
Priority: P1
Depends on: T042
Blocks: T046
Story / Requirement: FR-041-003
Parallel: Yes
Conflicts with: None

Goal:
Record maintainer-only evidence that real WeCom webhook sending works with local env.

Allowed files:
- `.ai-platform/evidence/T045/summary.md`
- `.ai-platform/evidence/T045/test-results.md`

Validation commands:
- PowerShell with real `WECOM_WEBHOOK_URL`
- `skillrun run --cwd examples/wecom_team_notice --input examples/send.input.json`

Test targets:
- Manual maintainer run only.

Deliverables:
- Redacted manual-send summary.
- Redacted command output or result envelope.

Acceptance criteria:
- Evidence redacts webhook URL and sensitive response details.
- Evidence records success or blocked outcome.
- CI does not depend on real webhook.

Blocker:
- Requires maintainer-provided `WECOM_WEBHOOK_URL`. This must be run locally by a maintainer and redacted in evidence.

Definition of Done:
- Manual evidence exists and contains no secret.
- Automated tests remain independent of real webhook.

TDD plan:
- Not applicable; this is release evidence for an external integration.

Packet path:
- `.ai-platform/specs/v0.4.1-wecom-team-notice/packets/T045.yaml`

Evidence required:
- `.ai-platform/evidence/T045/summary.md`
- `.ai-platform/evidence/T045/test-results.md`

### T046: Prepare v0.4.1 Release Notes

Status: Pending
Priority: P1
Depends on: T043, T044, T045
Blocks: None
Story / Requirement: Release readiness
Parallel: No
Conflicts with: T044

Goal:
Prepare v0.4.1 release notes and release report.

Allowed files:
- `RELEASE_NOTES.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/v0.4.1-wecom-team-notice/tasks.md`

Validation commands:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Test targets:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Deliverables:
- v0.4.1 release notes.
- Release report update.
- Final task status updates.

Acceptance criteria:
- v0.4.1 is described as example-only unless runtime changes become necessary.
- Release notes include known limits and manual-send guidance.
- Worktree is clean before handoff.

Definition of Done:
- Release docs are ready for user review.
- Release validation commands pass.

TDD plan:
- RED/GREEN applies only if release docs introduce tested assertions.

Packet path:
- `.ai-platform/specs/v0.4.1-wecom-team-notice/packets/T046.yaml`

Evidence required:
- `.ai-platform/evidence/T046/summary.md`
- `.ai-platform/evidence/T046/test-results.md`
- `.ai-platform/evidence/T046/diff.patch`

## User Review Gate

- Approval: Granted on 2026-05-13.
- Reviewer notes: T041, T042, T043 and T044 were implemented, reviewed and accepted. T045 is blocked on maintainer-only real webhook validation. T046 remains pending until T045 is completed or explicitly waived.
