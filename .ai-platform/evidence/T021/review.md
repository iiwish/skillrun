# T021 Review

Task: T021 Add Explicit Init Language Flags, `--py` Alias, And JS Templates
Date: 2026-05-13
Result: Accepted

## Spec Compliance Review

Passed.

- `skillrun init <name> --py` creates the Python capsule path and does not introduce a separate adapter identity.
- `skillrun init <name> --js` creates the JS alpha skeleton with `action.mjs`, explicit schema exports, `skillrun.config.json`, and example input.
- Missing language flag and conflicting Python/JS flags fail with clear CLI errors.
- Runtime commands remain language-flag-free and Manifest-driven.
- No Node metadata extraction, Node runtime execution, package manager files, TypeScript support, sandbox, registry, or HTTP transport behavior was added.

## Engineering Quality Review

Passed.

- Language selection is represented explicitly with `InitLanguage`.
- Template selection stays local to init behavior.
- CLI help marks `init --js` as alpha, which avoids overstating current support.
- JS template returns structured data and does not rely on stdout as a business result.

## QA Acceptance Review

Passed.

- Targeted validation passed: `cargo test --test init --test cli`.
- Full validation passed: `cargo test`.
- Diff hygiene passed: `git diff --check`.
- Delivery artifact validation passed.

## Findings

Critical: 0
High: 0
Medium: 0
Low: 0

No blocking findings.

## Residual Risk

JS capsules still cannot produce a Manifest or execute. This is expected and remains in T022/T023/T024 scope.
