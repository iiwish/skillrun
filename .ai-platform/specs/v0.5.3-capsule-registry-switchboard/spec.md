# SkillRun v0.5.3 Spec: Capsule Registry + Switchboard

**Metadata**

- Feature: v0.5.3 Capsule Registry + Switchboard
- Status: Confirmed
- Date: 2026-05-15
- Source docs: `docs/v0.5.3-capsule-registry-switchboard.md`

---

## User Stories

### US-001: Register Local Capsule

As a SkillRun consumer, I want to register a local capsule path so future tools can list and manage known capsules without scanning arbitrary folders.

### US-002: Review Local Inventory

As a Desktop or automation consumer, I want `registry list --json` and `registry inspect <id> --json` so I can render capsule inventory without parsing terminal text.

### US-003: Enable And Disable Capsules

As a user, I want to enable or disable a registered capsule so future Router exposure can respect an explicit local switchboard state.

### US-004: Fail Closed On Enable

As a user, I want enable to fail when the capsule is stale, instruction-only, unsupported, or not ready, so broken or unsafe-by-contract capsules are not exposed by accident.

## Functional Requirements

- FR-001: `skillrun registry add --cwd <capsule> [--id <id>]` registers a local capsule path.
- FR-002: `registry add` creates entries as disabled by default.
- FR-003: `registry add` does not execute or import action source.
- FR-004: `skillrun registry list --json` emits one parseable JSON object.
- FR-005: `skillrun registry inspect <id> --json` emits one parseable JSON object.
- FR-006: registry JSON includes id, path, source_type, enabled, registered_at, manifest, skill, runtime, tool, and readiness summary where available.
- FR-007: `skillrun registry remove <id>` removes registry state but does not delete capsule files.
- FR-008: `skillrun switchboard list --json` emits enabled/disabled state for registered capsules.
- FR-009: `skillrun switchboard enable <id>` sets enabled true only when Manifest is fresh and readiness is ok.
- FR-010: `skillrun switchboard disable <id>` sets enabled false without executing action source.
- FR-011: duplicate ids are rejected unless a future explicit update command is added.
- FR-012: missing registry file is treated as an empty registry.

## Non-Functional Requirements

- NFR-001: Registry state is local and transparent; it is not a trust registry.
- NFR-002: JSON fields use snake_case.
- NFR-003: Commands must preserve Consumer Mode non-import behavior.
- NFR-004: v0.5.3 must not add Router, daemon, Desktop, MCP client config mutation, `.skr` import, signed packages, dependency installation, or sandbox semantics.
- NFR-005: Storage must support `SKILLRUN_HOME` override for tests and automation.

## Open Decisions

- OD-001: Final command spelling: `switchboard` is explicit but long; shorter aliases can be deferred.
- OD-002: Whether `registry add` should allow stale capsules as disabled entries. Baseline says yes, because inventory and enable are separate actions.
- OD-003: Whether `registry list --json` recomputes readiness every time. Baseline says yes for correctness, with caching deferred.

## Review Gate

The user requested v0.5.2 merge/push and continuation on 2026-05-15. This spec is approved for plan/work graph execution.
