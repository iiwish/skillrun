# SkillRun Official Demo Capsules

This directory contains **demo capsules** that show how SkillRun can wrap external CLI tools and real-world services with SOP-backed contracts, preflight boundaries, and run evidence.

## Difference from `examples/`

| | `examples/` | `demos/` |
|---|---|---|
| **Purpose** | Reference implementations for Core testing and author guidance | Demonstrations of "external CLI SkillRun-ization" |
| **CI** | Part of `cargo test` | **Not part of Core CI**; may require external tools |
| **Dependencies** | Zero external CLI dependencies; self-contained Python/JS | May require external CLIs (e.g., `lark-cli`, `docker`) |
| **Stability** | Must pass on every Core commit | Best-effort; may lag behind external tool updates |

## Rules for Demo Capsules

1. **Each demo must prove a single point**: "This external CLI/service can be constrained by SkillRun contract."
2. **Preflight is mandatory**: Every demo must have hard boundaries (allowlists, dry-run defaults, sensitive content checks).
3. **Artifact is mandatory**: Every demo must generate an audit artifact (preview, receipt, or diagnostic report).
4. **README required**: Each demo directory must explain prerequisites, setup steps, and trust boundaries.
5. **Hard limit**: Official demo capsules should not exceed 6. If a demo's maintenance cost exceeds its value, it should be deprecated.

## Current Demos

| Demo | External Tool | Proves |
|------|---------------|--------|
| `lark_notice_sender` | `lark-cli` | Flysheet/Lark messages can be sent with chat_id allowlists, dry-run preview, and sensitive word checks. |
