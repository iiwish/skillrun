# T005 Evidence Summary

Task: T005 Implement Run Records And Python Action Adapter IPC Success Path
Status: Accepted
Executor: codex direct execute fallback
Date: 2026-05-12

## Direct Execute Fallback

用户要求继续推进 T005，但没有显式要求 delegation。当前宿主指令要求只有用户明确要求 subagents 时才能创建 subagent，因此 T005 在当前 worktree 中由 Codex direct execute fallback 完成。

## Changed Files

- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/tasks.md`
- `.ai-platform/specs/mvp/analysis.md`
- `.ai-platform/specs/mvp/tasks.md`
- `.ai-platform/specs/mvp/packets/T005.yaml`
- `.ai-platform/evidence/T005/summary.md`
- `.ai-platform/evidence/T005/test-results.md`
- `.ai-platform/evidence/T005/diff.patch`
- `src/main.rs`
- `src/cli.rs`
- `src/runtime.rs`
- `src/run_record.rs`
- `src/adapters/python.rs`
- `tests/cli.rs`
- `tests/runtime.rs`

## Implementation Summary

- 实现 `skillrun test [--cwd <dir>]`，默认使用 `examples/default.input.json`。
- 实现 `skillrun run [--cwd <dir>] --input <file>`，每次创建唯一 run id。
- 每次 run 在 `.skillrun/runs/` 下创建唯一 run 目录，包含 `input.json`、`context.json`、`output.json`、`stdout.log`、`stderr.log`、`artifacts/` 和 `record.json`。
- Python Action adapter 通过 `SKILLRUN_CONTEXT_JSON`、`SKILLRUN_INPUT_JSON`、`SKILLRUN_OUTPUT_JSON` 和 `SKILLRUN_ARTIFACT_DIR` 进行 file-based IPC。
- CLI stdout 输出 machine-readable `ok: true` envelope；adapter stdout/stderr 只写入 log 文件，不作为业务结果。
- `record.json` 记录 run id、mode、status、timestamps、duration、Manifest hash、skill hash、action hash、declared permissions 和 run-local 文件路径。
- 保持 T006/T007/T008 范围独立：未实现完整 structured failure taxonomy、artifact permission enforcement 或 stale Manifest guard。

## TDD Summary

- RED: `cargo test --test runtime` 失败，3 个 runtime tests 因 `test` / `run` 仍返回 `command not implemented yet` 而失败。
- GREEN: 实现 runtime orchestration 后，`cargo test --test runtime` 通过。
- REFACTOR/validation: relative `--cwd` 在 E2E 中暴露 IPC path 问题，已改为 runtime 内部绝对路径并新增回归测试；随后 `cargo test` 和 init/manifest/test/run e2e commands 均通过。

## Review

- Spec compliance: Pass。T005 acceptance points 已由 runtime tests 和 E2E commands 覆盖。
- Bug/code quality: Pass。Core 与 Python adapter 通过文件 IPC 解耦；stdout/stderr 捕获为 logs；run record 持久化独立在 `src/run_record.rs`。
- QA acceptance: Pass。默认 test、显式 run、唯一 run id、run 目录结构、run record 和 stdout log 边界均已验证。
- Rereview: Pass。2026-05-12 复审无 blocking findings，用户要求复审通过后提交并继续 T006。

## Residual Risk

- T005 只实现成功路径和最小错误返回；完整 `ValidationError`、`PolicyViolation`、`ProtocolViolation` 和 `RuntimeError` 分类留到 T006。
- Artifact path 越界检查和 declared permission enforcement 留到 T007。
- Stale Manifest fail-closed guard 留到 T008。
