# T033 Review

Task ID: T033
Reviewer: Codex
Branch: `codex/v0.4-integration`

## Verdict

Passed.

## Spec Compliance

- Missing Python, missing Node and missing Pydantic return `DependencyError` instead of `RuntimeError`.
- Dependency errors are recoverable and include LLM-oriented retry guidance.
- Stale Manifest validation still runs first and is not converted into a dependency envelope.
- Existing validation, policy, protocol and runtime error tests remain green.

## Engineering Review

- Runtime dependency precheck runs after Manifest validation and after run paths are created, preserving both trust boundary and audit trail.
- Dependency failure envelopes use the same `finish_run` path as adapter results, so `run_id`, `run_dir`, `record` and failed run records remain consistent.
- The change does not modify MCP code; MCP-specific behavior remains scoped to T034.

## QA Acceptance

Accepted for T033.

## Residual Risk

MCP server behavior on dependency failures still needs explicit survival and tool result tests. That is T034.
