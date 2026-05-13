# T032 Test Results

## Initial Failure

Command:

```bash
cargo test --test consumer_guards --test manifest
```

Result: failed.

Key failures:

```text
missing pydantic check missing "executable: python required: >=3.10 detected: Python 3.11.0 status: satisfied"
pydantic v1 check missing "package: pydantic required: >=2,<3 detected: 1.10.0 status: unsupported-version"
```

Cause: Windows fake `python.cmd` was not discovered by the runtime probe.

## GREEN

Command:

```bash
cargo test --test consumer_guards --test manifest
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
