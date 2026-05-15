# v0.5.4 Core Stabilization Audit Analysis

**Metadata**

- Version: v0.5.4 analysis
- Status: In_Progress
- Audit date: 2026-05-15
- Branch: `codex/v0.5.4-core-stabilization-audit`
- Scope: whole `skillrun` project as Core
- Public doc: `docs/v0.5.4-core-stabilization-audit.md`

---

## Summary

v0.5.4 should be a stabilization release, not a feature expansion release.

No evidence suggests the project architecture is fundamentally wrong. The main risk is that several runtime and consumer contracts are strong in narrative but not yet fully enforced in Core.

## Findings Summary

- Critical: 0
- P0: 2
- P1: 3
- P2: 1

## P0 Findings

### P0-001: command adapter readiness executes arbitrary configured executable

Evidence:

- `src/readiness.rs:445`
- `src/readiness.rs:446`

Risk:

`check`, `doctor`, and `switchboard enable` may run `<manifest command executable> --version` for Level 0 command capsules. For arbitrary third-party command capsules, this is an implicit execution surface inside what users expect to be readiness inspection.

Required fix:

- Use non-executing executable presence detection for `runtime.adapter="command"`.
- Keep version execution only for blessed runtime probes such as Python/Node.
- Add regression test with a command executable that would create a marker if executed during `check`.

Execution evidence:

- RED: `cargo test --test consumer_guards command_adapter_readiness_probe -- --nocapture` failed because command readiness executed the fake command probe.
- GREEN: `cargo test --test consumer_guards command_adapter_readiness_probe -- --nocapture` passed.
- Validation: `cargo test --test consumer_guards --test runtime --test registry` passed.

### P0-002: Core does not enforce Manifest input/output JSON schemas

Evidence:

- `src/config.rs:306`
- `src/config.rs:308`
- `src/config.rs:312`
- `src/runtime.rs:295`
- `src/runtime.rs:312`
- `src/runtime.rs:313`

Risk:

For `command` adapter, static schemas can be published through Manifest and MCP without being enforced by Core. This weakens `typed input/output` from runtime contract to documentation.

Required fix:

- Validate input before adapter launch.
- Validate successful output envelope against Manifest `schemas.output`.
- Return `ValidationError` for bad input and `ProtocolViolation` for bad adapter output.

Execution evidence:

- RED: `cargo test --test runtime command_adapter_invalid -- --nocapture` exposed that invalid command adapter input/output were not enforced by Core before the fix.
- GREEN: `cargo test --test runtime command_adapter_invalid -- --nocapture` passed.
- MCP validation: `cargo test --test mcp_server mcp_stdio_invalid_command_input_returns_validation_error_before_adapter_launch -- --nocapture` passed.
- Focused validation: `cargo test --test runtime --test errors --test mcp_server` passed.
- Full validation: `cargo fmt --check`, `git diff --check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` passed.

## P1 Findings

### P1-001: registry/switchboard list still fail as a whole for corrupt Manifest entries

Evidence:

- `src/registry.rs:155`
- `src/registry.rs:161`
- `src/registry.rs:162`
- `src/registry.rs:232`
- `src/registry.rs:238`
- `src/registry.rs:239`
- `src/registry.rs:330`
- `src/registry.rs:332`
- `src/registry.rs:428`

Risk:

A single registered capsule with unreadable or malformed Manifest can poison inventory rendering. Desktop needs list commands to be robust control-plane facts.

Required fix:

- Convert corrupt Manifest/readiness errors into per-capsule readiness states for list/inspect.
- Keep `switchboard enable` fail-closed.

Execution evidence:

- RED: `cargo test --test registry invalid_manifest -- --nocapture` failed because a corrupt Manifest made `registry list --json` exit instead of returning inventory.
- GREEN: `cargo test --test registry invalid_manifest -- --nocapture` passed.
- Focused validation: `cargo test --test registry` passed.
- Full validation: `cargo fmt --check`, `git diff --check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` passed.

### P1-002: Consumer JSON contracts are tested but not frozen as fixtures

Evidence:

- `cargo test -- --list` reports broad coverage.
- JSON tests assert selected fields in Rust tests, but there are no schema/golden fixture files for Desktop-facing contracts.

Risk:

Desktop will depend on these fields across repositories. Without fixtures or schema docs, accidental field drift becomes likely.

Required fix:

- Add stable JSON fixture/golden contract files for Consumer JSON and registry/switchboard JSON.
- Document additive vs breaking JSON changes.

### P1-003: version layers are not explicit enough for stable release

Evidence:

- `Cargo.toml:3`
- `src/manifest.rs:127`
- `src/manifest.rs:128`
- `src/manifest.rs:154`

Risk:

Docs discuss v0.5.3/v0.5.4 while binary/package version remains `0.5.0`; Manifest and IPC versions remain `0.1.0`. These may be valid, but they need explicit semantics before Desktop and `.skr` consumption expand.

Required fix:

- Document binary version, release line, Manifest IR version, and adapter protocol version separately.
- Decide whether v0.5.4 updates `Cargo.toml` or remains an unreleased integration line.

## P2 Findings

### P2-001: MCP result text is not the right source for Envelope Explorer

Evidence:

- `src/mcp.rs:369`
- `src/mcp.rs:377`
- `src/mcp.rs:394`

Risk:

MCP text projection is useful for agents but insufficient as an audit/event source. Desktop should not parse MCP text to reconstruct envelopes.

Required fix:

- Document that Desktop Envelope Explorer consumes run records or future Core API, not MCP text.
- Defer MCP shape changes unless required by a later protocol decision.

## Validation Evidence

- `cargo test -- --list` passed and enumerated the current test matrix.
- `cargo clippy --all-targets -- -D warnings` passed.

## Gate Recommendation

Do not start Desktop implementation against this Core yet.

Allowed next step:

- Execute v0.5.4 P0 fixes in this branch.

Blocked until P0 completion:

- Treating Consumer JSON and registry/switchboard output as Desktop-stable contracts.
- Starting a separate Desktop repo that assumes the current Core surface is final.
