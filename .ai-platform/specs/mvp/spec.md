# SkillRun MVP Feature Spec

Version: v0.1
Status: Confirmed
Source: `docs/mvp.md`
Last updated: 2026-05-11

## Purpose

This feature-scoped spec is an index for the confirmed SkillRun MVP contract. The canonical product contract is `.ai-platform/docs/product-design.md`; the source MVP contract is `docs/mvp.md`.

## Scope

- Rust-first SkillRun runtime with Python Action adapter path.
- `skillrun init`, `manifest`, `inspect`, `test`, `run`, `serve --mcp`, and `pack`.
- Manifest-driven Consumer Mode.
- File-based IPC and structured output/error envelopes.
- Instruction-only Skill guard.
- B001 `refund` hero example and B002-B004 docs-level business examples.

## Requirement Sources

- Functional requirements: `.ai-platform/docs/product-design.md`, section 5.
- Non-functional requirements: `.ai-platform/docs/product-design.md`, section 6.
- Test strategy: `.ai-platform/docs/test-strategy.md`.

## User Review Gate

- Approval: Approved on 2026-05-11
- Reviewer notes: User approved `docs/mvp.md`; later document review cleared TDR, work graph, test strategy, and business examples.
