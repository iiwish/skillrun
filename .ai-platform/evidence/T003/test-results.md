# T003 Test Results

Date: 2026-05-11

## RED

Command:

```powershell
cargo test --test manifest
```

Result: failed as expected.

Key output:

```text
running 2 tests
test manifest_fails_when_action_is_missing ... FAILED
test manifest_generates_yaml_with_hashes_and_pydantic_schemas ... FAILED
```

## GREEN

Command:

```powershell
cargo test --test manifest
```

Result: passed.

Key output:

```text
running 3 tests
test manifest_fails_when_action_is_missing ... ok
test manifest_metadata_extraction_times_out ... ok
test manifest_generates_yaml_with_hashes_and_pydantic_schemas ... ok
test result: ok. 3 passed; 0 failed
```

## Full Validation

Command:

```powershell
cargo test
```

Result: passed.

Key output:

```text
running 3 tests
test help_lists_planned_mvp_commands ... ok
test unimplemented_planned_commands_fail_until_implemented ... ok
test version_uses_approved_project_name ... ok

running 4 tests
test init_requires_python_flag ... ok
test init_rejects_path_like_capsule_names ... ok
test init_python_creates_standard_capsule ... ok
test init_refuses_non_empty_target ... ok

running 2 tests
test manifest_fails_when_action_is_missing ... ok
test manifest_metadata_extraction_times_out ... ok
test manifest_generates_yaml_with_hashes_and_pydantic_schemas ... ok
```

Command:

```powershell
cargo run -- init refund --python --output tmp\e2e-manifest
cargo run -- manifest --cwd tmp\e2e-manifest\refund
```

Result: passed.

Key output:

```text
created tmp\e2e-manifest\refund
generated tmp\e2e-manifest\refund\.skillrun\manifest.generated.yaml
```

## Governance Validation

Command:

```powershell
python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --feature-id mvp --task-id T003
```

Result: passed.

Command:

```powershell
python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2
```

Result: passed.
