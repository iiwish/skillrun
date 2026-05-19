# Agent Context Skills and Runtime Capsules

**Document status**: Draft_For_Product_Review  
**Last updated**: 2026-05-18  
**Scope**: SkillRun CLI, Desktop, `.skr` packaging, and Anthropic-style `SKILL.md` distribution

---

## One-Sentence Judgment

SkillRun should treat Agent capabilities as two related but separate artifact types:

```text
Context Skill    = SKILL.md + optional references/scripts/templates, loaded by an Agent
Runtime Capsule  = SKILL.md + action + schema + examples + permissions + Manifest
```

The CLI must own validation, packaging, install planning, and rollback semantics for both. Desktop should only visualize and confirm CLI contracts.

## Why This Exists

The current SkillRun runtime already recognizes an `instruction-only` directory: a folder with `SKILL.md`, references, scripts, assets, or examples but no explicit `action.py` / `action.mjs` and no valid Manifest. Runtime commands refuse to infer execution from that shape.

That behavior is correct, but the public product language is incomplete. A pure Agent Skill is not a broken Capsule. It is a different kind of capability:

- It does not need SkillRun runtime execution.
- It can still be valuable when mounted into Codex, Claude Code, or another Agent terminal.
- It still needs inspection, packaging, workspace sharing, target install planning, and rollback.
- It must not be confused with a runnable Capsule or exposed as an MCP tool.

## Artifact Taxonomy

| Artifact | Primary job | Runtime execution | MCP exposure | Install target |
| --- | --- | --- | --- | --- |
| Context Skill | Teach an Agent a workflow, style, domain rules, or tool-use discipline | No SkillRun execution | No | Agent skill directory |
| Runtime Capsule | Execute a bounded business action with schema, preflight, envelope, and evidence | Yes | Optional through SkillRun Router | SkillRun registry |

## Current Behavior

Today, SkillRun Core supports the negative boundary:

- `inspect` can report `status: instruction-only`.
- `check` and `doctor` explain that SkillRun does not infer actions from Markdown, scripts, references, assets, or examples.
- `run`, `serve --mcp`, `router`, `pack`, and `switchboard enable` refuse instruction-only skills as runtime capsules.

This is not enough for Context Skill distribution. It only prevents accidental execution.

## Proposed CLI Ownership

The CLI should become the source of truth for Context Skill lifecycle because install paths, target compatibility, rollback, and workspace sharing must be scriptable and testable.

Proposed command family:

```bash
skillrun skill inspect <skill-dir-or-package> --json
skillrun skill validate <skill-dir> --format anthropic --json
skillrun skill pack <skill-dir> --format anthropic -o <name>.skr
skillrun skill install <package.skr> --target codex --scope user --json
skillrun skill install <package.skr> --target claude-code --scope project --cwd . --json
skillrun skill mount plan <package.skr> --target codex --scope user --json
skillrun skill remove <name> --target codex --scope user --json
```

Contract rules:

- `skillrun skill install` copies or links files only after a plan has been shown.
- `skillrun skill install` does not execute scripts.
- `skillrun skill install` does not mark the skill trusted.
- `skillrun skill pack` for Context Skills does not require a Manifest.
- `skillrun pack` for Runtime Capsules continues to require a valid Manifest.
- `skillrun import` for Runtime Capsules continues to import disabled by default.

## `.skr` Packaging Direction

`.skr` can remain the distribution extension, but the package metadata must identify the artifact kind before any consumer action:

```json
{
  "package_schema_version": "skr.package.v1",
  "artifact_kind": "context_skill",
  "format": "anthropic-skill",
  "entrypoint": "SKILL.md"
}
```

Runtime Capsule packages should be explicit too:

```json
{
  "package_schema_version": "skr.package.v1",
  "artifact_kind": "runtime_capsule",
  "manifest_path": ".skillrun/manifest.generated.yaml",
  "mcp_exposable": true
}
```

Without this distinction, Desktop and workspace sharing will either over-trust Context Skills or over-execute Runtime Capsules.

## Desktop Boundary

Desktop should not directly copy skills into Codex, Claude Code, Cursor, or project directories. It should call CLI contracts:

```text
Open .skr / folder
  -> skillrun skill inspect --json
  -> skillrun skill mount plan --target <agent> --json
  -> user confirms
  -> skillrun skill install --json
  -> Desktop shows installed target and rollback/remove action
```

Desktop owns:

- File selection.
- Target and scope selection.
- Plan review.
- User confirmation.
- Visual status.

Desktop does not own:

- Target path rules.
- File copying/linking.
- Rollback semantics.
- Package extraction.
- Trust or signature interpretation.

## Product Language

Use this taxonomy in public docs:

```text
Agent Capability
  - Context Skill: loaded by Agent terminals, not executed by SkillRun
  - Runtime Capsule: executed by SkillRun Core, optionally exposed through MCP
```

Avoid saying "SkillRun is an MCP tool manager." More accurate:

> SkillRun packages and distributes Agent capabilities. Some capabilities are context-only skills loaded by the Agent. Others are runtime capsules executed by SkillRun and exposed through MCP when needed.

## Non-Goals

Do not do these in the first implementation:

- One-click global enable of third-party Context Skills.
- Automatic script execution during Context Skill install.
- Automatic dependency installation.
- Marketplace ranking, likes, or recommendations.
- Signed trust claims before the package trust model exists.
- Desktop-only install logic that bypasses CLI planning.
- Treating Anthropic-style `SKILL.md` as a runnable SkillRun Capsule.

## MVP Gate

The first useful slice is:

```text
CLI:
  inspect/validate Context Skill
  plan install to one target
  apply install
  remove/rollback

Desktop:
  open .skr
  show Context Skill vs Runtime Capsule
  confirm target install
```

This slice corrects the MCP-first perception without changing the Runtime Capsule contract.
