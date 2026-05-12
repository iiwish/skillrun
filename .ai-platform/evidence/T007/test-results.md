# T007 Test Results

Date: 2026-05-12

## RED

Command: `cargo test --test artifacts --test permissions`
Result: 按预期失败，exit code 1。
Signal:
- `artifact_parent_traversal_returns_permission_denied` 失败，因为当前实现返回 `ProtocolViolation` 而不是 `PermissionDenied`。
- `absolute_artifact_path_returns_permission_denied` 失败，因为当前实现返回 `ProtocolViolation` 而不是 `PermissionDenied`。
- `missing_artifact_file_returns_permission_denied` 失败，因为当前实现返回 `ProtocolViolation` 而不是 `PermissionDenied`。
- `valid_declared_artifact_is_accepted` 失败，因为 adapter 把 `{ output, artifacts }` 当作 typed `Output` 校验。

Command: `cargo test --test permissions`
Result: 按预期失败，exit code 1。
Signal:
- `declared_env_is_injected_and_recorded` 失败，因为 declared env 尚未从 Manifest 注入 action process。
- `undeclared_env_is_not_injected_into_action_process` 已通过，证明 `env_clear` 的基础隔离已经存在。
- `declared_env_cannot_override_ipc_envs` 已通过，当前 IPC env 写入顺序未暴露覆盖问题。

## GREEN

Command: `cargo test --test artifacts --test permissions`
Result: 通过，exit code 0。
Summary:
- Artifact tests: 4 passed.
- Permission tests: 3 passed.

## Full Validation

Command: `cargo fmt -- --check`
Result: 通过，exit code 0。

Command: `cargo test`
Result: 通过，exit code 0。
Summary:
- Artifacts tests: 4 passed.
- CLI tests: 3 passed.
- Errors tests: 4 passed.
- Init tests: 4 passed.
- Inspect tests: 3 passed.
- Manifest tests: 3 passed.
- Permissions tests: 3 passed.
- Runtime tests: 4 passed.

Command: `git diff --check`
Result: 通过，exit code 0。

## Governance Validation

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T007`
Result: 通过，exit code 0。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
Result: 通过，exit code 0。

## Rereview Validation

Command: `cargo fmt -- --check`
Result: 通过，exit code 0。

Command: `git diff --check`
Result: 通过，exit code 0。

Command: `cargo test --test artifacts --test permissions`
Result: 通过，exit code 0。Artifact tests 4 passed；permission tests 3 passed。

Command: `cargo test`
Result: 通过，exit code 0。28 integration tests passed, 0 failed。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T007`
Result: 通过，exit code 0。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
Result: 通过，exit code 0。
