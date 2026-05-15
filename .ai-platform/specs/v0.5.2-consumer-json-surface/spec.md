# SkillRun v0.5.2 Consumer JSON Surface Spec

**Metadata**

- Version: v0.5.2 planning baseline
- Status: Confirmed
- Source: User requested v0.5.2 design and development after merging v0.5.1 into `main`
- Last updated: 2026-05-15
- Review: User requested review and continuation on 2026-05-15; spec approved for Plan / Work Graph.

---

## Product Positioning

v0.5.2 turns existing Consumer Mode inspection and readiness data into a stable machine-readable CLI surface. It is the first headless layer for future Desktop, Router, and automation workflows.

It does not introduce registry, router, daemon, Tauri UI, package trust, or new sandbox behavior.

## Target Users

- `US-001` Desktop / UI integrator: needs structured capsule status without parsing terminal text.
- `US-002` Automation script author: needs deterministic readiness data and exit codes.
- `US-003` SkillRun maintainer: needs a small compatibility surface that does not destabilize human CLI output.
- `US-004` Future router implementer: needs a stable way to decide which capsules are inspectable and ready before exposure.

## User Stories And Scenarios

### US-001: Inspect a Capsule for UI Display

As a Desktop client, I want to call `skillrun inspect --json --cwd path/to/capsule` so I can render the capsule contract, schema presence, runtime, permissions, examples, preflight marker, and MCP tool metadata without parsing text.

### US-002: Check Host Readiness

As an automation script, I want to call `skillrun check --json --cwd path/to/capsule` so I can determine whether the current machine can consume or run the capsule and explain missing dependencies.

### US-003: Keep Human CLI Stable

As a maintainer, I want `--json` to be additive so existing text output and tests remain stable.

### US-004: Reuse Readiness Data

As a future router implementer, I want `doctor --json` and `check --json` to share a readiness schema so I do not maintain two integration models.

## Core User Journey

1. A consumer receives or creates a Skill Capsule.
2. An automation or UI calls `skillrun inspect --json --cwd path/to/capsule`.
3. The caller renders status, runtime, permissions, examples, and preflight signal.
4. The caller runs `skillrun check --json --cwd path/to/capsule`.
5. If `ok=true`, the caller can proceed to a future registry/router flow.
6. If `ok=false`, the caller renders `status`, `reason`, `dependency_checks`, `source_checks`, `example_checks`, and `next_step`.

## Functional Requirements

- `FR-001`: `inspect` accepts `--json` and emits one valid JSON object to stdout.
- `FR-002`: `inspect --json` supports `runnable`, `invalid-runnable`, and `instruction-only` states.
- `FR-003`: `inspect --json` does not import or execute action source.
- `FR-004`: `check` accepts `--json` and emits one valid JSON object to stdout.
- `FR-005`: `check --json` preserves existing exit code semantics: `ok=true` exits 0, `ok=false` exits non-zero.
- `FR-006`: `doctor` accepts `--json` and emits the same readiness schema as `check --json`, with `command="doctor"`.
- `FR-007`: Default human output for `inspect`, `check`, and `doctor` remains unchanged.
- `FR-008`: JSON fields use stable snake_case names.
- `FR-009`: `test` and `run` are documented as already producing standard output/error envelope JSON and are not wrapped in v0.5.2.
- `FR-010`: JSON contract tests cover success and representative failure states.

## Non-Functional Requirements

- `NFR-001 Reliability`: JSON output must always be parseable when command execution reaches a report state.
- `NFR-002 Compatibility`: Existing text output behavior and tests must continue to pass.
- `NFR-003 Security`: Consumer JSON commands must not import untrusted action source for metadata.
- `NFR-004 Maintainability`: `check --json` and `doctor --json` should reuse the existing readiness model rather than duplicating logic.
- `NFR-005 UX`: JSON should be useful for Desktop and automation without requiring knowledge of terminal formatting.

## Functional Scope

In scope:

- CLI parsing for `--json` on `inspect`, `check`, and `doctor`.
- Structured report types or serde serialization for inspect and readiness data.
- Tests for JSON output and non-import behavior.
- Public documentation for v0.5.2.

Out of scope:

- Registry or capsule import commands.
- Enable / disable switchboard.
- Router or MCP mount profiles.
- Local daemon.
- Tauri/Desktop implementation.
- Run record query/index.
- Machine-readable CLI errors for parser or filesystem failures.
- Changes to Manifest schema.
- Changes to `run` / `test` envelope semantics.

## Edge Cases

- `EC-001`: `inspect --json` on instruction-only Skill returns status `instruction-only` with missing file details.
- `EC-002`: `inspect --json` on stale Manifest returns status `invalid-runnable` and includes `reason`.
- `EC-003`: `check --json` on missing Manifest returns `ok=false`, `status="missing-manifest"` or related readiness status.
- `EC-004`: `check --json` on dependency failure returns dependency checks and non-zero exit.
- `EC-005`: `doctor --json` on instruction-only Skill returns the same readiness schema as check.
- `EC-006`: Unknown options still fail with stderr and non-zero exit; v0.5.2 does not introduce JSON parser error envelopes.

## Constraints And Assumptions

- The project uses Rust Core and serde/serde_json are already available.
- Current `run` and `test` stdout is already JSON envelope and should not be wrapped.
- Desktop will initially call CLI commands, not a daemon API.
- v0.5.2 should be a narrow additive release.
- User approval is required before implementation tasks are generated or executed.

## Data Or Integration Needs

Expected JSON surfaces:

- Inspect report object.
- Readiness report object shared by check and doctor.

No external API integration is required.

## Success Criteria

- `cargo test` passes.
- Existing human output tests pass without updating expected behavior except where tests add JSON mode.
- New JSON tests parse stdout with `serde_json`.
- JSON mode does not import action source in stale / modified action tests.
- v0.5.2 docs clearly state non-goals and v0.6 relationship.

## Acceptance Criteria

- `skillrun inspect --json --cwd a generated Python capsule` exits 0 and includes `status="runnable"`, `runtime.adapter="python"`, and schema presence.
- `skillrun inspect --json` on stale source exits 0 and includes `status="invalid-runnable"`.
- `skillrun inspect --json` on instruction-only skill exits 0 and includes `status="instruction-only"`.
- `skillrun check --json --cwd a generated Python capsule` exits 0 and includes `ok=true`.
- `skillrun check --json` on dependency failure exits non-zero and includes `ok=false`.
- `skillrun doctor --json` outputs the same readiness data shape as `check --json`.
- Human `inspect`, `check`, and `doctor` outputs remain compatible with existing tests.

## Clarifications

- 2026-05-15: User requested v0.5.1 merge to `main`, branch creation for v0.5.2, and start of v0.5.2 design/development using `ai-delivery-governor`.
- 2026-05-15: Current governed decision is to create a reviewable v0.5.2 spec before implementation because `--json` changes public CLI contracts.
- 2026-05-15: User asked to review related documents and continue if there were no issues. Review found no blocking issue, so the spec is treated as approved for planning.

## Open Questions

- Should `--json` be added to `manifest` and `pack` in a later release, or kept limited to Consumer Mode commands?
- Should parser/filesystem errors eventually support a machine-readable CLI error envelope?
- Should run record query/index be v0.5.3 or delayed until Router work starts?
