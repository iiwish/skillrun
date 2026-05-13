# T029 Test Results

## RED

Command:

```bash
cargo test dependency_error_is_a_valid_structured_error_code
```

Result: failed as expected.

Key failure:

```text
left: Err("unknown error code: DependencyError")
right: Ok(())
```

## GREEN

Command:

```bash
cargo test dependency_error_is_a_valid_structured_error_code
```

Result: passed.

## Target Validation

Command:

```bash
cargo test --test errors --test cli --test consumer_guards
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
