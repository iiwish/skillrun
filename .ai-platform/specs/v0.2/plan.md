# SkillRun v0.2 Feature Plan

Version: v0.2
Status: Confirmed
Source spec: `.ai-platform/specs/v0.2/spec.md`
Last updated: 2026-05-12
Review: Codex review passed on 2026-05-12; user authorized continuation after review

## 1. Decision Summary

v0.2 用最小纵向切片闭合公开发布路径：

1. 先修正 README 和 release narrative。
2. 再实现真实 MCP stdio server。
3. MCP server 只暴露 Manifest-derived tools/resources。
4. `tools/call` 必须复用现有 Rust runtime + IPC。
5. 通过 scripted protocol fixture 验证。
6. 最后更新 release report，准备 v0.2 public release candidate。

## 2. Constitution Check

Relevant confirmed principles:

- SkillRun Core must remain Rust.
- Manifest is runtime IR and Consumer Mode source of truth.
- Consumer Mode must fail closed on stale Manifest.
- stdout/stderr are logs for SkillRun runtime, but MCP stdio server stdout must be reserved for MCP JSON-RPC messages.
- v0.2 must not claim sandbox or secure package semantics.
- Python `action.py` remains the only blessed adapter path.

Result:

- No constitution violation.
- The only tension is stdout behavior: runtime action stdout remains log-only inside run records, while server process stdout becomes MCP transport. The plan resolves this by routing server diagnostics to stderr and preserving action stdout in run-local logs.

## 3. Technical Decisions

### TDR-201: v0.2 MCP transport is stdio only

Decision:

- Implement only MCP stdio transport.
- Do not implement Streamable HTTP, SSE or remote hosting.

Rationale:

- stdio is enough for local MCP client integration.
- HTTP introduces auth, origin validation and server lifecycle concerns that would blur SkillRun's v0.2 boundary.

Consequence:

- `skillrun serve --mcp` reads newline-delimited JSON-RPC messages from stdin and writes newline-delimited JSON-RPC responses to stdout.

### TDR-202: Protocol target is MCP `2025-11-25`

Decision:

- v0.2 implementation targets official MCP specification `2025-11-25`.
- Each implementation packet must require a fresh official-docs check before coding.

Rationale:

- MCP is evolving; pinning a version prevents silent drift.

Consequence:

- Tests should assert `protocolVersion: "2025-11-25"` unless implementation deliberately negotiates a supported fallback.

### TDR-203: MCP server remains Manifest-driven

Decision:

- Server startup validates Manifest once.
- `tools/list`, `resources/list` and `resources/read` are derived from ValidManifest and source paths.
- Server does not import action metadata.

Rationale:

- This preserves SkillRun's central boundary: Consumer Mode only trusts static Manifest.

Consequence:

- Live source changes after startup do not require hot reload in v0.2.

### TDR-204: Tool calls reuse runtime execution

Decision:

- `tools/call` calls a public runtime execution function rather than duplicating adapter process logic in MCP code.

Rationale:

- Avoids bypassing IPC, run records, artifact validation, declared env injection and structured error discipline.

Consequence:

- `src/runtime.rs` may need a small public API that accepts an in-memory JSON input or writes a request input file under run dir.

### TDR-205: MCP result maps SkillRun envelope without erasing evidence

Decision:

- MCP result returns human/agent-friendly text content and `isError`.
- Full SkillRun envelope remains in run record/output file.
- Error mapping preserves `code`, `message`, `recoverable` and `llm_hint` in text or structured metadata if supported.

Rationale:

- MCP clients expect MCP result shapes; SkillRun still needs audit-grade run records.

Consequence:

- Tests must check both MCP response and run record existence.

### TDR-206: README becomes the release entrypoint

Decision:

- README first screen is optimized for adoption and boundary clarity, not exhaustive governance.
- Chinese README mirrors the English README.

Rationale:

- Public release depends on sharp first impression.

Consequence:

- Detailed governance stays in `.ai-platform/` and `docs/`.

## 4. Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| MCP spec drift | v0.2 implementation mismatches current clients | Pin `2025-11-25`; require fresh docs check in packet |
| stdout pollution | MCP client fails to parse server output | Tests assert stdout contains only JSON-RPC messages |
| Runtime duplication | MCP calls bypass SkillRun safety evidence | `tools/call` must reuse runtime API |
| README overclaim | Users misunderstand sandbox/package state | Explicit non-goals and known limitations |
| Long-running process test flakiness | CI instability | Use bounded scripted client fixture with timeouts |

## 5. Implementation Shape

Expected module shape:

- `src/mcp.rs`
  - JSON-RPC read/write loop.
  - lifecycle handling.
  - tools/resources handlers.
  - MCP response/error helpers.

- `src/cli.rs`
  - switch `serve --mcp` non-dry-run from not-implemented to server loop.
  - keep `--dry-run`.

- `src/runtime.rs`
  - expose minimal reusable execution API for MCP `tools/call`.
  - preserve existing CLI `test` and `run` behavior.

- `tests/mcp_server.rs`
  - protocol-level fixture.
  - lifecycle/tools/resources tests.
  - stdout/stderr discipline.

## 6. Test Strategy

Minimum validation:

- `cargo test --test mcp_server`
- `cargo test --test runtime`
- `cargo test --test consumer_guards`
- `cargo test --test e2e_matrix`
- `cargo test`

MCP fixture must:

- spawn the compiled `skillrun` binary as a child process。
- write newline-delimited JSON-RPC messages to stdin。
- read newline-delimited JSON-RPC responses from stdout。
- capture stderr separately。
- terminate the child process deterministically。

## 7. Execution Order

1. T012: Rewrite README release narrative.
2. T013: Add v0.2 MCP protocol contract tests.
3. T014: Implement long-running MCP stdio lifecycle.
4. T015: Implement tools/list and tools/call runtime wiring.
5. T016: Implement resources/list and resources/read.
6. T017: Complete MCP fixture and release-level E2E coverage.
7. T018: Prepare v0.2 release candidate.

T013 precedes T014-T016 because protocol tests should define the implementation target before code.

## 8. Supporting Artifacts

Required before execution:

- `.ai-platform/specs/v0.2/checklists/requirements.md`
- `.ai-platform/specs/v0.2/analysis.md`
- `.ai-platform/specs/v0.2/packets/T012.yaml` through `T018.yaml`

Optional after P0:

- `docs/mcp-v0.2-contract.md` if protocol contract needs public documentation beyond tests.

## 9. Consequences For Tasks

- No task may change Node adapter, registry, `.skr install`, HTTP server or sandbox scope.
- MCP implementation tasks must include negative tests for stale Manifest and stdout pollution.
- README task should complete before MCP implementation so public promise is stable.
- Release task cannot mark v0.2 as done until all P0 tasks have evidence.

## 10. User Review Gate

- Approval: Confirmed after Codex review on 2026-05-12
- Reviewer notes: No blocking findings. T012 can become Ready after checklist, analysis and packet creation; later tasks remain dependency-gated.
