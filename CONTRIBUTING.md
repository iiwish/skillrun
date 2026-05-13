# Contributing to SkillRun

Thank you for helping make SkillRun sharper, safer, and easier to trust. SkillRun is intentionally narrow: one SOP plus one action becomes a Manifest-driven Agent skill capsule.

## Project Principles

- Use `SkillRun` for the project name and `skillrun` for the CLI, crate, commands, and code identifiers.
- Keep the SkillRun core in Rust.
- Treat Python as the first action adapter target, not as the implementation language for the core.
- Keep public docs clear about what exists today versus what is planned.
- Preserve fail-closed runtime behavior: no implicit execution of instruction-only skills, no stdout success fallback, and no source-code metadata import in consumer mode.

## Development Setup

Install a stable Rust toolchain, then run:

```bash
cargo test
cargo run -- --help
```

Useful local checks:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Branches and Commits

- Use `codex/` prefixed branches for Codex-managed work.
- Keep `main` stable and merge through a reviewed branch.
- Use Conventional Commits, for example `feat(manifest): add adapter metadata` or `docs(release): clarify rc gates`.
- Keep each commit focused on one intent. Code, tests, and docs for that intent can live together.

## Pull Requests

Before opening a pull request:

- Rebase or merge from the current `main`.
- Run the checks that match the change scope.
- Update README, docs, release notes, or examples when behavior changes.
- Call out behavior boundaries and security implications when the runtime contract changes.
- Include evidence for new runtime behavior, especially fixtures, command output, or test names.

Small documentation fixes can be lightweight. Runtime changes should include tests that show the intended behavior and the failure mode.

## Review Standard

Reviews prioritize correctness, fail-closed behavior, user-visible contracts, maintainability, and release clarity. A change is ready when a future maintainer can understand why it exists, how it was validated, and what risk remains.
