# T033 Test Results

## RED

Command:

```bash
cargo test --test runtime --test errors
```

Result: failed as expected before implementation.

Key failures:

```text
left: String("RuntimeError")
right: "DependencyError"
```

## GREEN

Command:

```bash
cargo test --test runtime --test errors
```

Result: passed.

## Format

Command:

```bash
cargo fmt --check
```

Result: passed.

## Full Validation

Command:

```bash
cargo test
```

Result: passed.

## Lint

Command:

```bash
cargo clippy --all-targets -- -D warnings
```

Result: passed.
