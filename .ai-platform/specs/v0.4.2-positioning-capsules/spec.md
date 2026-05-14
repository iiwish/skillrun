# SkillRun v0.4.2 Feature Spec: Positioning And Official Reference Capsules

Version: v0.4.2
Status: Confirmed
Created: 2026-05-14
Updated: 2026-05-14
Source: user requested completion of v0.4.2 documentation and examples on 2026-05-14
Review: Scope approved in conversation for a v0.4.2 branch from `main`; user acceptance of the completed diff remains pending.

## 一句话判断

v0.4.2 应该完成定位文档、愿景/trust model 文档和三个官方参考胶囊，证明 SkillRun 的核心不是泛安全工具，而是 Manifest-bound SOP-backed Skill Capsule runtime。

## Functional Requirements

- FR-042-001: Add public positioning documentation that distinguishes SkillRun from FastMCP, MCP-only tools, agent frameworks, registries and sandboxes.
- FR-042-002: Add a vision document that can be ambitious without changing README current-state claims.
- FR-042-003: Add a trust model document that honestly states current guarantees and non-guarantees.
- FR-042-004: Add official reference capsule documentation for the v0.4.2 examples.
- FR-042-005: Add `commit_message_gate` as a runnable Python stable reference capsule.
- FR-042-006: Add `bounded_file_patcher` as a runnable Python stable reference capsule.
- FR-042-007: Add `readonly_diagnostics_runner` as a runnable Python stable reference capsule.
- FR-042-008: Update README, docs index, business examples, release notes and release report.
- FR-042-009: Add automated coverage proving the reference capsules can be inspected, checked, tested, exposed through MCP dry-run and packed.
- FR-042-010: Bump local package metadata and version expectations to `0.4.2`.

## Non-functional Requirements

- NFR-042-001: Do not change runtime architecture.
- NFR-042-002: Do not add a new adapter or implement v0.5 Language Agnosticism in this release.
- NFR-042-003: Do not claim OS sandboxing, registry trust, signed packages or dependency vendoring.
- NFR-042-004: Official reference capsules must default to low-risk behavior.
- NFR-042-005: All validation must run without network secrets or external services.

## Success Criteria

- `cargo fmt --check` passes.
- `git diff --check` passes.
- `cargo test --test business_examples` passes.
- `cargo test` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- Delivery artifact validator passes for `T047`.

## Out Of Scope

- Registry, marketplace or `skillrun install`.
- Signed `.skr`.
- Sandbox runtime.
- Arbitrary shell execution.
- Language-agnostic Adapter Protocol implementation.
- New MCP transport.
