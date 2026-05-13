# T023 Test Results

Date: 2026-05-13

## Commands

```text
cargo fmt
cargo test --test runtime --test errors --test artifacts
cargo test
```

Result: Passed

## TDD Evidence

RED:

```text
cargo test --test runtime --test errors --test artifacts
```

Result: Failed as expected before implementation. The new JS artifact/runtime path could not produce a JSON envelope because the Node runtime adapter was not implemented yet.

GREEN:

```text
cargo test --test runtime --test errors --test artifacts
```

Result: Passed

Targeted test summary:

```text
artifacts: 5 passed
errors: 9 passed
runtime: 8 passed
```

Full test suite summary:

```text
artifacts: 5 passed
business_examples: 2 passed
cli: 3 passed
consumer_guards: 6 passed
e2e_matrix: 2 passed
errors: 9 passed
init: 7 passed
inspect: 3 passed
instruction_only: 4 passed
manifest: 11 passed
mcp_server: 7 passed
pack: 4 passed
permissions: 3 passed
runtime: 8 passed
```
