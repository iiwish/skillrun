# T047 Test Results

## `cargo fmt --check`

Status: passed.

## `git diff --check`

Status: passed.

## `cargo test --test business_examples`

Status: passed.

Summary:

```text
running 4 tests
test docs_explain_b001_to_b004_without_expanding_v0_runtime_scope ... ok
test refund_hero_example_proves_business_value_end_to_end ... ok
test wecom_team_notice_example_runs_locally_without_real_webhook ... ok
test v042_official_reference_capsules_run_without_registry_or_sandbox_claims ... ok
test result: ok. 4 passed; 0 failed
```

## `cargo test`

Status: passed.

Summary:

```text
All Rust unit and integration test suites passed for skillrun v0.4.2.
```

## `cargo clippy --all-targets -- -D warnings`

Status: passed.

Summary:

```text
Checking skillrun v0.4.2
Finished `dev` profile
```

## Delivery Artifact Validator

Command:

```text
python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T047
```

Status: passed with non-blocking warnings.

Summary:

```text
summary: 0 error(s), 4 warning(s), 0 info
```

The warnings are legacy search warnings for older spec packet locations (`mvp`, `v0.2`, `v0.3`, `v0.4`). The v0.4.2 packet exists at `.ai-platform/specs/v0.4.2-positioning-capsules/packets/T047.yaml`.

## Detailed Official Capsule Matrix

Status: passed.

The detailed matrix was run after the local commit on fresh temporary copies of each capsule, so generated manifests, run records and package archives did not modify repository examples.

Coverage:

- `commit_message_gate`: `manifest`, `inspect`, `check`, `test`, `serve --mcp --dry-run`, `pack`, default validation success, invalid message `PolicyViolation`, explicit `perform_commit=true` Git failure mapped to `DependencyError`.
- `bounded_file_patcher`: `manifest`, `inspect`, `check`, `test`, `serve --mcp --dry-run`, `pack`, default exact replacement success, sample file changed to `Status: reviewed`, blocked path `PolicyViolation`.
- `readonly_diagnostics_runner`: `manifest`, `inspect`, `check`, `test`, `serve --mcp --dry-run`, `pack`, `pwd`, `list` and `git_status` diagnostics return structured output with `exit_code`.

Summary:

```text
PASS base matrix: commit_message_gate
PASS base matrix: bounded_file_patcher
PASS base matrix: readonly_diagnostics_runner
PASS detail: commit_message_gate default, invalid policy, explicit git failure
PASS detail: bounded_file_patcher patch success and blocked path
PASS detail: readonly_diagnostics_runner pwd, list, git_status structured output
```
