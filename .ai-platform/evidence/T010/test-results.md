# T010 Test Results

Date: 2026-05-12

## RED

- `cargo test --test pack`
  - Result: failed as expected before implementation.
  - Signal: 2 failed, 1 passed. Valid pack cases failed with `command not implemented yet: pack`; stale Manifest guard already failed closed before fallback.

## GREEN

- `cargo test --test pack`
  - Result: passed.
  - Evidence: 3 passed, 0 failed before rereview hardening; 4 passed, 0 failed after adding invalid Manifest package-name coverage.

- `cargo test --test consumer_guards`
  - Result: passed.
  - Evidence: 4 passed, 0 failed.

## REFACTOR / FULL VALIDATION

- `cargo fmt -- --check`
  - Result: passed.

- `git diff --check`
  - Result: passed.

- `cargo test`
  - Result: passed.
  - Evidence: 42 integration tests passed before rereview hardening.

- `cargo run -- init refund --python --output tmp/e2e-init`
  - Result: passed.
  - Evidence: created `tmp/e2e-init/refund`.

- `cargo run -- manifest --cwd tmp/e2e-init/refund`
  - Result: passed.
  - Evidence: generated `tmp/e2e-init/refund/.skillrun/manifest.generated.yaml`.

- `cargo run -- pack --cwd tmp/e2e-init/refund`
  - Result: passed.
  - Evidence: generated `tmp/e2e-init/refund/dist/refund-0.1.0.skr` and printed that `.skr` does not vendor dependencies.

## Governance Validation

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T010`
  - Result: passed.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
  - Result: passed.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.

## Rereview Validation

- `cargo test --test pack`
  - Result: passed during T010 rereview.
  - Evidence: 4 passed, 0 failed, including invalid Manifest package-name rejection.

- `cargo test --test consumer_guards`
  - Result: passed during T010 rereview.
  - Evidence: 4 passed, 0 failed.

- `cargo fmt -- --check`
  - Result: passed during T010 rereview.

- `git diff --check`
  - Result: passed during T010 rereview.

- `cargo test`
  - Result: passed during T010 rereview.
  - Evidence: 43 integration tests passed after adding invalid Manifest package-name coverage.

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T010`
  - Result: passed during T010 rereview.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
  - Result: passed during T010 rereview.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.
