# T027 Evidence Summary

Task: T027 Update README And TypeScript Boundary Docs
Status: Accepted
Date: 2026-05-13

## Scope

Updated docs to explain the Python stable path, `--py` alias, JS Action Alpha, TypeScript boundary, runtime command language-flag boundary and `.skr` packaging boundary.

## Changed Files

- `README.md`
- `docs/ssot.md`
- `docs/business-examples.md`

## What Changed

- Kept `skillrun init refund --python` as the README main Quickstart.
- Documented `--py` as a short alias for `--python`, not as a separate language path.
- Added a compact JS alpha section for `skillrun init refund-js --js` and canonical ESM `action.mjs`.
- Stated that runtime commands read Manifest and do not accept `--python`, `--py` or `--js`.
- Clarified that `action.ts`, `ts-node`, `tsx`, type-to-schema extraction, source maps, CJS compatibility and package-manager install flows are out of scope.
- Aligned SSOT Node schema language with the implemented v0.3 contract: explicit `inputSchema` / `outputSchema` exports from `action.mjs`.
- Clarified `.skr` as a source + Manifest archive, not a dependency bundle, secure install format, registry package or runtime image.
- Updated business examples so docs-level examples do not expand runtime scope beyond the already defined JS alpha path.

## Existing Worktree Note

`README.md`, `README.zh-CN.md` and `docs/mvp.md` already had uncommitted Capsule URL learning-path edits before T027 started. T027 preserved them. The T027 commit should include only the README boundary updates, not the pre-existing README Capsule URL section, and should continue to exclude `README.zh-CN.md` and `docs/mvp.md`.

## Validation

- `git diff --check`: passed.
- `python D:/data/ai-rd-skill/ai-delivery-governor/scripts/validate_delivery_artifacts.py --root D:/data/skillrunv2`: passed.
- `cargo test --test business_examples --test cli`: passed.
- `git diff --check`: passed again during review.
- `python D:/data/ai-rd-skill/ai-delivery-governor/scripts/validate_delivery_artifacts.py --root D:/data/skillrunv2 --task-id T027`: passed with only cross-spec lookup warnings for older spec folders.

## Review Notes

- T027 is documentation-only.
- No security, package-manager, dependency vendoring, sandbox, registry, HTTP transport or full TypeScript support claim was added.
- README first-screen positioning still distinguishes SkillRun from FastMCP: function-to-tool vs SOP-backed capability-to-Skill Capsule.

## Residual Risk

The working tree still contains pre-existing `README.zh-CN.md` and `docs/mvp.md` edits that are not part of T027. They should remain excluded from any T027 commit unless the maintainer explicitly accepts them.

## Review State

Accepted on 2026-05-13 after spec compliance, engineering quality and QA acceptance review passed.
