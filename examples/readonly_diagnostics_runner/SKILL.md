# Read-only Diagnostics Runner

## Purpose

This SkillRun capsule runs a small allowlist of read-only diagnostic commands.
It demonstrates how a high-risk terminal-shaped action can be narrowed into a
typed skill contract.

## SOP

1. Accept only named diagnostics from the schema. Do not accept arbitrary shell
   strings.
2. Run commands without `shell=True`.
3. Do not support redirection, pipes, command chaining, mutation commands,
   package installs or process killing.
4. Capture stdout, stderr, exit code and timeout status.
5. Truncate oversized output so the Agent can recover without flooding context.

## Allowed Diagnostics

- `pwd`: print the capsule working directory.
- `list`: list files in the capsule working directory.
- `git_status`: run `git status --short` when the current directory is inside a Git repository.

## Recovery Guidance

If a diagnostic fails, inspect the structured `exit_code`, `stdout` and
`stderr`. Do not retry with a broader shell command.

## Prohibited Behavior

- Do not pass arbitrary command strings.
- Do not run destructive commands.
- Do not treat this capsule as a general-purpose shell or an OS sandbox.
