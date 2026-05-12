# T006 Test Results

Date: 2026-05-12

## RED

Command: `cargo test --test errors`
Result: 按预期失败，exit code 1。

Signal:
- `invalid_input_returns_validation_error_envelope` 失败，因为 stdout 不是 JSON envelope。
- `preflight_rejection_returns_policy_violation_with_hint` 失败，因为 stdout 不是 JSON envelope。
- `missing_output_returns_protocol_violation_not_stdout_success` 失败，因为 stdout 不是 JSON envelope。
- `uncategorized_action_failure_returns_runtime_error` 失败，因为 stdout 不是 JSON envelope。

## GREEN

Command: `cargo test --test errors`
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
- Errors tests: 4 passed.
- Init tests: 4 passed.
- Inspect tests: 3 passed.
- Manifest tests: 3 passed.
- Runtime tests: 4 passed.

Command: `git diff --check`
Result: 通过，exit code 0。

## Governance Validation

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T006`
Result: 通过，exit code 0。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
Result: 通过，exit code 0。

## Rereview Validation

Command: `cargo fmt -- --check`
Result: 通过，exit code 0。

Command: `git diff --check`
Result: 通过，exit code 0。

Command: `cargo test --test errors`
Result: 通过，exit code 0。4 passed, 0 failed。

Command: `cargo test`
Result: 通过，exit code 0。21 integration tests passed, 0 failed。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T006`
Result: 通过，exit code 0。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
Result: 通过，exit code 0。
