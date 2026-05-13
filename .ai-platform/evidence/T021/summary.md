# T021 Evidence Summary

Task: T021 Add Explicit Init Language Flags, `--py` Alias, And JS Templates
Status: Accepted
Date: 2026-05-13

## Scope

Added explicit init language selection for Python stable and JS alpha capsules.

## Changed Files

- `src/cli.rs`
- `src/init.rs`
- `templates/js/SKILL.md`
- `templates/js/action.mjs`
- `templates/js/examples/default.input.json`
- `templates/js/skillrun.config.json`
- `tests/cli.rs`
- `tests/init.rs`

## What Changed

- Added `InitLanguage` and a language-selected `create_capsule` init path.
- Kept `--python` as the stable Python path.
- Added `--py` as an alias for `--python`, with the same generated files and config.
- Added `--js` as a JS alpha init path that generates `action.mjs`, explicit JSON Schema exports, default input, and Node adapter config.
- Rejected conflicting Python/JS language flags.
- Kept language flags scoped to `init`; runtime commands remain Manifest-driven.
- Updated CLI help to label `init --js` as alpha.
- Did not add Node metadata extraction, Node runtime execution, package manager files, TypeScript support, sandbox behavior, registry behavior, or HTTP transport.

## Validation

- RED: `cargo test --test init --test cli` failed before implementation because `init --js` was absent from CLI help and the new init behavior was not implemented.
- `cargo fmt`: passed.
- GREEN: `cargo test --test init --test cli` passed.
- Full suite: `cargo test` passed.

## Review Notes

- Spec compliance: The task stays within the T021 allowed files and implements only init/template behavior.
- Engineering quality: Language selection remains explicit and small; `--py` is not treated as a separate adapter.
- QA acceptance: Python init behavior is preserved, JS alpha files are generated, no package manager files are generated, and missing/conflicting language flags produce clear errors.

## Residual Risk

- JS capsules cannot yet generate a Manifest or run. That is intentional and remains owned by T022/T023/T024.

## Review Decision

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
