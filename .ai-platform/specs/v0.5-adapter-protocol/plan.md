# SkillRun v0.5.0 Plan: Language-agnostic Adapter Protocol

Version: v0.5.0
Status: Confirmed
Last updated: 2026-05-14
Source spec: `.ai-platform/specs/v0.5-adapter-protocol/spec.md`
Review: User requested review, commit and continuation on 2026-05-14; plan accepted for packetized execution.

## Decision Summary

### D-050-001: Level 0 Command Adapter Is Required

v0.5.0 will implement Level 0 command adapter. This is the smallest vertical slice that proves SkillRun can run a language-neutral process through standard IPC/envelope contracts without blessing a new language ecosystem.

### D-050-002: Command Adapter Uses Explicit Argv Only

`runtime.command` must be an argv array, not a shell string. SkillRun should not invoke a shell for command adapter execution. This avoids shell quoting ambiguity and keeps Level 0 from becoming a shell runner.

### D-050-003: Static Schemas Use JSON Schema First

Level 0 command adapter should use explicit JSON Schema in config or generated Manifest. Lightweight schema sugar can be considered later, but v0.5 should keep the contract precise.

### D-050-004: Requirements Are Diagnostics, Not Installation

Command adapter readiness may check executable presence. It must not install packages, create environments, vendor dependencies or infer package-manager behavior.

### D-050-005: Conformance Starts As Rust Integration Tests

v0.5.0 conformance should begin as Rust integration tests using existing fixtures and one command-adapter fixture. A future CLI conformance runner can be considered after the protocol stabilizes.

## Constitution Check

- Rust Core remains the runtime authority.
- Core still reads Manifest, creates IPC, validates envelope/artifact contracts and exposes MCP.
- Language ecosystems do not define product semantics.
- Consumer Mode remains static for metadata and no-import.
- No sandbox, registry, dependency vendoring or package-manager install flow is introduced.

## Proposed Manifest / Config Shape

Author-facing `skillrun.config.json` shape:

```json
{
  "runtime": {
    "adapter": "command",
    "command": ["ruby", "action.rb"],
    "timeout": "30s"
  },
  "input_schema": {
    "type": "object",
    "required": ["name"],
    "additionalProperties": false,
    "properties": {
      "name": { "type": "string" }
    }
  },
  "output_schema": {
    "type": "object",
    "required": ["message"],
    "additionalProperties": false,
    "properties": {
      "message": { "type": "string" }
    }
  }
}
```

Manifest should record:

```yaml
runtime:
  adapter: command
  command:
    - ruby
    - action.rb
  protocol_version: adapter.v1
```

The command process must read the same IPC environment variables and write the same output/error envelope as Python and JS adapters.

## Conformance Categories

- Success envelope written to `SKILLRUN_OUTPUT_JSON`.
- Invalid input maps to `ValidationError` before command execution when schema fails.
- Command exits without output file maps to `ProtocolViolation`.
- Command writes malformed output maps to `ProtocolViolation`.
- Artifact path traversal maps to `PermissionDenied`.
- stdout text is captured as logs, never result.
- Missing command executable maps to `DependencyError`.
- Stale Manifest fails before runtime dispatch.

## Risks

- Users may treat command adapter as arbitrary shell execution.
- Static schema in config can become verbose.
- Command examples may accidentally imply official support for a language.
- Existing Python/JS paths could regress if internal adapter refactor is too broad.

## Mitigations

- Only support argv arrays, not shell strings.
- Use one tiny example and document it as command-adapter example, not Ruby/PHP/Go support.
- Keep Python and JS release tests green throughout.
- Implement conformance tests before broad refactor.

## Consequences For Tasks

Tasks should be split so protocol documentation, config/manifest support, runtime dispatch, conformance tests and docs/examples can be reviewed independently. Implementation must not begin until task packets exist.
