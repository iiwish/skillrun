# v0.5.3 Capsule Registry + Switchboard Analysis

**Metadata**

- Version: v0.5.3 analysis
- Status: Completed
- Source spec: `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/spec.md`
- Source plan: `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/plan.md`
- Source tasks: `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/tasks.md`
- Last updated: 2026-05-15

---

## Summary

No Critical or High findings were found. The v0.5.3 scope is coherent if it remains a local state layer and does not expand into Router, `.skr import`, marketplace, or trust.

The user requested v0.5.2 merge/push and continuation on 2026-05-15. Execution may begin with T059 after generating the required execution packet.

## Findings Summary

- Critical: 0
- High: 0
- Medium: 0
- Low: 2

## Requirement Coverage

| Requirement | Covered by task |
| --- | --- |
| FR-001 registry add | T059 |
| FR-002 default disabled | T059 |
| FR-003 no action import | T059, T060 |
| FR-004 registry list JSON | T059 |
| FR-005 registry inspect JSON | T059 |
| FR-006 registry fields | T059 |
| FR-007 remove without deleting files | T059 |
| FR-008 switchboard list JSON | T060 |
| FR-009 enable gates | T060 |
| FR-010 disable no import | T060 |
| FR-011 duplicate id rejection | T059 |
| FR-012 empty registry | T059 |
| FR-013 missing path list tolerance | T059, T060 |

Coverage status: Complete.

## Dependency Review

Task sequence:

```text
T059 -> T060 -> T061
```

Rationale:

- T060 depends on registry storage.
- T061 depends on implemented behavior.
- `src/cli.rs` is shared and makes parallel execution inappropriate.

## Risk Review

### LOW-001: `switchboard` command is explicit but long

Impact:

CLI may feel verbose.

Recommendation:

Keep explicit naming in v0.5.3. Add aliases later only if usage proves friction.

### LOW-002: Add/register and enable semantics need clear output

Impact:

Users may assume registered means enabled or trusted.

Recommendation:

Command output and docs should state `registered`, `enabled`, and `ready` separately.

## Execution Gate

Current state:

- Spec: Confirmed.
- Checklist: Completed.
- Plan: Confirmed.
- Tasks: Confirmed.
- Packets: T059, T060, and T061 generated.

Execution allowed:

- All planned v0.5.3 tasks are completed.

Blocking gate:

- None for local v0.5.3 implementation. Release tag, remote push, package publication, Router, mount profile, and Desktop work remain separate explicit decisions.

## Execution Evidence

### T059

- RED: `cargo test --test registry` failed because `registry` was an unknown command.
- GREEN: `cargo test --test registry` passed with 3 tests.
- Full validation: `cargo test` passed.
- Notes: Registry entries are disabled by default, stored under `SKILLRUN_HOME` when provided, and remove only local registry state without deleting capsule files.

### T060

- RED: focused tests failed because `switchboard` was an unknown command.
- GREEN: `cargo test --test registry --test consumer_guards --test instruction_only` passed.
- Full validation: `cargo test` passed.
- Notes: Enable fails closed for stale Manifest, instruction-only, and dependency-error capsules. Disable only changes local exposure intent.

### T061

- Validation: `cargo fmt --check` passed.
- Validation: `git diff --check` passed.
- Full validation: `cargo test` passed.
- Notes: Docs and release notes now describe registry as local inventory and switchboard as future exposure intent, not marketplace, trust, sandbox, Router, mount profile, or dependency installation.

### Pre-Merge Hardening

- Finding: stale local inventory entries whose registered capsule directory was moved or deleted caused `registry list --json`, `registry inspect <id> --json`, and `switchboard list --json` to fail as whole commands.
- Fix: missing capsule paths are now represented as per-capsule `readiness.status="missing-path"`, `ok=false`, `manifest.present=false`, and a remediation `next_step`; `switchboard enable <id>` remains fail-closed.
- RED: `cargo test --test registry registry_and_switchboard_lists_tolerate_missing_capsule_paths` failed with `cwd does not exist`.
- GREEN: `cargo test --test registry registry_and_switchboard_lists_tolerate_missing_capsule_paths` passed.
