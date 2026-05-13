# T030 Test Results

## RED

Command:

```bash
cargo test --test manifest
```

Result: failed as expected before implementation.

Key failure:

```text
missing YAML path runtime.requirements.executable.name
```

## GREEN

Command:

```bash
cargo test --test manifest --test pack
```

Result: passed.

## Format

Command:

```bash
cargo fmt --check
```

Result: initially failed on import ordering in `tests/manifest.rs`.

Command:

```bash
cargo fmt
cargo fmt --check
```

Result: passed.

## Full Validation

Command:

```bash
cargo test
```

Result: passed.

## Diff Hygiene

Command:

```bash
git diff --check
```

Result: passed.
