# Commit Message Gate

## Purpose

This SkillRun capsule validates a proposed Git commit message before an agent
uses it. It is designed to move Conventional Commits rules out of fragile
prompts and into a typed, testable preflight boundary.

## SOP

1. Accept only concise Conventional Commits subject lines.
2. Require one of the supported types: `feat`, `fix`, `docs`, `style`,
   `refactor`, `perf`, `test`, `chore`, `build`, `ci` or `revert`.
3. Require `type: summary` or `type(scope): summary` with exactly one space
   after the colon.
4. Keep the subject at or below 70 characters.
5. Reject multiline messages in this capsule. Longer release notes belong in a
   separate artifact or PR body, not the commit subject.
6. Do not stage files automatically.

## Execution Boundary

By default this capsule only validates and returns a normalized subject. If
`perform_commit` is set to `true`, it may run `git commit -m <message>` for
already staged changes only. It never runs `git add .`.

## Required Context

- `message`: proposed commit subject.
- `perform_commit`: defaults to `false`.

## Recovery Guidance

If this capsule returns `PolicyViolation`, rewrite the commit subject according
to the `llm_hint` and retry. If it returns `DependencyError`, inspect the local
Git repository state before retrying.

## Prohibited Behavior

- Do not auto-stage files.
- Do not include multiline explanations in the commit subject.
- Do not bypass a rejected message by calling Git directly.
