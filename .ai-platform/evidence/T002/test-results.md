# T002 Test Results

Date: 2026-05-11

## RED

Command:

```powershell
cargo test --test init
```

Result: failed as expected.

Key output:

```text
running 4 tests
test init_python_creates_standard_capsule ... FAILED
test init_rejects_path_like_capsule_names ... FAILED
test init_requires_python_flag ... FAILED
test init_refuses_non_empty_target ... FAILED
```

## GREEN

Command:

```powershell
cargo test --test init
```

Result: passed.

Key output:

```text
running 4 tests
test init_rejects_path_like_capsule_names ... ok
test init_requires_python_flag ... ok
test init_refuses_non_empty_target ... ok
test init_python_creates_standard_capsule ... ok
test result: ok. 4 passed; 0 failed
```

## Regression And Validation

Command:

```powershell
cargo test --test cli
```

Result: passed.

Key output:

```text
running 3 tests
test unimplemented_planned_commands_fail_until_implemented ... ok
test help_lists_planned_mvp_commands ... ok
test version_uses_approved_project_name ... ok
test result: ok. 3 passed; 0 failed
```

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
test init_rejects_path_like_capsule_names ... ok
test init_requires_python_flag ... ok
test init_refuses_non_empty_target ... ok
test init_python_creates_standard_capsule ... ok
```

Command:

```powershell
cargo run -- init refund --python --output tmp\e2e-init
```

Result: passed.

Key output:

```text
created tmp\e2e-init\refund
```

Command:

```powershell
cargo run -- --help
```

Result: passed.

Key output:

```text
Implemented:
  init --python
```

## Governance Validation

Command:

```powershell
python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --feature-id mvp --task-id T002
```

Result: passed.

Command:

```powershell
python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2
```

Result: passed.
