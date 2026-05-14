# Command Hello

## Purpose

This SkillRun capsule demonstrates the Level 0 command adapter.
It is intentionally small: SkillRun Core starts an explicit argv command, creates
standard IPC files, and validates the output envelope written by the command.

## SOP

1. Accept a single `name` value and produce a short greeting.
2. Treat stdout and stderr as logs only. Do not put the result on stdout.
3. Write the result envelope to the path in `SKILLRUN_OUTPUT_JSON`.
4. Write artifacts only inside the directory in `SKILLRUN_ARTIFACT_DIR`.
5. Do not use network access, package managers or dynamic dependency installs.

## Recovery Guidance

If the command adapter fails with `DependencyError`, ask the user to install or
select the declared command executable. If it fails with `ProtocolViolation`,
inspect the run logs and ensure the command writes a valid SkillRun envelope.

## Boundary

This is not a new Python adapter and does not use the SkillRun Python SDK. The
`python action.py` command is just a portable executable command for the example.
Any language or binary can use the same Level 0 contract when it reads the IPC
paths and writes the standard output envelope.
