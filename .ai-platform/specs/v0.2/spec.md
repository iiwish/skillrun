# SkillRun v0.2 Feature Spec

Version: v0.2
Status: Confirmed
Source: `.ai-platform/specs/v0.2/sop.md`
Last updated: 2026-05-12
Review: Codex review passed on 2026-05-12; user authorized continuation after review

## 1. Product Positioning

v0.2 的定位是 SkillRun 的第一版 public release candidate。

v0.1 证明内部 contract：Skill Capsule 可以被生成 Manifest、inspect、test/run、dry-run MCP contract 和 pack。

v0.2 证明外部采用路径：

> **用户可以把一个 Manifest-driven Skill Capsule 作为真实 MCP stdio server 暴露给 MCP client 调用。**

v0.2 不改变 SkillRun 的核心身份：SkillRun 不是 FastMCP 替代品，不是通用 Agent framework，也不是安全 package manager。它仍然是 SOP-backed Skill Capsule runtime。

## 2. Target Users

- AI engineer / platform engineer：已经有 SOP 和 Python action，希望把它变成 MCP client 可调用的能力。
- 开源早期 adopter：想理解 SkillRun 与 FastMCP、LangChain tools、OpenAI tools 的边界。
- 后续 contributor / Codex agent：需要明确 v0.2 task 边界、协议目标和验证方式。

## 3. User Stories

### US-201: 理解项目定位

用户打开 README 后，可以在 30 秒内理解：

- SkillRun 的产品原子是 Skill Capsule。
- SkillRun 不是函数转 tool 的轻量 wrapper。
- MCP 是对外接口，不是全部价值。
- v0.2 的安全边界仍然不是 sandbox。

### US-202: 启动真实 MCP stdio server

用户在一个有效 capsule 中运行：

```bash
skillrun serve --mcp --cwd examples/refund
```

该命令启动 long-running MCP stdio server，而不是只输出 dry-run contract。

### US-203: 通过 MCP 调用 SkillRun tool

MCP client 可以完成 initialize、tools/list、tools/call。tool schema 来自 Manifest，tool call 通过现有 Rust runtime + IPC 调用 action，并返回 MCP tool result。

### US-204: 通过 MCP 读取 Skill 资源

MCP client 可以完成 resources/list、resources/read，读取 `SKILL.md` 和 examples 等 Manifest-derived resources。读取资源不得动态 import action。

### US-205: 发布 v0.2 release candidate

维护者可以基于真实测试 evidence 发布 v0.2 release candidate，并清楚说明 v0.2 做了什么、不做什么、还有哪些已知限制。

## 4. Core User Journey

```bash
skillrun init refund --python
cd refund
skillrun manifest
skillrun test
skillrun serve --mcp
```

MCP client 连接该进程后：

```text
initialize
-> notifications/initialized
-> tools/list
-> tools/call
-> resources/list
-> resources/read
```

## 5. Functional Requirements

### FR-201: README Release Narrative

README 第一屏必须把 SkillRun 定位为 `manifest-driven Agent skill capsule`，并明确：

- FastMCP turns functions into tools。
- SkillRun turns SOP-backed capabilities into Skill Capsules。
- v0.2 支持真实 MCP stdio server。
- `.skr` 是 source + Manifest archive。
- v0.2 不是 sandbox、不是 registry、不是 dependency bundle。

### FR-202: MCP Stdio Server Startup

`skillrun serve --mcp --cwd examples/refund` 这类命令必须：

- 在启动前执行 Consumer Mode Manifest validation。
- Manifest 缺失、stale 或 source hash 不匹配时 fail closed。
- 启动 long-running stdio server。
- 支持 `--dry-run` 保留现有 contract inspection 行为。
- stdout 只写 MCP JSON-RPC messages。
- 日志和 diagnostics 只写 stderr 或 run/debug 文件。

### FR-203: MCP Lifecycle

server 必须支持当前官方 MCP specification `2025-11-25` 的最小 lifecycle：

- 接收 `initialize`。
- 返回 protocol version、serverInfo 和 capabilities。
- 接收 `notifications/initialized` 后进入 normal operation。
- stdin EOF 后退出。
- 对未支持或无效 method 返回 JSON-RPC error。

v0.2 server capabilities 至少包含：

```json
{
  "tools": {},
  "resources": {}
}
```

不承诺 prompts、sampling、elicitation、roots、tasks、logging capability。

### FR-204: MCP Tools List

`tools/list` 必须：

- 只从 Manifest 生成 tools。
- v0.2 每个 capsule 暴露一个 primary tool。
- tool `name` 来自 Manifest `tool.name`。
- tool `description` 来自 Manifest `tool.description`。
- tool `inputSchema` 来自 Manifest `schemas.input`。
- tool `outputSchema` 可以来自 Manifest `schemas.output`，如果 implementation 选择暴露。
- 不动态 import action。

### FR-205: MCP Tools Call

`tools/call` 必须：

- 根据 request `params.name` 匹配 Manifest tool。
- 将 `params.arguments` 作为 SkillRun runtime input。
- 通过现有 Rust runtime + Python adapter IPC 执行 action。
- 不绕过 run record、artifact boundary、declared env injection、output/error envelope validation。
- 成功时返回 MCP tool result，至少包含 `content: [{ "type": "text", "text": ... }]` 和 `isError: false`。
- SkillRun error envelope 时返回 `isError: true`，并把 structured error 摘要放入 text content。
- 保留完整 SkillRun envelope 到 run record；MCP result 不需要暴露内部绝对路径。

### FR-206: MCP Resources List

`resources/list` 必须：

- 只从 Manifest/source paths 暴露受控 resources。
- 至少暴露 `SKILL.md`。
- 可以暴露 examples input files。
- resource URI 使用 `skillrun://{skill}/{path}` 形式。
- 不暴露 `.skillrun/runs/`。
- 不暴露 arbitrary local files。

### FR-207: MCP Resources Read

`resources/read` 必须：

- 只允许读取 `resources/list` 中出现过的受控 URI。
- `SKILL.md` 返回 `text/markdown`。
- examples 返回 `application/json`。
- 不动态 import action。
- path traversal、未知 URI、stale Manifest 均失败。

### FR-208: MCP Protocol Test Fixture

v0.2 必须包含 scripted MCP client fixture 或等价 protocol-level integration test，覆盖：

- initialize。
- notifications/initialized。
- tools/list。
- tools/call success。
- tools/call structured error。
- resources/list。
- resources/read。
- stdout 不混入非 JSON-RPC 日志。

### FR-209: Release Candidate Hygiene

v0.2 发布前必须：

- 更新 version 和 README。
- 更新中文 README。
- 更新 release report。
- 记录已知限制。
- 运行 `cargo test`。
- 明确 public release notes。

## 6. Non-Functional Requirements

### NFR-201: Protocol Fidelity

v0.2 MCP behavior 必须对照官方 MCP specification `2025-11-25`。执行 implementation task 前，Codex 必须再次核对官方 docs，因为 MCP specification 可能更新。

### NFR-202: Trust Boundary Honesty

文档必须继续说明：

- permissions 不是 sandbox。
- third-party action 仍然是 third-party code execution。
- `.skr` 不是 signed package。
- v0.2 不提供 dependency isolation。

### NFR-203: Runtime Boundary Preservation

MCP tool call 不得绕过现有 runtime discipline：

- Consumer Mode validation。
- IPC file protocol。
- output/error envelope。
- artifact containment。
- declared env injection。
- run record。

### NFR-204: Logging Discipline

stdio server stdout 不得写日志、debug text、panic text 或 human-readable diagnostics。stderr 可以承载 UTF-8 log text。

### NFR-205: Backward Compatibility

`skillrun serve --mcp --dry-run` 保留，用于 contract inspection 和 CI。v0.2 不破坏 v0.1 command flow。

### NFR-206: Scope Control

v0.2 不引入 Node adapter、HTTP transport、registry、install、sandbox、多 action 编排或 marketplace。

## 7. Functional Scope

In scope:

- README release narrative。
- MCP stdio transport。
- Minimal MCP lifecycle。
- tools/list。
- tools/call。
- resources/list。
- resources/read。
- Protocol fixture tests。
- Release candidate docs。

Out of scope:

- Streamable HTTP。
- MCP prompts。
- MCP sampling / elicitation / roots。
- MCP auth。
- task-augmented execution。
- progress notification。
- cancellation。
- resource subscription。
- tools/list pagination beyond single-page result。
- multiple tools per capsule。

## 8. Edge Cases

- Manifest stale before server startup：exit non-zero, stderr explains stale source.
- Invalid JSON-RPC line on stdin：return JSON-RPC parse error if possible, otherwise stderr and exit policy defined in plan.
- Unrecognized method：return method-not-found JSON-RPC error.
- `tools/call` unrecognized tool：return JSON-RPC error or MCP tool error, must be deterministic.
- `tools/call` invalid arguments：return MCP result with `isError: true` mapped from SkillRun `ValidationError`.
- Action returns `PolicyViolation`：return MCP result with `isError: true` and include `llm_hint` when present.
- Resource URI path traversal：reject.
- `SKILL.md` modified while server is running：v0.2 may choose startup-only validation; live revalidation is not required.

## 9. Constraints And Assumptions

- Rust Core remains the implementation boundary.
- Python `action.py` remains the only v0.2 blessed action target.
- The target MCP protocol version is `2025-11-25`, current official latest as checked on 2026-05-12.
- v0.2 only supports stdio transport.
- The scripted test fixture is sufficient for release gate; manual compatibility with a named MCP client is optional.
- v0.2 is the first public release candidate; v0.1 is not separately published.
- English README is primary; Chinese README is a full mirror.

## 10. Data And Integration Needs

- No external network service is required.
- MCP client fixture should spawn `skillrun serve --mcp --cwd examples/refund` or an equivalent generated capsule as a subprocess.
- Test fixtures should use generated capsule or `examples/refund`.
- No real secrets, payment APIs or network calls are required.

## 11. Success Criteria

- A new reader understands SkillRun's FastMCP boundary from README first screen.
- `cargo test` passes.
- `skillrun serve --mcp --cwd examples/refund` can be exercised by a scripted MCP client fixture.
- `tools/list` schema matches Manifest schema.
- `tools/call` produces a SkillRun run record.
- `resources/read` reads `SKILL.md` without importing action.
- stdout contains only valid newline-delimited JSON-RPC messages during stdio server operation.

## 12. Acceptance Criteria

- AC-201: README and Chinese README are updated and reviewed.
- AC-202: MCP stdio server lifecycle tests pass.
- AC-203: MCP tools/list and tools/call tests pass.
- AC-204: MCP resources/list and resources/read tests pass.
- AC-205: stale Manifest server startup test passes.
- AC-206: stdout/stderr discipline test passes.
- AC-207: release report records v0.2 evidence and known limitations.

## 13. Clarifications

- 2026-05-12: 用户决定先优化 README，但等做完 v0.2 再发布。
- 2026-05-12: 用户要求 Codex 开发 SOP；SOP 复审通过后创建 v0.2 spec / plan / tasks。
- 2026-05-12: SOP 决议：v0.2 只支持 MCP stdio transport；协议级 fixture 是 blocking release gate；v0.2 是第一版 public release candidate。

## 14. Open Questions

None blocking for planning.

## 15. Reference Sources

- Official MCP specification `2025-11-25`: https://modelcontextprotocol.io/specification/2025-11-25
- MCP transports: https://modelcontextprotocol.io/specification/2025-11-25/basic/transports
- MCP lifecycle: https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
- MCP tools: https://modelcontextprotocol.io/specification/2025-11-25/server/tools
- MCP resources: https://modelcontextprotocol.io/specification/2025-11-25/server/resources

## 16. User Review Gate

- Approval: Confirmed after Codex review on 2026-05-12
- Reviewer notes: No blocking findings. Implementation must not start until checklist, analysis and the relevant execution packet are present and task dependencies are satisfied.
