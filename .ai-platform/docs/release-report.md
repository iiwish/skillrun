# SkillRun v0.4 Release Report

Version: v0.4.0
Status: Ready_For_Release_Decision
Last updated: 2026-05-13
Review gate: T029-T036 evidence accepted; release handoff decisions remain pending

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
- Release report status: `Ready_For_Release_Decision`
- Local release tag: `v0.4.0` not created by this task
- Remote push and package publication: not performed by this handoff

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

Tag creation, remote push and package publication remain separate release handoff decisions.

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
- [ ] Create local release tag after explicit approval.
- [ ] Push tag to remote, if desired.
- [ ] Publish package/artifact, if desired.

## User Review Gate

- Approval: Granted for T036 on 2026-05-13.
- Reviewer notes: v0.4 implementation and documentation evidence through T036 is accepted. The package version has been bumped to `0.4.0`; local tag, remote push and package publication remain separate explicit decisions.
