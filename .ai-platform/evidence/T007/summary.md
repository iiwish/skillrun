# T007 Evidence Summary

Task: T007 Enforce Artifact And Declared Permission Boundaries
Status: Accepted
Executor: codex direct execute fallback
Date: 2026-05-12

## Direct Execute Fallback

用户要求继续推进 T007，但没有显式要求 delegation。当前宿主指令要求只有用户明确要求 subagents 时才能创建 subagent，因此 T007 在当前 worktree 中由 Codex direct execute fallback 完成。

## Changed Files

- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/tasks.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/mvp/analysis.md`
- `.ai-platform/specs/mvp/tasks.md`
- `.ai-platform/specs/mvp/packets/T007.yaml`
- `.ai-platform/evidence/T007/summary.md`
- `.ai-platform/evidence/T007/test-results.md`
- `.ai-platform/evidence/T007/diff.patch`
- `src/main.rs`
- `src/cli.rs`
- `src/config.rs`
- `src/manifest.rs`
- `src/errors.rs`
- `src/permissions.rs`
- `src/runtime.rs`
- `src/adapters/python.rs`
- `tests/artifacts.rs`
- `tests/permissions.rs`

## Implementation Summary

- 新增 `src/permissions.rs`，集中处理 declared env 收集和 artifact path containment。
- `skillrun.config.json` 的 `permissions` 现在会进入 generated Manifest，`permissions.env.read` 可声明允许注入的环境变量。
- runtime 只向 Python Action 子进程注入 Manifest 声明且宿主存在的 env；IPC control env 最后写入，避免被用户声明覆盖。
- Python Action adapter 支持成功结果返回 `{ "output": ..., "artifacts": [...] }`，并继续校验 typed `Output`。
- runtime 对 success envelope 中的 `artifacts` 做相对路径、parent traversal、drive/root prefix、canonical containment 和文件存在性校验。
- artifact 或 env 权限边界失败返回 `PermissionDenied` structured error envelope。

## TDD Summary

- RED: `cargo test --test artifacts --test permissions` 失败。artifact tests 因 adapter 还不支持 artifacts envelope、缺少 `PermissionDenied` 而失败；declared env 注入测试显示 env 仍为 `missing`。
- GREEN: 实现 permission helper、config permissions parsing、adapter artifacts passthrough 和 runtime containment 后，`cargo test --test artifacts --test permissions` 通过。
- REFACTOR/validation: 清理未使用 helper 后，`cargo fmt -- --check` 和 `cargo test` 均通过。

## Review

- Spec compliance: Pass。`../outside.txt`、绝对路径、缺失 artifact 文件均返回 `PermissionDenied`；合法 artifact 被接受。
- Bug/code quality: Pass。路径校验集中在 `src/permissions.rs`，并使用 canonical containment 防止已存在文件经由链接或路径解析逃逸。
- QA acceptance: Pass。undeclared env 不注入，declared env 注入并记录到 run record，IPC env 不会被 declared env 覆盖。
- Rereview: Pass。2026-05-12 复审无 blocking findings，用户要求复审通过后提交并继续 T008。

## Residual Risk

- T007 不提供 OS-level sandbox；Python Action 仍然是被执行代码，不能阻止其自行读取本地文件或访问网络。
- Stale Manifest fail-closed guard 留到 T008。
- MCP exposure 和 `.skr` packaging 留到 T009/T010。
