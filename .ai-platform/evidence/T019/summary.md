# T019 Evidence Summary

Task: T019 Add Python Path Characterization Tests
Status: Accepted
Date: 2026-05-13

## Scope

Added a Python-path characterization test before adapter boundary refactor work begins.

## Changed Files

- `tests/runtime.rs`

## What Changed

- Added `runtime_rejects_non_python_adapter_before_creating_run_records`.
- The test mutates a generated Python capsule Manifest from `adapter: python` to `adapter: node`.
- It verifies current v0.2 runtime behavior: unsupported adapter fails with `unsupported runtime adapter: node` before `.skillrun/runs` is created.

## Validation

- `cargo test --test manifest --test runtime --test e2e_matrix --test consumer_guards`: passed.

## Review Notes

- No production code changed.
- No JS implementation was introduced.
- The test intentionally captures current unsupported-adapter behavior so T020/T023 can refactor it deliberately.

## Residual Risk

- This characterization test will need to be revised when Node runtime support lands.

## Review Decision

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
