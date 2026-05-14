# SkillRun v0.4.2 Work Graph: Positioning And Official Reference Capsules

Version: v0.4.2
Status: Ready_For_User_Review
Source spec: `.ai-platform/specs/v0.4.2-positioning-capsules/spec.md`
Last updated: 2026-05-14
Review: User requested completion of this v0.4.2 slice on 2026-05-14; final acceptance is pending.

## Epic E006: Positioning And Reference Capsule Adoption Path

Goal:
Ship a documentation and reference-capsule slice that sharpens SkillRun's positioning without expanding the runtime architecture.

### T047: Add v0.4.2 Positioning Docs And Official Reference Capsules

Status: Needs_Review
Priority: P0
Depends on: v0.4.1 merged to `main`
Blocks: v0.4.2 user acceptance
Story / Requirement: FR-042-001 through FR-042-010, NFR-042-001 through NFR-042-005
Parallel: No
Conflicts with: release docs and business examples

Goal:
Add v0.4.2 positioning/trust docs, three official reference capsules, release notes, version bump and automated coverage.

Allowed files:
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `README.zh-CN.md`
- `RELEASE_NOTES.md`
- `docs/**`
- `examples/commit_message_gate/**`
- `examples/bounded_file_patcher/**`
- `examples/readonly_diagnostics_runner/**`
- `tests/business_examples.rs`
- version expectation tests under `tests/**`
- `.ai-platform/specs/v0.4.2-positioning-capsules/**`
- `.ai-platform/evidence/T047/**`
- `.ai-platform/docs/release-report.md`

Test targets:
- `tests/business_examples.rs`
- version expectation tests in the existing suite

Deliverables:
- Positioning, vision, trust model and v0.4.2 official capsule docs.
- Three runnable Python reference capsules.
- README, release notes, business catalog and release report updates.
- Automated coverage for reference capsules.

Acceptance criteria:
- Reference capsules run through manifest, inspect, check, test, MCP dry-run and pack.
- Docs do not claim registry, marketplace, OS sandboxing, arbitrary shell or new adapter architecture.
- Local package version reports `0.4.2`.

Definition of Done:
- `cargo fmt --check` passes.
- `git diff --check` passes.
- `cargo test --test business_examples` passes.
- `cargo test` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- Delivery artifact validator passes for `T047`.

Validation commands:
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test business_examples`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T047`

TDD plan:
- RED: Add business example coverage for v0.4.2 reference capsules.
- GREEN: Implement reference capsules and docs until tests pass.
- REFACTOR: Keep docs and examples scoped to v0.4.2 without runtime architecture changes.

Packet path:
- `.ai-platform/specs/v0.4.2-positioning-capsules/packets/T047.yaml`

Evidence required:
- `.ai-platform/evidence/T047/summary.md`
- `.ai-platform/evidence/T047/test-results.md`
- `.ai-platform/evidence/T047/diff.patch`

## User Review Gate

- Approval: final user acceptance pending.
- Reviewer notes: Work is ready for user review; merge, tag and publication remain separate decisions.
