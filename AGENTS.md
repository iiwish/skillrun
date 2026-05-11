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
