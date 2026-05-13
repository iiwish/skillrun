# SkillRun v0.3 Release Report

Version: v0.3.0
Status: Released_Local
Last updated: 2026-05-13
Review gate: T028 evidence accepted; local release handoff requested by maintainer

## Scope

This report records the v0.3 release candidate evidence for SkillRun after the v0.2.0 local release handoff.

The release story is deliberately narrow:

> Keep Python stable, prove the adapter boundary, and ship JS Action Alpha without claiming full TypeScript or dependency-management support.

v0.3 is not a general Node framework, TypeScript runtime, package manager, registry, sandbox, or new MCP transport release.

## Governance Summary

- v0.3 spec: `.ai-platform/specs/v0.3/spec.md`
- v0.3 plan: `.ai-platform/specs/v0.3/plan.md`
- v0.3 work graph: `.ai-platform/specs/v0.3/tasks.md`
- v0.3 checklist: `.ai-platform/specs/v0.3/checklists/requirements.md`
- v0.3 analysis: `.ai-platform/specs/v0.3/analysis.md`
- Release notes draft: `RELEASE_NOTES.md`
- Release report status: `Released_Local`
- Local release tag: `v0.3.0`
- Remote push and package publication: not performed by this handoff

## Accepted v0.3 Tasks

- T019: Python regression safety net accepted. Evidence: `.ai-platform/evidence/T019/`.
- T020: Adapter boundary refactor accepted. Evidence: `.ai-platform/evidence/T020/`.
- T021: JS alpha init and metadata path accepted. Evidence: `.ai-platform/evidence/T021/`.
- T022: JS runtime path accepted. Evidence: `.ai-platform/evidence/T022/`.
- T023: Consumer Mode guards accepted. Evidence: `.ai-platform/evidence/T023/`.
- T024: JS alpha local command matrix accepted. Evidence: `.ai-platform/evidence/T024/`.
- T025: JS alpha MCP and pack compatibility accepted. Evidence: `.ai-platform/evidence/T025/`.
- T026: Adapter-aware doctor accepted. Evidence: `.ai-platform/evidence/T026/`.
- T027: README and TypeScript boundary docs accepted. Evidence: `.ai-platform/evidence/T027/`.

T028 prepared the release matrix and report and was accepted on 2026-05-13. Version bump, tag creation, remote push and package publication remain separate release handoff decisions.

## Release Candidate Capabilities

- Rust CLI/Core remains the SkillRun implementation boundary.
- Python `action.py` remains the stable blessed action adapter path.
- `skillrun init <name> --python` remains the README main Quickstart path.
- `skillrun init <name> --py` is a short alias for the same Python template.
- `skillrun init <name> --js` generates a JS alpha capsule using canonical ESM `action.mjs`.
- Manifest generation routes through an adapter boundary and records `runtime.adapter`, `runtime.entrypoint`, source hashes, schemas, examples and permissions.
- JS metadata extraction reads explicit `inputSchema` and `outputSchema` exports from `action.mjs`.
- JS runtime execution supports optional `preflight(input, ctx)` and sync or async `run(input, ctx)`.
- Runtime envelopes, run records, artifact containment, permission checks and structured error behavior remain shared across Python and JS alpha.
- Consumer Mode validates static Manifest source hashes before `test`, `run`, `serve --mcp` and `pack`.
- `skillrun serve --mcp` remains Manifest-derived and language-neutral.
- `.skr` packaging remains source + Manifest archive generation.
- `skillrun doctor` reports adapter-aware capsule structure, Manifest freshness, schema/example status and recovery guidance without running business action code.

## Release Matrix

| Capability area | Evidence path | T028 status |
| --- | --- | --- |
| Python baseline | `cargo test`; `tests/e2e_matrix.rs::a001_to_a013_release_matrix_has_fresh_command_evidence` | Passed |
| MCP stdio baseline | `cargo test`; `tests/e2e_matrix.rs::a014_mcp_stdio_release_matrix_exercises_full_client_flow` | Passed |
| JS alpha local command path | `cargo test`; `tests/e2e_matrix.rs::js_alpha_local_command_matrix_covers_init_manifest_inspect_test_and_run` | Passed |
| `--py` alias | `cargo test`; `tests/e2e_matrix.rs::py_alias_manifest_smoke_uses_python_adapter_identity` | Passed |
| JS MCP surfaces | `cargo test`; `tests/mcp_server.rs` JS dry-run and stdio tests | Passed |
| JS `.skr` packaging | `cargo test`; `tests/pack.rs::pack_creates_js_skr_with_action_mjs_manifest_examples_and_no_dependencies_or_run_history` | Passed |
| Adapter-aware diagnostics | `cargo test`; `tests/consumer_guards.rs` doctor coverage | Passed |
| Governance evidence | Delivery artifact validator | Passed |

## Validation Summary

Release-candidate validation completed on 2026-05-13:

- `cargo test`: passed.
- `cargo run -- --version`: passed.
- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`: passed.

Current local binary version output:

```text
skillrun 0.3.0
```

The SemVer bump and local tag are part of the post-T028 release handoff. Remote tag push and package publication still require explicit maintainer approval.

## Known Limitations

- v0.3 supports JS Action Alpha only through canonical ESM `action.mjs`.
- `action.ts` is not a runtime entrypoint. TypeScript authors must compile to `action.mjs` outside SkillRun.
- SkillRun does not run `ts-node`, `tsx`, source maps, CJS/ESM compatibility matrices, or package-manager install flows.
- JS schemas must be explicit JSON Schema exports; no TypeScript type, JSDoc, Zod, TypeBox, example or package metadata inference is provided.
- Runtime commands remain Manifest-only and do not accept language flags.
- MCP transport remains stdio only. HTTP, SSE, Streamable HTTP, auth and hosted server modes are out of scope.
- `.skr` is not signed, not a registry package, not a dependency bundle, and not a reproducible runtime image.
- Dependency installation and runtime environment recreation remain the user's responsibility.
- SkillRun is not an OS sandbox. Running a third-party action still means executing third-party code.
- Consumer Mode avoids dynamic metadata import, but it does not prevent the action process itself from using local machine capabilities once executed.

## Release Decision Checklist

- [x] T019-T027 accepted.
- [x] T028 release matrix and report prepared.
- [x] Maintainer reviews T028 diff and evidence.
- [x] Maintainer accepts v0.3 release candidate.
- [x] Decide whether to bump Cargo/package version from `0.2.0` to `0.3.0`.
- [x] Create local release tag after explicit approval.
- [ ] Push tag to remote, if desired.
- [ ] Publish package/artifact, if desired.

## User Review Gate

- Approval: Granted for T028 on 2026-05-13.
- Reviewer notes: v0.3 release candidate evidence is accepted and the local release tag is created. Remote publication and package release have not been performed.
