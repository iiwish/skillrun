# Security Policy

SkillRun executes user-provided action code through declared adapters, so the project treats trust boundaries as part of the product surface.

## Supported Versions

| Version | Status |
| --- | --- |
| `main` | Active development |
| Latest tagged release | Supported for security review |
| Older tags | Best-effort only unless a release branch is announced |

The project is still pre-1.0. Public APIs and package formats may change, but security-sensitive behavior should remain explicit and fail closed.

## Reporting a Vulnerability

Please do not report suspected vulnerabilities in public issues.

When the repository is public, use GitHub private vulnerability reporting or a private maintainer channel listed on the project profile. Include:

- A short description of the vulnerability.
- Affected version, commit, or tag.
- Reproduction steps or a minimal capsule when possible.
- Impact assessment, including whether arbitrary code execution, data exposure, permission bypass, artifact escape, or Manifest confusion is involved.

The maintainer will acknowledge confirmed reports, coordinate a fix, and publish release notes when disclosure is appropriate.

## Security Boundaries

SkillRun currently does not claim to provide:

- A full OS sandbox.
- A signed package format.
- Dependency vendoring for `.skr` archives.
- Safe execution of arbitrary third-party actions.

Security-sensitive guarantees that should be preserved:

- `stdout` and `stderr` are logs only.
- Structured success and failure must come from explicit output or error envelopes.
- Missing or stale Manifests fail closed.
- Consumer mode must not import source code to recover metadata.
- Artifacts must stay inside declared artifact boundaries.
- Declared environment variables are part of the runtime contract.
