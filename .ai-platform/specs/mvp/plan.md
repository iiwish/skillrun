# SkillRun MVP Feature Plan

Version: v0.1
Status: Confirmed
Source: `.ai-platform/docs/technology-decision-record.md`
Last updated: 2026-05-11

## Plan Summary

The confirmed plan uses a Rust Cargo binary crate, Manifest-driven Consumer Mode, file-based IPC, structured envelopes, thin MCP exposure, and `.skr` packaging. Python remains only the MVP Action adapter path. The delivery path is split into T001-T011 in `.ai-platform/docs/tasks.md`.

## Technical Decisions

- TDR-001: Rust Core with Python Action adapter.
- TDR-002: Manifest-driven Consumer Mode.
- TDR-003: File-based IPC and structured envelopes.
- TDR-004: Instruction-only Skill guard.
- TDR-005: Repository layout and CLI stack.
- TDR-006: Thin MCP layer.
- TDR-007: Package primitive.
- TDR-008: Layered test strategy and business examples.

## Execution Order

Start with T001, then proceed through dependency-ready tasks one at a time. T002-T011 remain Draft until their packets are created and dependencies are satisfied.

## User Review Gate

- Approval: Approved on 2026-05-11
- Reviewer notes: Document review found and fixed parallel/conflict contradictions before packetization.
