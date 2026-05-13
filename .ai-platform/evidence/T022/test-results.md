# T022 Test Results

Date: 2026-05-13

## Commands

```text
cargo fmt
cargo test --test manifest --test consumer_guards
cargo test
```

Result: Passed

## TDD Evidence

RED:

```text
cargo test --test manifest --test consumer_guards
```

Result: Failed as expected before implementation. The new JS Consumer Mode test could not generate a Manifest because metadata dispatch returned `unsupported metadata adapter: node`.

GREEN:

```text
cargo test --test manifest --test consumer_guards
```

Result: Passed

Targeted test summary:

```text
consumer_guards: 6 passed
manifest: 11 passed
```

Full test suite summary:

```text
artifacts: 4 passed
business_examples: 2 passed
cli: 3 passed
consumer_guards: 6 passed
e2e_matrix: 2 passed
errors: 4 passed
init: 7 passed
inspect: 3 passed
instruction_only: 4 passed
manifest: 11 passed
mcp_server: 7 passed
pack: 4 passed
permissions: 3 passed
runtime: 5 passed
```
