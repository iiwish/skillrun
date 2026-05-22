# Agent Skills Compatibility

**Document status**: positioning baseline
**Last updated**: 2026-05-21

## Summary

Agent Skills define how agents discover and learn capabilities. SkillRun defines how executable skills are checked, run, packaged, evidenced, and exposed.

SkillRun should not position itself as an alternative to Agent Skills. Agent Skills are the emerging standard format for packaging instructions, scripts, references, and assets for agents. SkillRun should be compatible with that mental model while adding a runtime contract for skills that need typed execution, preflight checks, structured outputs, run evidence, packaging, and MCP Router exposure.

```text
Agent Skills made skills portable.
SkillRun makes executable skills dependable.
```

中文表达：

```text
Agent Skills 让技能可移植。
SkillRun 让可执行技能可检查、可运行、可追溯。
```

## Background

Anthropic's `anthropics/skills` repository describes Skills as folders of instructions, scripts, and resources that Claude loads dynamically for specialized tasks. The public Agent Skills specification defines the minimum skill shape as a directory with `SKILL.md`, plus optional `scripts/`, `references/`, and `assets/`.

The standard focuses on progressive disclosure:

1. Discovery: agents load `name` and `description`.
2. Activation: agents read the full `SKILL.md`.
3. Execution: agents follow instructions and may run bundled code or load referenced files.

References:

- <https://github.com/anthropics/skills>
- <https://agentskills.io/>
- <https://agentskills.io/specification>

## Product Boundary

SkillRun should treat Agent Skills as the authoring and discovery layer:

```text
Agent Skill
  SKILL.md
  scripts/
  references/
  assets/

Skill Capsule
  Agent Skill-compatible instructions
  + typed action entrypoint
  + schema contract
  + preflight boundary
  + generated Manifest
  + source hashes
  + structured output/error envelope
  + artifacts
  + run evidence
  + .skr package
  + Router exposure
```

The shortest useful distinction:

```text
Agent Skills = how an agent learns a capability
MCP          = how an agent invokes external capabilities
SkillRun    = how executable skills are validated, run, evidenced, packaged, and mounted
```

## Comparison

| Question | Agent Skills | MCP | SkillRun |
| --- | --- | --- | --- |
| How does an agent discover a capability? | `name`, `description`, and `SKILL.md` metadata | Not the main concern | Should preserve or map skill metadata |
| How does an agent learn the workflow? | Instructions, references, assets, progressive disclosure | Not the main concern | SOP is compiled into a runtime contract |
| How does an agent invoke a tool? | Client-dependent; scripts may exist | Protocol for tools/resources | Router exposes Manifest-derived MCP tools |
| Are inputs and outputs typed? | Not the primary contract | Tool schemas can describe calls | Core contract with schema validation and envelopes |
| Can a consumer inspect before execution? | Format validation exists | Not the main concern | `inspect`, `check`, `doctor`, Consumer Mode |
| Is execution evidence standardized? | Not the primary contract | Not the main concern | Run records, hashes, artifacts, logs, envelopes |
| Does the package fail closed when stale? | Client-dependent | Not the main concern | Manifest freshness and source hash checks |
| Is third-party code made safe? | No | No | No; SkillRun is explicit about not being an OS sandbox |

## When To Use What

Use plain Agent Skills when:

- The value is mainly instructions, references, examples, or templates.
- Scripts are optional helper utilities.
- The agent client already provides the execution and trust boundary you need.
- You do not need stable run evidence or typed input/output enforcement.

Use MCP directly when:

- You already have a service or process that should be exposed as tools/resources.
- SOP, packaging, consumer checks, and run records are outside your concern.

Use SkillRun when:

- The skill includes a real executable action.
- Inputs and outputs should be typed and validated.
- Preflight policy, approval, missing-context, or dependency boundaries matter.
- Consumers should inspect or check the skill before execution.
- Execution must produce structured success/error envelopes, artifacts, and run records.
- The same skill should be packaged as `.skr` and exposed through a local MCP Router.

## Compatibility Direction

SkillRun should become Agent Skills-compatible without polluting the Agent Skills format.

Recommended direction:

- Keep `SKILL.md` as the human and agent-facing SOP file.
- Preserve standard frontmatter concepts such as `name`, `description`, `license`, `compatibility`, `metadata`, and experimental `allowed-tools` where possible.
- Keep SkillRun-specific runtime configuration in `skillrun.config.json` and generated Manifest fields.
- Let `skillrun manifest` map compatible `SKILL.md` metadata into the Manifest after a schema-reviewed implementation. The proposed field-level mapping is tracked in [Agent Skills Metadata Mapping](agent-skills-metadata-mapping.md).
- Treat instruction-only Agent Skills as valid skills, but not runnable capsules unless an action entrypoint and runtime contract exist.
- Keep `.skr` as a SkillRun distribution artifact, not as the Agent Skills standard package format.
- Expose enabled executable capsules through `skillrun router serve --mcp`, not by mounting `.skr` directly into MCP clients.

## Current Non-Goals

- Do not fork the Agent Skills standard.
- Do not claim SkillRun is the universal skill format.
- Do not make SkillRun-specific fields mandatory in `SKILL.md` if they can live in `skillrun.config.json` or Manifest.
- Do not claim scripts in Agent Skills are safe just because they are packaged.
- Do not promise sandboxing, dependency installation, marketplace trust, or signed distribution as current behavior.

## Public Positioning

Recommended homepage/README phrasing:

```text
SkillRun is a runtime and packaging toolchain for executable Agent Skills.
```

Longer:

```text
Agent Skills give agents portable instructions, scripts, references, and assets. SkillRun adds the runtime contract for executable skills: typed inputs and outputs, preflight checks, Manifest freshness, structured envelopes, run evidence, .skr packaging, and MCP Router exposure.
```

Chinese:

```text
Agent Skills 定义技能如何被 Agent 发现和学习；SkillRun 定义可执行技能如何被检查、运行、打包、留证和挂载。
```
