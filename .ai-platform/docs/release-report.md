# SkillRun Release Report

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
