# SkillRun v0.2.0 Release Candidate Report

Version: v0.2.0
Status: Ready_For_User_Review
Last updated: 2026-05-12
Review gate: Maintainer release decision pending

## Scope

This report records the first public release candidate for SkillRun. v0.1 was an internal MVP proof and was not published separately. v0.2.0 is the first candidate intended for public release review.

The release story is deliberately narrow:

> Turn one SOP and one action into a manifest-driven Agent skill capsule with real MCP stdio serving.

## Governance Summary

- v0.2 SOP: `.ai-platform/specs/v0.2/sop.md`
- v0.2 spec: `.ai-platform/specs/v0.2/spec.md`
- v0.2 plan: `.ai-platform/specs/v0.2/plan.md`
- v0.2 work graph: `.ai-platform/specs/v0.2/tasks.md`
- v0.2 checklist: `.ai-platform/specs/v0.2/checklists/requirements.md`
- v0.2 analysis: `.ai-platform/specs/v0.2/analysis.md`
- Release report status: `Ready_For_User_Review`
- No release tag or public package has been created.

## Accepted v0.2 Tasks

- T012: README release narrative accepted. Evidence: `.ai-platform/evidence/T012/`.
- T013: MCP stdio protocol contract tests accepted. Evidence: `.ai-platform/evidence/T013/`.
- T014: Long-running MCP stdio lifecycle accepted. Evidence: `.ai-platform/evidence/T014/`.
- T015: MCP tools/list and tools/call runtime wiring accepted. Evidence: `.ai-platform/evidence/T015/`.
- T016: MCP resources/list and resources/read accepted. Evidence: `.ai-platform/evidence/T016/`.
- T017: MCP release-level E2E fixture and release matrix accepted. Evidence: `.ai-platform/evidence/T017/`.

T018 prepares the release candidate for review and remains separate from the maintainer's publish/hold/revise decision.

## Release Candidate Capabilities

- Rust CLI/Core remains the SkillRun implementation boundary.
- Python `action.py` remains the only blessed v0.2 action adapter target.
- `skillrun manifest` generates the Manifest runtime IR from local author sources.
- Consumer Mode validates static Manifest source hashes and fails closed when stale.
- `skillrun inspect`, `skillrun test`, `skillrun run`, and `skillrun pack` remain covered by the MVP release matrix.
- `skillrun serve --mcp --cwd <capsule>` now starts a real long-running MCP stdio server.
- MCP `initialize`, `notifications/initialized`, `tools/list`, `tools/call`, `resources/list`, and `resources/read` are covered by scripted client tests.
- MCP tool calls reuse the existing runtime and preserve run records.
- MCP resources are Manifest-derived and limited to `SKILL.md` plus example input files.
- `skillrun serve --mcp --dry-run` remains available for contract inspection.
- `.skr` packages remain source + Manifest archives and exclude run history.

## Validation Summary

Validation completed on 2026-05-12:

- `cargo test`: passed.
- `cargo run -- --version`: passed.
- `cargo test --test e2e_matrix a014_mcp_stdio_release_matrix_exercises_full_client_flow`: passed as the scripted MCP client release flow.
- Delivery artifact validator: passed.

Expected release version output:

```text
skillrun 0.2.0
```

## Known Limitations

- v0.2 supports MCP stdio transport only. HTTP, SSE, Streamable HTTP, auth, and hosted server modes are out of scope.
- v0.2 exposes one primary Manifest-derived tool per capsule.
- v0.2 does not implement MCP prompts, sampling, roots, elicitation, progress, cancellation, resource subscriptions, or pagination.
- v0.2 does not provide a sandbox. Running a third-party action still means executing third-party code.
- `.skr` is not signed, not a registry package, not a dependency bundle, and not a reproducible runtime image.
- Dependency installation and runtime environment recreation are still the user's responsibility.
- Node adapter, OpenAPI import, marketplace, registry, install flow, multi-action orchestration, and GUI are post-v0.2 scope.
- Consumer Mode avoids dynamic metadata import, but it does not prevent the action process itself from using local machine capabilities once executed.

## Release Decision Checklist

- [ ] Maintainer reviews T018 diff and evidence.
- [ ] Maintainer confirms whether to publish, hold, or revise.
- [ ] If publishing, create the release tag only after explicit approval.
- [ ] If holding, record blocking issues before opening follow-up implementation work.

## User Review Gate

- Approval: Pending
- Reviewer notes: v0.2.0 is prepared for review, not published.
