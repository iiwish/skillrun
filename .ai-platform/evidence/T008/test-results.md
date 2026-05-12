# T008 Test Results

Date: 2026-05-12

## RED

Command: `cargo test --test consumer_guards --test instruction_only`
Result: 按预期失败，exit code 1。
Signal:
- `run_refuses_stale_skill_hash_before_creating_run` 失败，因为修改 `SKILL.md` 后 run 仍成功执行。
- `serve_refuses_stale_action_before_unimplemented_fallback` 失败，因为 serve 直接返回 unimplemented。
- `pack_refuses_stale_config_before_unimplemented_fallback` 失败，因为 pack 直接返回 unimplemented。
- `manifest_refuses_to_infer_actions_from_markdown_or_scripts` 失败，因为错误信息只说 missing action.py。
- `run_serve_and_pack_refuse_instruction_only_skill` 失败，因为 run 只报告 missing Manifest，未说明 instruction-only 和 action.py next step。

## GREEN

Command: `cargo test --test consumer_guards --test instruction_only`
Result: 通过，exit code 0。
Summary:
- Consumer guard tests: 4 passed.
- Instruction-only tests: 4 passed.

## Full Validation

Command: `cargo fmt -- --check`
Result: 通过，exit code 0。

Command: `git diff --check`
Result: 通过，exit code 0。

Command: `cargo test`
Result: 通过，exit code 0。
Summary:
- Artifacts tests: 4 passed.
- CLI tests: 3 passed.
- Consumer guard tests: 4 passed.
- Errors tests: 4 passed.
- Init tests: 4 passed.
- Inspect tests: 3 passed.
- Instruction-only tests: 4 passed.
- Manifest tests: 3 passed.
- Permissions tests: 3 passed.
- Runtime tests: 4 passed.

## Governance Validation

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T008`
Result: 通过，exit code 0。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
Result: 通过，exit code 0。

## Rereview Validation

Command: `cargo fmt -- --check`
Result: 通过，exit code 0。

Command: `git diff --check`
Result: 通过，exit code 0。

Command: `cargo test --test consumer_guards --test instruction_only`
Result: 通过，exit code 0。Consumer guard tests 4 passed；instruction-only tests 4 passed。

Command: `cargo test`
Result: 通过，exit code 0。36 integration tests passed, 0 failed。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T008`
Result: 通过，exit code 0。

Command: `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
Result: 通过，exit code 0。
