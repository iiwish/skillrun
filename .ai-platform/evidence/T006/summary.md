# T006 Evidence Summary

Task: T006 Implement Structured Error Envelope Handling
Status: Accepted
Executor: codex direct execute fallback
Date: 2026-05-12

## Direct Execute Fallback

用户要求继续推进 T006，但没有显式要求 delegation。当前宿主指令要求只有用户明确要求 subagents 时才能创建 subagent，因此 T006 在当前 worktree 中由 Codex direct execute fallback 完成。

## Changed Files

- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/tasks.md`
- `.ai-platform/specs/mvp/analysis.md`
- `.ai-platform/specs/mvp/tasks.md`
- `.ai-platform/specs/mvp/packets/T006.yaml`
- `.ai-platform/evidence/T006/summary.md`
- `.ai-platform/evidence/T006/test-results.md`
- `.ai-platform/evidence/T006/diff.patch`
- `src/main.rs`
- `src/errors.rs`
- `src/runtime.rs`
- `src/adapters/python.rs`
- `tests/errors.rs`

## Implementation Summary

- 新增 `src/errors.rs`，集中构造和校验 `ok: false` error envelope。
- CLI 对 runtime failure outcome 输出 JSON envelope，并以 non-zero exit code 返回。
- Python Action adapter 将 Pydantic input validation 映射为 `ValidationError`。
- `preflight` 和业务 `ValueError` 映射为 `PolicyViolation`，并提供 `llm_hint`。
- adapter 缺失或写出非法 output envelope 时，Rust Core 返回 `ProtocolViolation`，不会让 stdout 假装成功。
- 未分类 action failure 映射为 `RuntimeError`，stack trace 保留在 `stderr.log`，不进入 `display.markdown`。
- 保持 T007/T008 范围独立：未实现 artifact permission enforcement 或 stale Manifest guard。

## TDD Summary

- RED: `cargo test --test errors` 失败，4 个 tests 因失败路径没有 JSON envelope 而失败。
- GREEN: 实现 structured envelope 和 adapter exception mapping 后，`cargo test --test errors` 通过。
- REFACTOR/validation: 清理未使用 helper 后，`cargo fmt -- --check` 和 `cargo test` 均通过。

## Review

- Spec compliance: Pass。`ValidationError`、`PolicyViolation`、`ProtocolViolation` 和 `RuntimeError` 均有可复现 tests。
- Bug/code quality: Pass。error envelope 形状集中在 `src/errors.rs`，runtime outcome 明确区分 success/failure。
- QA acceptance: Pass。stdout 假成功、stack trace display 泄漏、run record failed status 均已覆盖。
- Rereview: Pass。2026-05-12 复审无 blocking findings，用户要求复审通过后提交并继续 T007。

## Residual Risk

- `PermissionDenied` 属于 T007 权限与 artifact enforcement，本任务未实现。
- Stale Manifest fail-closed guard 留到 T008。
