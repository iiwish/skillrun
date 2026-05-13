# v0.4.1 WeCom Team Notice Analysis

Version: v0.4.1
Status: Ready_For_User_Review
Created: 2026-05-13

## Consistency Review

### Finding A001: Scope aligns with SkillRun positioning

The proposed example keeps SkillRun's product atom as a Skill Capsule. It avoids turning the project into a WeCom API wrapper by requiring SOP, schema, preflight, artifacts, declared env and MCP exposure from Manifest.

Status: Pass.

### Finding A002: Bash adapter should remain out of scope

The user concern that bash has fewer environment dependencies is real, but bash would weaken typed schema, structured errors, Windows compatibility and the core SkillRun narrative. Python stable path is the right first official example path.

Status: Pass.

### Finding A003: Real send must not enter CI

Real WeCom webhook sending requires a secret URL and network. CI should cover dry-run, policy violation and missing webhook `DependencyError`; real send should be manual maintainer evidence only.

Status: Pass with note.

### Finding A004: Agent usage needs explicit MCP model

The docs must say: local humans use `skillrun run`; Agents use MCP clients configured with `skillrun serve --mcp`. The Agent should not infer shell commands.

Status: Pass.

## Residual Risks

- WeCom webhook behavior can change independently of SkillRun.
- Some users may still misread the example as official WeCom integration.
- Secret detection will be heuristic and must not be marketed as DLP.
- Network permission declaration is not sandbox enforcement.

## Recommendation

Proceed after user approval with T041-T046. Keep v0.4.1 as an example-focused patch/minor follow-up unless implementation reveals a required runtime contract change.

