# T051 Test Results

## RED 1

Command:

```text
cargo test --test adapter_conformance
```

Result: failed as expected.

Key output:

```text
error: no test target named `adapter_conformance` in default-run packages
```

## RED 2

Command:

```text
cargo test --test adapter_conformance
```

Result: failed as expected after first test draft.

Key output:

```text
missing YAML path input_schema.properties.order_id.type
```

Resolution: corrected the tests to assert the current Manifest IR shape under `schemas.input` and `schemas.output`.

## GREEN

Command:

```text
cargo test --test adapter_conformance
```

Result: passed.

```text
running 3 tests
test js_alpha_adapter_manifest_maps_to_protocol_contract ... ok
test python_stable_adapter_manifest_maps_to_protocol_contract ... ok
test python_and_js_adapters_capture_stdout_as_logs_not_results ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Formatting

Command:

```text
cargo fmt --check
```

Initial result: failed on `tests/adapter_conformance.rs` formatting.

Command:

```text
cargo fmt
```

Result: applied formatting.

Command:

```text
cargo fmt --check
```

Final result: passed.

## Whitespace

Command:

```text
git diff --check
```

Result: passed.

## Full Suite

Command:

```text
cargo test
```

Result: passed.

Summary: full workspace test suite passed, including the new `adapter_conformance` target.
