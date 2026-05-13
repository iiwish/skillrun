# 分支保护建议

公开开源前，SkillRun 可以保持单维护者的轻量流程；公开后，`main` 应作为可信公共历史保护。

## 公开前

- 日常工作使用 `codex/` 分支。
- `main` 只接收经过验证的文档、版本或功能合并。
- 合并回 `main` 前确认 `git status` 干净，并记录验证命令。
- 允许维护者本地 `--no-ff` 合并，但不要改写已推送的 `main` 和 tag。

## 公开后 GitHub 设置

建议对 `main` 启用：

- Require a pull request before merging。
- Require status checks to pass before merging。
- Required checks: `fmt`, `clippy`, `test`。
- Require branches to be up to date before merging。
- Require conversation resolution before merging。
- Restrict force pushes。
- Restrict deletions。
- Require signed tags for release tag，或至少使用 annotated tags。

单维护者阶段可以不强制多 reviewer，但仍建议所有非平凡 runtime 变更通过 PR 描述、CI 和 release note 检查。

## 合并方式

- 功能或版本分支优先使用 merge commit，保留集成边界。
- 小型文档修复可以 squash merge。
- 不在公开历史中使用 force push 修正审美问题。
- 只有在公开前发现 secrets、隐私、版权或大文件误提交时，才考虑历史重写。
