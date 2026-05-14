# Bounded File Patcher

## Purpose

This SkillRun capsule applies one bounded text replacement inside an allowed
project directory. It is an example of converting file-write rules into a
manifest-bound skill contract, not a claim of OS sandboxing.

## SOP

1. Modify only project-relative files under `src/`, `docs/` or `tests/`.
2. Never accept absolute paths, drive-prefixed paths, hidden path segments or
   parent-directory traversal.
3. Never modify secret-bearing files, lockfiles or package manager manifests.
4. Require an exact `old_text` match before writing `new_text`.
5. Reject ambiguous replacements where `old_text` appears more than once.
6. Record a markdown patch summary artifact for audit.

## Required Context

- `file_path`: project-relative path under `src/`, `docs/` or `tests/`.
- `old_text`: exact text to replace.
- `new_text`: replacement text.

## Recovery Guidance

If this capsule returns `PolicyViolation`, choose an allowed path or narrower
replacement. If it returns `ValidationError`, provide all required fields.

## Prohibited Behavior

- Do not overwrite entire files without an exact old-text match.
- Do not modify `.env`, secrets, lockfiles, package manifests or hidden folders.
- Do not treat this capsule as an OS sandbox.
