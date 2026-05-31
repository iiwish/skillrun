# SkillRun Examples

This directory is the single top-level home for official SkillRun example capsules.

## Core-Validated Examples

These examples are self-contained and participate in Core validation or local release checks:

- `meeting_action_brief`: product/adoption hero for meeting notes, action items, risks, follow-up copy, artifacts, and `.skr` packaging.
- `refund`: contract/policy-boundary hero for approvals, structured errors, artifacts, run records, MCP exposure, and `.skr` packaging.
- `wecom_team_notice`: local notification workflow with dry-run preview, approval boundaries, declared env, and markdown artifacts.
- `commit_message_gate`: Conventional Commits validation without auto-staging files.
- `bounded_file_patcher`: exact text replacement inside declared directories with patch artifacts.
- `readonly_diagnostics_runner`: named allowlist diagnostics without arbitrary shell strings.
- `command_hello`: Level 0 command adapter contract without a SkillRun SDK.

## Optional External-Tool Examples

These examples also live under `examples/` so the repository has one examples entry point. They may require external CLIs, credentials, or network access, and are not part of the mandatory Core CI path.

- `examples/lark_notice_sender`: Feishu/Lark message sender through `lark-cli`, constrained by chat_id allowlist, dry-run default, sensitive content checks, and audit artifacts.

Rules for optional external-tool examples:

1. Each example must prove a single point: an external CLI or service can be constrained by a SkillRun contract.
2. Preflight boundaries are mandatory: allowlists, dry-run defaults, or sensitive content checks.
3. Audit artifacts are mandatory.
4. The example must not be described as a general adapter, marketplace item, trusted package, or sandboxed execution.
5. Optional external-tool examples should remain rare. If maintenance cost exceeds value, mark the example deprecated or remove it.
