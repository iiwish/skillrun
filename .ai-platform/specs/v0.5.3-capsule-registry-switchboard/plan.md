# SkillRun v0.5.3 Plan: Capsule Registry + Switchboard

**Metadata**

- Version: v0.5.3 technical plan
- Status: Confirmed
- Source spec: `.ai-platform/specs/v0.5.3-capsule-registry-switchboard/spec.md`
- Last updated: 2026-05-15
- Review: User requested v0.5.2 merge/push and continuation on 2026-05-15; plan approved for sequenced execution.

---

## Decision Summary

v0.5.3 will add a local registry and switchboard state layer:

- `skillrun registry add/list/inspect/remove`
- `skillrun switchboard list/enable/disable`
- JSON-first state output for automation and future Desktop/Router consumers

It will not import `.skr`, expose MCP tools, mutate MCP client configs, or introduce trust/sandbox claims.

## Core Decisions

### D-001: Registry Stores Inventory, Not Trust

Registry entries identify local capsule paths and state. They do not certify that a capsule is trusted.

### D-002: Enabled Means Exposure Intent

`enabled=true` means future Router may expose the capsule. It does not mean the action is sandboxed or trusted.

### D-003: Enable Fails Closed

Enable must fail unless the capsule passes static Consumer Mode checks and readiness is ok.

### D-004: Registry Add Defaults Disabled

Registration and exposure are separate actions. This prevents `add` from becoming implicit exposure.

### D-005: Storage Uses `SKILLRUN_HOME`

Tests and automation need deterministic state. v0.5.3 should resolve registry storage from `SKILLRUN_HOME`, with user home fallback.

## Risks

| Risk | Severity | Mitigation |
| --- | --- | --- |
| Registry is misunderstood as marketplace | High | Docs and command output must say local registry only. |
| Enabled is misunderstood as trusted | High | Use wording "exposure intent"; enable still gates on readiness. |
| State file corruption breaks all commands | Medium | Atomic write or safe temp-file replace in implementation. |
| Readiness recompute slows list | Low | Accept for v0.5.3 correctness; caching deferred. |
| Command surface expands too far | Medium | No `.skr import`, no router, no mount profiles in v0.5.3. |

## Validation Strategy

- Unit tests for registry file load/save and id validation.
- CLI tests using `SKILLRUN_HOME` temp directory.
- Consumer guard tests proving enable fails on stale/instruction-only/dependency-error.
- Full `cargo test`.

## Consequences For Tasks

Tasks should be sequenced:

```text
T059 -> T060 -> T061
```

`src/cli.rs` and registry state are shared, so tasks should not run in parallel.
