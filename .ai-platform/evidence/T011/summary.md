# T011 Evidence Summary

Task: T011 - Complete Refund Hero Example, Business Examples, And Full Test Strategy Validation
Status: Accepted
Date: 2026-05-12
Execution mode: Direct Execute fallback

## Direct Execute Reason

本轮用户要求在 T010 复审提交后继续推进。当前环境规则要求只有用户明确请求 sub-agents 时才可以派生 agent，因此 T011 采用 direct execute fallback，并按 packet 记录真实命令、diff 和复审证据。

## Changed Files

- `.ai-platform/docs/release-report.md`
- `.ai-platform/docs/tasks.md`
- `.ai-platform/specs/mvp/analysis.md`
- `.ai-platform/specs/mvp/tasks.md`
- `.ai-platform/specs/mvp/packets/T011.yaml`
- `README.md`
- `README.zh-CN.md`
- `docs/business-examples.md`
- `examples/refund/SKILL.md`
- `examples/refund/action.py`
- `examples/refund/examples/default.input.json`
- `examples/refund/examples/policy_violation.input.json`
- `examples/refund/examples/invalid.input.json`
- `src/adapters/python.rs`
- `tests/business_examples.rs`
- `tests/e2e_matrix.rs`
- `tests/errors.rs`

## Implementation Summary

- Added the B001 `examples/refund` hero capsule with a stronger SOP, approval boundary, success artifact, policy violation input, and invalid input.
- Added release-level `tests/e2e_matrix.rs` to exercise A001-A013 with fresh command evidence.
- Added `tests/business_examples.rs` to run the refund hero example end-to-end and verify B002-B004 remain docs-level examples.
- Updated `ValidationError` envelopes to be `recoverable=true`, matching the approved A006 release matrix.
- Updated README and Chinese README to reflect implemented pack support, current MVP state, `.skr` dependency boundary, and B001-B004 examples.
- Updated `docs/business-examples.md` to explicitly state v0.1 implements only refund and that `.skr` does not vendor dependencies.
- Updated release report to `Ready_For_User_Review` with A001-A013, N001-N016, and B001-B004 coverage summaries.

## Diff Summary

- Release validation: new E2E and business-example integration tests.
- Hero example: new refund capsule with policy boundary and markdown receipt artifact.
- Runtime behavior: `ValidationError` is now recoverable for agent follow-up.
- Documentation: status and business narrative now match implemented v0.1 behavior.
- Governance: created T011 packet, moved T011 to `Needs_Review`, and updated release readiness.

## A001-A013 Coverage Summary

- A001-A013 are covered by `tests/e2e_matrix.rs`, with supporting coverage from existing task tests.
- `tests/business_examples.rs` additionally validates the B001 refund path through manifest, inspect, test, run, serve dry-run, pack, unpack, and inspect.

## N001-N016 Coverage Summary

- N001-N016 are mapped in `.ai-platform/docs/release-report.md`.
- Coverage is automated except documented trust-boundary notes for Author Mode metadata execution and `.skr` dependency/runtime-image limitations.

## B001-B004 Summary

- B001 `Refund Decision`: implemented and tested end-to-end.
- B002 `Support Triage`: docs-level example only.
- B003 `Access Request Approval`: docs-level example only.
- B004 `Vendor Risk Review`: docs-level example only.

## Review Notes

- Spec compliance: PASS. T011 closes the release matrix without adding post-MVP runtime scope.
- Bug/code quality: PASS. New tests use temporary directories and do not mutate repository examples during execution.
- QA acceptance: PASS. Targeted, full, formatting, whitespace, command-level, and governance validations passed.
- User acceptance: PASS. User requested T011 rereview, commit, and continuation on 2026-05-12.

## Residual Risks

- Long-running MCP stdio server mode remains intentionally outside v0.1; dry-run contract exposure is the implemented MCP path.
- `.skr` remains a source/Manifest package, not a dependency-vendored runtime image.
