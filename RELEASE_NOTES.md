# SkillRun v0.2.0 Release Notes

SkillRun v0.2.0 is the first public release handoff.

## Headline

Turn one SOP and one action into a manifest-driven Agent skill capsule with real MCP stdio serving.

## What Is Included

- Real `skillrun serve --mcp` long-running MCP stdio server.
- MCP lifecycle support for `initialize` and `notifications/initialized`.
- Manifest-derived `tools/list`.
- Runtime-backed `tools/call` that preserves SkillRun run records and structured error behavior.
- Manifest-derived `resources/list` and `resources/read` for `SKILL.md` and example inputs.
- Release-level scripted MCP client fixture covering lifecycle, tools, resources and stdout discipline.
- `.skr` packaging as a source + Manifest archive.
- README and release report updated for v0.2.0.

## Boundaries

- MCP transport is stdio only.
- Python `action.py` is the only blessed action adapter target.
- One primary Manifest-derived tool per capsule.
- `.skr` is not signed, not a registry package, not a dependency bundle and not a runtime image.
- SkillRun is not an OS sandbox. Running third-party actions still means executing third-party code.

## Validation

- `cargo test`
- `cargo run -- --version`
- `cargo test --test e2e_matrix a014_mcp_stdio_release_matrix_exercises_full_client_flow`
- Delivery artifact validator
