# T004 Evidence Summary

Task: T004 Render Inspect Output And Instruction-only Status
Status: Accepted
Executor: codex direct execute fallback
Date: 2026-05-12

## Direct Execute Fallback

用户要求继续推进 governed task，但没有显式授权 subagents。当前宿主指令要求只有用户明确要求 delegation 时才能创建 subagent，因此 T004 在当前 worktree 中由 Codex direct execute fallback 完成。

## Changed Files

- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/tasks.md`
- `.ai-platform/specs/mvp/analysis.md`
- `.ai-platform/specs/mvp/tasks.md`
- `.ai-platform/specs/mvp/packets/T004.yaml`
- `.ai-platform/evidence/T004/summary.md`
- `.ai-platform/evidence/T004/test-results.md`
- `.ai-platform/evidence/T004/diff.patch`
- `src/cli.rs`
- `src/inspect.rs`
- `src/main.rs`
- `src/manifest.rs`
- `tests/cli.rs`
- `tests/inspect.rs`

## Implementation Summary

- 在 Rust CLI 中加入 `skillrun inspect [--cwd <dir>]` dispatch。
- 新增 human-readable inspect renderer，通过 YAML 解析读取 `.skillrun/manifest.generated.yaml`。
- runnable capsule 输出会展示 status、skill name、SOP/action hash、schema presence、adapter、entrypoint、permissions、examples、preflight status 和 MCP tool metadata。
- 只有 `SKILL.md` 但缺少 runnable contract 的 instruction-only Skill 目录会显示为 `instruction-only`，不会被升级为 runnable capsule，也不会创建 Manifest。
- 新增共享 Manifest path helper，没有改变 Manifest generation 语义。
- T004 未实现 runtime、MCP serving、packaging、run behavior 或 stale Manifest enforcement。

## TDD Summary

- RED: `cargo test --test inspect` 失败，3 个 inspect tests 都因为 `inspect` 仍返回 `command not implemented yet` 而失败。
- GREEN: 实现 renderer 后，`cargo test --test inspect` 通过。
- REFACTOR/validation: `cargo test` 以及 init/manifest/inspect e2e commands 均通过。

## Review

- Spec compliance: Pass。T004 acceptance points 已由 tests 和 inspect output 覆盖。
- Bug/code quality: Pass。Inspect 只读取 Manifest 和本地文件文本，不 import 或运行 `action.py`。
- QA acceptance: Pass。runnable 和 instruction-only 路径均已覆盖。Stale Manifest enforcement 按计划留到 T008。
- Rereview: Pass。2026-05-12 复审无 blocking findings，用户要求复审通过后提交并继续 T005。

## Residual Risk

- T004 只报告 source presence 和 preflight status，完整 stale Manifest fail-closed 行为按计划留到 T008。
- Inspect output 是 human-readable 格式，T004 不引入 machine-stable inspect JSON contract。
