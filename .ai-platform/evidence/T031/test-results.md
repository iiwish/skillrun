# T031 Test Results

## RED

Command:

```bash
cargo test --test cli --test consumer_guards --test instruction_only
```

Result: failed as expected before implementation.

Key failure:

```text
help output should list planned command: check
```

## GREEN

Command:

```bash
cargo test --test cli --test consumer_guards --test instruction_only
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
