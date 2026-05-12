# T012 Evidence Summary

Task ID: T012
Executor: Codex direct execute fallback
Branch / worktree context: `main` in `D:\data\skillrunv2`
Status: Accepted

## Files Changed

- `Cargo.toml`
- `README.md`
- `README.zh-CN.md`
- `tests/business_examples.rs`
- `.ai-platform/specs/v0.2/tasks.md`

## Commands Run

- `cargo test --test business_examples docs_explain_b001_to_b004_without_expanding_v0_runtime_scope`
  - RED result: failed as expected before README/Cargo edits.
  - GREEN result: passed after README/Cargo edits.
- `cargo test`
  - Result: passed.

## TDD Evidence

- RED: Added assertions requiring the README to contain `manifest-driven Agent skill capsule`, contain `FastMCP turns functions into MCP tools`, and stop using the old `tested MCP skill package` phrase.
- GREEN: Rewrote English and Chinese README first-screen narrative, status, package limitations, security boundary and roadmap wording; updated Cargo description.
- REFACTOR: Kept the rest of the README structure stable and only tightened release narrative.

## Diff Summary

- Reframed SkillRun as a manifest-driven Agent skill capsule runtime.
- Made FastMCP vs SkillRun boundary explicit in the README first screen.
- Clarified that v0.2 is the public release candidate target and that real MCP stdio serving is still pending.
- Clarified `.skr` as source + Manifest archive, not signed package, dependency bundle or sandbox.
- Added test assertions to prevent the old release narrative from returning.
- Moved T012 to `Needs_Review` in the v0.2 work graph.

## Spec Compliance Review

Status: Passed.

- Satisfies `US-201`, `FR-201` and `NFR-202`.
- Does not modify runtime behavior.
- Keeps README and Chinese README semantically aligned.

## Bug / Code Quality Review

Status: Passed.

- No implementation code changed.
- Full test suite passed.
- Cargo description no longer overclaims MCP package completion.

## QA Acceptance Review

Status: Accepted by user request to review, commit, and continue to T013.

- The README now communicates the project boundary in the first screen.
- v0.2 is described as pending work, not as already shipped.

## Residual Risks

- T012 does not implement real MCP stdio serving; that remains dependency-gated behind T013-T017.
- README will need a final release pass in T018 after MCP implementation evidence exists.
