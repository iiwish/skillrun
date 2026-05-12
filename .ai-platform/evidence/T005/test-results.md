# T005 Test Results

Date: 2026-05-12

## RED

Command: `cargo test --test runtime`
Result: 按预期失败，exit code 1。

Signal:
- `test_command_uses_default_example_and_writes_run_record` 失败，因为 `test` 返回 `command not implemented yet`。
- `run_command_uses_explicit_input_and_unique_run_ids` 失败，因为 `run` 返回 `command not implemented yet`。
- `adapter_stdout_is_captured_as_log_not_result` 失败，因为 `test` 返回 `command not implemented yet`。

## GREEN

Command: `cargo test --test runtime`
Result: 通过，exit code 0。

Summary:
- 4 passed.
- 0 failed.

## Full Validation

Command: `cargo fmt -- --check`
Result: 通过，exit code 0。

Command: `cargo test`
Result: 通过，exit code 0。

Summary:
- CLI tests: 3 passed.
- Init tests: 4 passed.
- Inspect tests: 3 passed.
- Manifest tests: 3 passed.
- Runtime tests: 4 passed.

Command: `git diff --check`
Result: 通过，exit code 0。

## E2E Validation

Command: `cargo run -- init refund --python --output tmp/e2e-runtime`
Result: 通过，exit code 0。

Command: `cargo run -- manifest --cwd tmp/e2e-runtime/refund`
Result: 通过，exit code 0。

Command: `cargo run -- test --cwd tmp/e2e-runtime/refund`
Result: 通过，exit code 0。

Command: `cargo run -- run --cwd tmp/e2e-runtime/refund --input examples/default.input.json`
Result: 通过，exit code 0。

关键输出：

```json
{
  "ok": true,
  "output": {
    "decision": "approved",
    "amount": 120
  },
  "run_id": "run-..."
}
```

## Governance Validation

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T005`
Result: 通过，exit code 0。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
Result: 通过，exit code 0。
