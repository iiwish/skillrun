# SkillRun v0.3 Curated Issue Drafts

Version: v0.3
Status: Ready_For_User_Review
Created: 2026-05-12

这些 issue draft 用于发布 v0.2.0 后引导社区贡献。目标是主动限制项目方向，避免被拉向通用 Agent framework、HTTP server、marketplace 或 sandbox。

## Label Set

- `scope:v0.3`
- `scope:post-v0.3`
- `type:docs`
- `type:author-dx`
- `type:mcp-compat`
- `type:diagnostics`
- `good first issue`
- `needs-design`

## Curated Issues

### 1. Improve 10-minute Quickstart

Labels: `scope:v0.3`, `type:docs`, `good first issue`

Goal:
让 README 中的新作者路径更像真实使用，而不是命令列表。

Acceptance:
- Covers `init -> manifest -> inspect -> test -> serve --mcp --dry-run -> serve --mcp`.
- Explains when to use `--dry-run`.
- Does not claim sandbox, registry or dependency bundling.

### 2. Design `skillrun doctor` / `skillrun validate`

Labels: `scope:v0.3`, `type:author-dx`, `type:diagnostics`, `needs-design`

Goal:
设计一个不执行业务 action 的 capsule 诊断入口。

Acceptance:
- Checks Manifest presence/freshness.
- Checks required capsule files.
- Checks examples exist and are schema-compatible if feasible without running action.
- Reports instruction-only status clearly.
- Does not dynamically import action in Consumer Mode.

### 3. Improve Stale Manifest Recovery Messages

Labels: `scope:v0.3`, `type:author-dx`, `good first issue`

Goal:
当 `SKILL.md`、`action.py` 或 config stale 时，错误信息直接告诉作者下一步怎么恢复。

Acceptance:
- Error identifies stale source path.
- Error suggests `skillrun manifest` when appropriate.
- Error does not expose irrelevant stack trace details.

### 4. Explain Manifest-derived MCP Contract In Inspect Output

Labels: `scope:v0.3`, `type:author-dx`, `type:mcp-compat`

Goal:
让 `skillrun inspect` 更清楚地解释 MCP tool/resource 来自 Manifest，而不是来自 live source import。

Acceptance:
- Shows primary tool name.
- Shows resource exposure summary.
- Mentions Consumer Mode no metadata import.
- Keeps output compact enough for terminal use.

### 5. Add MCP Client Compatibility Notes

Labels: `scope:v0.3`, `type:mcp-compat`, `type:docs`

Goal:
记录如何用常见 MCP client 连接 `skillrun serve --mcp`。

Acceptance:
- Documents stdio transport only.
- Keeps client-specific instructions optional.
- Does not add HTTP/server-hosting scope.

### 6. Improve Python Action Template Comments

Labels: `scope:v0.3`, `type:author-dx`, `good first issue`

Goal:
让 `init --python` 生成的 `action.py` 更像一个可学习的 SkillRun action，而不是普通脚本。

Acceptance:
- Explains `Input`, `Output`, `preflight`, `run`.
- Shows where structured errors belong.
- Avoids real external API calls or secrets.

### 7. Add Capsule Quality Checklist

Labels: `scope:v0.3`, `type:docs`, `type:diagnostics`

Goal:
为作者提供一个人工 checklist，用来判断一个 Skill Capsule 是否适合给 Agent 调用。

Acceptance:
- Covers SOP clarity, schema quality, example coverage, error recoverability, artifact behavior and permission honesty.
- Explicitly states natural-language SOP is not a hard policy engine.

### 8. Track Post-v0.3 Expansion Ideas Without Implementing Them

Labels: `scope:post-v0.3`, `needs-design`

Goal:
为 Node adapter、HTTP transport、registry、sandbox、signed package 等方向建立停车场，防止它们进入 v0.3。

Acceptance:
- Each idea states why it is out of v0.3.
- Each idea names the trust or protocol question that must be solved first.
- No implementation work is attached to this issue.

## Maintainer Guidance

优先接受能降低作者摩擦、提升诊断质量、澄清边界的 PR。

暂缓接受会扩大 runtime surface、改变 Manifest IR、引入远程 transport、承诺安全隔离或要求包生态治理的 PR。
