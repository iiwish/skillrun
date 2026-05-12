# T008 Evidence Summary

Task: T008 Implement Stale Manifest And Instruction-only Command Guards
Status: Accepted
Executor: codex direct execute fallback
Date: 2026-05-12

## Direct Execute Fallback

用户要求继续推进 T008，但没有显式要求 delegation。当前宿主指令要求只有用户明确要求 subagents 时才能创建 subagent，因此 T008 在当前 worktree 中由 Codex direct execute fallback 完成。

## Changed Files

- `README.md`
- `README.zh-CN.md`
- `.ai-platform/docs/tasks.md`
- `.ai-platform/docs/release-report.md`
- `.ai-platform/specs/mvp/analysis.md`
- `.ai-platform/specs/mvp/tasks.md`
- `.ai-platform/specs/mvp/packets/T008.yaml`
- `.ai-platform/evidence/T008/summary.md`
- `.ai-platform/evidence/T008/test-results.md`
- `.ai-platform/evidence/T008/diff.patch`
- `src/main.rs`
- `src/cli.rs`
- `src/consumer.rs`
- `src/manifest.rs`
- `src/inspect.rs`
- `src/runtime.rs`
- `tests/cli.rs`
- `tests/inspect.rs`
- `tests/consumer_guards.rs`
- `tests/instruction_only.rs`

## Implementation Summary

- 新增 `src/consumer.rs`，集中校验 Consumer Mode 的 generated Manifest、source paths 和 source hashes。
- `skillrun run/test` 在创建 run directory 和启动 adapter 前先执行 Consumer Mode guard。
- `skillrun serve --mcp` 和 `skillrun pack` 现在先执行 Consumer Mode guard；valid capsule 仍返回 unimplemented，stale 或 instruction-only 则 fail closed。
- `skillrun inspect` 在 Manifest 存在但 source missing/stale 时展示 `invalid-runnable`，不会 import `action.py`。
- `skillrun manifest` 缺少显式 `action.py` 时明确说明不会从 Markdown、scripts、references、assets 或 examples 推断 Action。
- README 状态更新到 T008；T009/T010 的 MCP 和 packaging 仍未实现。

## TDD Summary

- RED: `cargo test --test consumer_guards --test instruction_only` 失败。stale `SKILL.md` 仍可 run 成功，serve/pack 直接落到 unimplemented，instruction-only run 只报 missing Manifest，manifest 缺少“不推断 action”说明。
- GREEN: 实现 `consumer` guard、serve/pack guard 入口、inspect invalid-runnable 和 manifest refusal message 后，`cargo test --test consumer_guards --test instruction_only` 通过。
- REFACTOR/validation: 更新旧 CLI/inspect 断言到 T008 语义后，`cargo fmt -- --check`、`git diff --check` 和 `cargo test` 均通过。

## Review

- Spec compliance: Pass。修改 `SKILL.md`、`action.py` 或 `skillrun.config.json` 后 run/serve/pack 均 fail closed。
- Bug/code quality: Pass。Consumer Mode guard 集中在 `src/consumer.rs`，run/test、serve、pack 和 inspect 复用同一套 source hash 校验。
- QA acceptance: Pass。instruction-only 目录可以 inspect，但 manifest/run/serve/pack 不会从 Markdown、scripts 或 examples 推断可执行入口。
- Rereview: Pass。2026-05-12 复审无 blocking findings，用户要求复审通过后提交并继续 T009。

## Residual Risk

- T008 不实现真正 MCP server；T009 仍需接入 Consumer Mode guard 后暴露 Manifest-driven MCP。
- T008 不生成 `.skr`；T010 仍需复用 Consumer Mode guard 后实现 pack。
