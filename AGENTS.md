# Codex 项目约定

## 项目身份

- 项目名称使用 `SkillRun`。
- CLI、crate、命令和代码标识使用小写 `skillrun`。
- SkillRun 本体使用 Rust 实现；Python 只作为 MVP 首个 Action adapter 目标。
- 不给项目名追加 `v2` 后缀；`v0.1.0` 等版本号只用于 release 或 artifact 版本。

## 文档语言

- 面向项目的默认文档语言是中文。
- 关键术语、命令、API 名称、状态名、文件名和协议名可以保留英文，例如 `Manifest`、`Skill Capsule`、`Ready_For_User_Review`、`skillrun manifest`。
- 新增或更新治理文档时，优先保持中文叙述和稳定 heading，方便后续 agent 解析。

## Git 分支策略

- `main` 作为稳定主线，只接收已经完成 review、验证并准备进入公开历史的变更。
- Codex 不直接在 `main` 上提交功能、版本或较大文档变更；开始可提交工作前，先从当前主线创建 `codex/` 前缀分支。
- 版本或里程碑工作使用 `codex/v<major>.<minor>-integration` 形式的集成分支，例如 `codex/v0.3-integration`。
- 独立任务可从版本集成分支再拆短分支，命名为 `codex/<task-id>-<topic>` 或 `codex/<topic>`，完成后先合回版本集成分支。
- 当发现本地 `main` 已领先远端或包含未推送提交时，先基于当前 `main` 创建合适的 `codex/` 分支承接现有提交，再继续后续修改。
- 合并回 `main` 前应确认工作区干净、测试结果明确、文档与任务状态同步；合并方式优先使用 PR 或显式的非快进合并记录。
- 未经用户明确要求，不改写共享历史，不对用户已有提交执行 `reset`、`rebase`、`amend` 或强推。

## Git 提交规范

- 提交信息使用 Conventional Commits：`type(scope): summary`；没有清晰 scope 时可用 `type: summary`。
- `type` 优先使用 `feat`、`fix`、`docs`、`test`、`refactor`、`chore`、`build`、`ci`、`perf`、`revert`。
- `scope` 使用稳定的小写标识，优先选择 `cli`、`manifest`、`runtime`、`adapters`、`mcp`、`docs`、`specs`、`tests`、`release` 等项目边界。
- `summary` 使用英文小写祈使短句，不以句号结尾，例如 `feat(adapters): add node metadata support`。
- 单个提交应保持原子性：同一提交只表达一个意图，相关代码、测试、文档可以同提交，不混入无关格式化或临时文件。
- 提交前优先运行与变更范围匹配的验证，例如 `cargo fmt`、`cargo test`、相关 CLI golden/contract 测试；无法运行时在交付说明中明确原因。
- Codex 暂存文件前必须检查 `git status` 和 `git diff`，只暂存本次任务相关文件，避免吸收用户未要求提交的工作区修改。
- 需要 release 或 artifact 版本时，版本号只出现在 release 文档、tag 或产物元数据中，不写入项目名称或分支外的命名约定。
