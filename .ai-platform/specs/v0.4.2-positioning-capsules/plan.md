# SkillRun v0.4.2 Plan

Version: v0.4.2
Status: Confirmed
Last updated: 2026-05-14

## Decision Summary

- D-042-001: Keep v0.4.2 as a documentation and official-reference-capsule release.
- D-042-002: Preserve the SkillRun core positioning as Manifest-driven SOP-backed Skill Capsule runtime.
- D-042-003: Treat v0.5 Language Agnosticism as future Adapter Protocol work, not a v0.4.2 runtime change.
- D-042-004: Use official reference capsules, not registry/store language.

## Constitution Check

- Rust Core remains unchanged.
- Python remains the stable reference action path.
- Security claims remain honest and avoid OS sandbox promises.
- `.skr` remains source + Manifest archive.

## Risks And Mitigations

- Risk: examples look like a security product. Mitigation: trust model and example docs explicitly state non-goals.
- Risk: diagnostics runner is mistaken for shell. Mitigation: schema uses named diagnostics only.
- Risk: file patcher is mistaken for sandbox. Mitigation: docs and capsule SOP say it is bounded patching, not OS sandboxing.

## Consequences For Tasks

T047 owns the whole v0.4.2 documentation and reference capsule slice because the work is cohesive and low-risk, but it still has bounded allowed files and validation commands.
