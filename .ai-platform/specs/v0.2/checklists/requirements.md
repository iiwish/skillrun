# SkillRun v0.2 Requirements Checklist

Version: v0.2
Status: Completed
Source spec: `.ai-platform/specs/v0.2/spec.md`
Last updated: 2026-05-12

## Checklist Scope

本 checklist 检查 v0.2 spec、plan 和 tasks 是否足以进入 packetized execution。它测试需求文本质量，不测试实现。

## Requirement Quality Checks

### Clarity

- [x] v0.2 北极星是否清楚区分 internal MVP proof 和 public release candidate。
- [x] 是否明确 MCP 是 v0.2 的外部采用路径，而不是 SkillRun 全部价值。
- [x] 是否明确 target protocol version 为 MCP `2025-11-25`。
- [x] 是否明确 v0.2 只支持 stdio transport。
- [x] 是否明确 README 第一屏应解释 SkillRun 与 FastMCP 的边界。

### Completeness

- [x] 是否覆盖 README narrative、MCP lifecycle、tools、resources、protocol fixture 和 release hygiene。
- [x] 是否定义 stale Manifest、invalid input、PolicyViolation、resource traversal、stdout pollution 等边界。
- [x] 是否说明 `.skr` 不是 signed package、dependency bundle 或 sandbox。
- [x] 是否保留 `serve --mcp --dry-run` 的 backward compatibility。
- [x] 是否定义 release candidate 的 Definition of Done。

### Consistency

- [x] spec、plan、tasks 对 v0.2 范围保持一致。
- [x] 所有 tasks 都映射到 `US-*`、`FR-*` 或 `NFR-*`。
- [x] work graph dependency 顺序与 plan execution order 一致。
- [x] 术语使用保持 `SkillRun`、`skillrun`、`Skill Capsule`、`Manifest`、`MCP stdio server`。
- [x] README 叙事与安全边界不冲突。

### Testability

- [x] 每个 P0 requirement 都有对应 task 和 validation command。
- [x] MCP server 行为有 protocol-level scripted fixture 要求。
- [x] stdout/stderr discipline 有可测验收条件。
- [x] `tools/call` 成功和 structured error 路径都有测试要求。
- [x] release hygiene 有 `cargo test`、version 和 release report 检查。

### Edge Cases

- [x] startup stale Manifest fail closed 已定义。
- [x] unrecognized method、invalid JSON-RPC、unrecognized tool、invalid arguments 已定义。
- [x] resource URI traversal 和 arbitrary local file exposure 已定义。
- [x] live source mutation 不作为 v0.2 hot reload requirement。
- [x] child process hanging 风险通过 bounded scripted fixture 和 timeout 处理。

### Non-Functional Requirements

- [x] Protocol fidelity 有版本锚点和 fresh docs check 要求。
- [x] Trust boundary honesty 有明确要求。
- [x] Runtime discipline 保留 existing IPC/run record/artifact/env boundaries。
- [x] Logging discipline 明确 stdout 只承载 MCP JSON-RPC。
- [x] Scope control 明确拒绝 Node、HTTP、registry、sandbox、marketplace。

## Findings Summary

- Critical: 0
- High: 0
- Medium: 0
- Low: 0

## Resolution Notes

- MCP 相关 scope 已收敛到 stdio transport。
- v0.2 的 release story 已从 “MCP dry-run contract” 调整为 “真实 MCP stdio serving”。
- README 和 release hygiene 被前置为 T012/T018，避免实现完成后叙事仍漂移。

## User Review Gate

- Approval: Completed by Codex review on 2026-05-12 per user request.
- Reviewer notes: No checklist finding blocks packet creation or T012 execution.
