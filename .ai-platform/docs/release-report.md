# SkillRun Release Report

## v0.5.0 Release Candidate

Version: v0.5.0
Status: Ready_For_User_Review
Last updated: 2026-05-14
Review gate: T050-T055 evidence prepared on `codex/v0.5-integration`; merge, tag creation and publication remain pending explicit user command

### Scope

v0.5.0 is the Adapter Protocol release candidate. It moves “language support” from ad hoc Core behavior into a documented adapter boundary, then proves that boundary with a Level 0 command adapter.

The release story is deliberately constrained:

> Core reads Manifest, creates IPC, validates envelopes, and exposes MCP; adapters bridge action ecosystems back into that contract.

### Included v0.5.0 Work

- Public Adapter Protocol contract: `docs/adapter-protocol.md`.
- v0.5 design boundary: `docs/v0.5-adapter-protocol.md`.
- Conformance tests for Python stable and JS alpha adapter behavior.
- Command adapter Manifest support with explicit argv command and static JSON schemas.
- Command executable readiness diagnostics without source metadata import or dependency installation.
- Level 0 command adapter runtime dispatch through standard SkillRun IPC env vars.
- Command adapter stdout/stderr log discipline and Core envelope/artifact validation.
- SDK-free command adapter example: `examples/command_hello`.
- README, release notes, business example catalog and version metadata updates.
- Version bump from `0.4.2` to `0.5.0`.

### Validation Summary

- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `cargo test --test runtime --test errors --test adapter_conformance`: passed during T053.
- `cargo test --test business_examples`: passed during T054.
- `cargo test`: passed after one transient business example empty-output failure was rerun successfully.
- `cargo clippy --all-targets -- -D warnings`: passed.
- Delivery artifact validator: passed with non-blocking legacy-spec scan warnings.

### Review Summary

- Spec compliance review: no blocking issue found against the v0.5 scope. The work defines Adapter Protocol, conformance coverage, command Manifest support, command runtime support and a runnable example.
- Bug/code-quality review: runtime and business example suites pass; full suite passes on rerun.
- QA acceptance review: ready for user review; merge, tag and publication decisions remain separate explicit user actions.

### Known Limits

- This release does not introduce registry, marketplace, `skillrun install`, signed capsules, trusted downloads, dependency vendoring or runtime images.
- Command adapter uses explicit argv only; shell strings remain rejected.
- Command adapter readiness diagnoses executable presence; it does not install command dependencies.
- Command adapter is not an OS sandbox. Running third-party command actions still executes third-party code.
- Python stable and JS alpha remain the only language adapters with authoring conveniences. Command adapter is protocol-level execution, not a new blessed SDK.

## v0.4.2 Release Candidate

Version: v0.4.2
Status: Ready_For_User_Review
Last updated: 2026-05-14
Review gate: User requested completion of the v0.4.2 documentation and example-capsule slice on 2026-05-14; user acceptance, merge, tag creation and publication remain pending explicit command

### Scope

v0.4.2 is a positioning and example-led patch release candidate. It does not change the SkillRun runtime architecture. It sharpens the public narrative and adds official reference capsules that demonstrate reusable SOP-backed preflight patterns.

The release story is deliberately constrained:

> Move agent safety rules out of fragile prompts and into inspectable, testable Skill Contracts.

### Included v0.4.2 Work

- Project positioning: `docs/positioning.md`.
- Long-term vision: `docs/vision.md`.
- Honest trust model: `docs/trust-model.md`.
- Official capsule design: `docs/v0.4.2-official-capsules.md`.
- Commit message reference capsule: `examples/commit_message_gate`.
- Bounded file patcher reference capsule: `examples/bounded_file_patcher`.
- Read-only diagnostics reference capsule: `examples/readonly_diagnostics_runner`.
- README, docs index, SSOT, business example catalog and release notes updates.
- Version bump from `0.4.1` to `0.4.2`.

### Validation Summary

- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `cargo test --test business_examples`: passed, including the v0.4.2 reference capsule matrix.
- `cargo test`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- Detailed official capsule matrix: passed for all three v0.4.2 reference capsules on fresh temporary copies.

### Review Summary

- Spec compliance review: no blocking issue found against the v0.4.2 scope. The work stays within documentation, examples, version metadata and tests.
- Bug/code-quality review: no blocking issue found after full test and clippy validation.
- QA acceptance review: ready for user review; final user acceptance is still pending.

### Known Limits

- This release does not introduce registry, marketplace, `skillrun install`, signed capsules, dependency vendoring or runtime images.
- This release does not introduce a new adapter or implement the v0.5 language-agnostic Adapter Protocol.
- `readonly_diagnostics_runner` is not a general-purpose shell.
- `bounded_file_patcher` is not an OS sandbox.
- `commit_message_gate` does not stage files automatically.
- `.skr` remains source + Manifest archive.

## v0.4.1 Merge Readiness

Version: v0.4.1
Status: Ready_For_Main_Merge_Review
Last updated: 2026-05-14
Review gate: T041-T046 evidence accepted; merge, tag creation and publication remain pending explicit user command

### Scope

v0.4.1 is an example-led patch release candidate. It adds the official `wecom_team_notice` Skill Capsule and includes one narrow Python adapter fix discovered during real Windows webhook validation.

The release story remains constrained:

> Run a real local WeCom notification skill without turning SkillRun into a WeCom wrapper.

### Accepted v0.4.1 Tasks

- T041: WeCom capsule skeleton accepted. Evidence: `.ai-platform/evidence/T041/`.
- T042: WeCom action implementation accepted. Evidence: `.ai-platform/evidence/T042/`.
- T043: Business example test matrix accepted. Evidence: `.ai-platform/evidence/T043/`.
- T044: v0.4.1 example documentation accepted. Evidence: `.ai-platform/evidence/T044/`.
- T045: Manual real-send validation accepted. Evidence: `.ai-platform/evidence/T045/`.
- T046: Release notes and merge-readiness docs accepted. Evidence: `.ai-platform/evidence/T046/`.

### Validation Summary

- Maintainer real send: `ok=true`, `decision=sent`, WeCom response `errcode=0`, with webhook URL omitted from evidence.
- Python adapter fix: preserves baseline Windows process env while still injecting business env only when declared by Manifest permissions.
- Full local validation passed: `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `git diff --check`.

### Known Limits

- This is not a WeCom adapter, OpenAPI-to-MCP bridge, WeCom CLI wrapper, bash adapter, hosted server, registry release or sandbox release.
- Real webhook sending remains manual and opt-in; CI covers dry-run and structured error paths only.
- The Python adapter process-env fix preserves host runtime viability; it does not expose arbitrary undeclared business secrets.

## v0.4.0 Release Record

Version: v0.4.0
Status: Released
Last updated: 2026-05-13
Review gate: T029-T036 evidence accepted; tag and public release artifact publication completed by maintainer

## Scope

This report records the v0.4.0 release-candidate evidence for SkillRun after the v0.3.0 local release handoff.

The release story is deliberately narrow:

> A Skill Capsule can be inspected and dependency-checked after distribution, even when the current machine cannot run it.

v0.4 is not an HTTP transport, installer, package manager, registry, sandbox, signed package or runtime-image release.

## Governance Summary

- v0.4 spec: `.ai-platform/specs/v0.4/spec.md`
- v0.4 plan: `.ai-platform/specs/v0.4/plan.md`
- v0.4 work graph: `.ai-platform/specs/v0.4/tasks.md`
- v0.4 checklist: `.ai-platform/specs/v0.4/checklists/requirements.md`
- v0.4 analysis: `.ai-platform/specs/v0.4/analysis.md`
- Release notes draft: `RELEASE_NOTES.md`
- Release report status: `Released`
- Local release tag: `v0.4.0` created after explicit approval
- Remote push and GitHub release artifact publication: completed by maintainer

## Accepted v0.4 Tasks

- T029: DependencyError contract accepted. Evidence: `.ai-platform/evidence/T029/`.
- T030: Manifest runtime requirements accepted. Evidence: `.ai-platform/evidence/T030/`.
- T031: Readiness engine and `skillrun check` accepted. Evidence: `.ai-platform/evidence/T031/`.
- T032: Python and Node runtime discovery accepted. Evidence: `.ai-platform/evidence/T032/`.
- T033: Runtime dependency failures mapped to `DependencyError` accepted. Evidence: `.ai-platform/evidence/T033/`.
- T034: MCP server survival on `DependencyError` accepted. Evidence: `.ai-platform/evidence/T034/`.
- T035: Portable `.skr` check matrix accepted. Evidence: `.ai-platform/evidence/T035/`.

T036 prepared release-facing documentation and validation evidence and was accepted on 2026-05-13.

## Release Candidate Capabilities

- `skillrun check --cwd <capsule>` statically diagnoses capsule readiness.
- `inspect` displays the Manifest contract; `check` diagnoses host readiness; `doctor` remains the human-friendly diagnostic view.
- Manifest runtime requirements record adapter-default Python and Node requirements.
- Python readiness checks report Python executable and Pydantic v2 status.
- Node readiness checks report Node executable status and do not introduce npm/package-manager checks.
- Missing Python, Node or Pydantic produces structured `DependencyError` instead of leaking raw spawn/import failures.
- `test`, `run` and MCP `tools/call` preserve structured dependency failure semantics.
- MCP stdio server remains alive after one tool call hits a dependency failure.
- `.skr` archives remain source + Manifest archives, but unpacked capsules can run `inspect` and `check`.
- Consumer Mode diagnostics do not import `action.py` or `action.mjs` for metadata.

## Release Matrix

| Capability area | Evidence path | Status |
| --- | --- | --- |
| Error contract | `cargo test --test errors --test cli --test consumer_guards`; `.ai-platform/evidence/T029/` | Passed |
| Manifest runtime requirements | `cargo test --test manifest --test pack`; `.ai-platform/evidence/T030/` | Passed |
| Readiness engine and `check` | `cargo test --test cli --test consumer_guards --test instruction_only`; `.ai-platform/evidence/T031/` | Passed |
| Python and Node runtime discovery | `cargo test --test consumer_guards --test manifest`; `.ai-platform/evidence/T032/` | Passed |
| Runtime `DependencyError` envelope | `cargo test --test runtime --test errors`; `.ai-platform/evidence/T033/` | Passed |
| MCP dependency survival | `cargo test --test mcp_server`; `.ai-platform/evidence/T034/` | Passed |
| Portable `.skr` diagnosis | `cargo test --test pack --test e2e_matrix`; `.ai-platform/evidence/T035/` | Passed |
| Full regression | `cargo test`; `cargo clippy --all-targets -- -D warnings`; `git diff --check` | Passed in T035/T036 validation |

## Validation Summary

Release-documentation validation completed on 2026-05-13:

- `git diff --check`: passed.
- `cargo test`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.

Current local binary version output after the release bump:

```text
skillrun 0.4.0
```

Tag creation, remote push and GitHub release artifact publication were completed after release approval.

## Known Limitations

- v0.4 diagnoses dependencies but does not install Python, Node, Pydantic, npm packages, virtualenvs or runtime images.
- `.skr` is not signed, not a registry package, not a dependency bundle, and not a reproducible runtime image.
- SkillRun is not an OS sandbox. Running third-party actions still means executing third-party code.
- Consumer Mode avoids dynamic metadata import, but executing an action still executes third-party code.
- MCP transport remains stdio only. HTTP, SSE, Streamable HTTP, auth and hosted server modes are out of scope.
- JS support remains JS Action Alpha through canonical ESM `action.mjs`.
- `action.ts`, TypeScript runtime execution, `ts-node`, `tsx`, source maps, CJS/ESM compatibility matrices and package-manager install flows remain out of scope.
- Runtime commands remain Manifest-only and do not accept language flags.

## Release Decision Checklist

- [x] T029-T035 accepted.
- [x] T036 release docs prepared.
- [x] Maintainer reviews T036 diff and evidence.
- [x] Maintainer accepts v0.4 release candidate.
- [x] Bump Cargo/package version from `0.3.0` to `0.4.0`.
- [x] Create local release tag after explicit approval.
- [x] Push tag to remote, if desired.
- [x] Publish package/artifact, if desired.

## User Review Gate

- Approval: Granted for T036 on 2026-05-13.
- Reviewer notes: v0.4 implementation and documentation evidence through T036 is accepted. The package version was bumped to `0.4.0`; tag, remote push and public artifact publication were completed by the maintainer.
