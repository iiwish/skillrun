# SkillRun v0.5.2 Consumer JSON Surface Plan

**Metadata**

- Version: v0.5.2 technical plan
- Status: Confirmed
- Source spec: `.ai-platform/specs/v0.5.2-consumer-json-surface/spec.md`
- Last updated: 2026-05-15
- Review: User requested review, local commit, and continuation on 2026-05-15; plan approved for execution.

---

## Decision Summary

v0.5.2 will add an additive `--json` output mode for existing Consumer Mode commands:

- `skillrun inspect --json`
- `skillrun check --json`
- `skillrun doctor --json`

It will not change default human text output and will not alter `run` / `test` envelope semantics.

## Constitution Check

| Principle | Status | Notes |
| --- | --- | --- |
| Product identity and language | Satisfied | CLI remains `skillrun`; docs use Chinese with stable English protocol terms. |
| Small and hard boundaries | Satisfied | No registry, router, daemon, marketplace, or sandbox added. |
| Rust Core implementation | Satisfied | CLI and report serialization remain in Rust. |
| Consumer Mode non-import boundary | Satisfied | JSON mode must not import action source. |
| stdout/stderr discipline | Satisfied | `run` / `test` continue to use output/error envelope; `--json` applies to report commands only. |
| Testing discipline | Satisfied | Tasks require JSON contract tests and full `cargo test`. |

No constitution violation is required.

## Decisions

### D-001: Add `--json` only to Consumer Mode report commands

`--json` will be supported on `inspect`, `check`, and `doctor`.

Rationale:

- These commands expose contracts and readiness state.
- They are the immediate data source for v0.6 Desktop/Router work.
- Adding JSON to authoring or packaging commands would broaden scope.

### D-002: Keep default human output unchanged

Existing text rendering remains the default for `inspect`, `check`, and `doctor`.

Rationale:

- v0.5.2 must be additive and low-risk.
- Existing tests and user workflows should remain stable.

### D-003: Share readiness JSON schema between `check` and `doctor`

`check --json` and `doctor --json` should serialize the same readiness report shape, differing only in `command`.

Rationale:

- Future Desktop and Router code should not maintain two readiness models.
- Current `check` and `doctor` already share `readiness::evaluate`.

### D-004: Create a structured inspect report model

`inspect --json` needs a structured report model rather than serializing existing human text.

Rationale:

- Human `inspect` is formatted for terminals.
- Desktop needs fields for status, Manifest, skill, runtime, permissions, examples, preflight, and tool metadata.

### D-005: Leave `run` and `test` unchanged

`run` and `test` already output output/error envelope JSON and should not receive a v0.5.2 wrapper.

Rationale:

- Wrapping would risk breaking existing CLI/MCP semantics.
- Run record query/index is a separate future feature.

## Alternatives Considered

### Alternative A: Add `--json` to every command

Rejected for v0.5.2.

Reason:

- It would pull authoring, packaging, and error-envelope design into a patch release.
- It does not directly serve v0.6 Consumer Mode prerequisites.

### Alternative B: Make JSON the default output

Rejected.

Reason:

- It would break human CLI compatibility and existing tests.

### Alternative C: Implement a local daemon first

Rejected for v0.5.2.

Reason:

- A daemon needs a stable data contract first.
- CLI JSON is the smaller headless foundation.

## Risks

| Risk | Severity | Mitigation |
| --- | --- | --- |
| JSON model drifts from human readiness behavior | Medium | Reuse existing readiness report data and tests. |
| `inspect --json` duplicates too much parsing logic | Medium | Extract structured report generation and render human text from the same or adjacent helpers where practical. |
| JSON mode accidentally imports source | High | Add stale/source-modified tests for JSON mode. |
| CLI parser becomes repetitive | Low | Use small per-command `json` flags first; avoid broad parser refactor. |
| Error paths are not machine-readable | Low | Explicitly out of scope for v0.5.2; keep stderr behavior. |

## Mitigations

- Add JSON contract tests for each command.
- Preserve existing text tests.
- Run focused tests first, then `cargo test`.
- Keep field names stable and snake_case.
- Do not change Manifest schema or runtime behavior.

## Supporting Artifacts

- Spec: `.ai-platform/specs/v0.5.2-consumer-json-surface/spec.md`
- Checklist: `.ai-platform/specs/v0.5.2-consumer-json-surface/checklists/requirements.md`
- Public docs: `docs/v0.5.2-consumer-json-surface.md`
- Tasks: `.ai-platform/specs/v0.5.2-consumer-json-surface/tasks.md`

## Consequences For Tasks

- Tasks should be split by report surface: inspect first, readiness second, docs/validation last.
- `src/cli.rs` is a shared file and creates sequencing conflicts between implementation tasks.
- Behavior changes require RED-GREEN-REFACTOR tests unless explicitly waived by user.
