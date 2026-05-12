# T011 Test Results

Date: 2026-05-12

## RED

- `cargo test --test e2e_matrix --test business_examples`
  - Result: failed as expected before implementation.
  - Signal: `examples/refund` did not exist and docs/README did not yet state the `.skr` dependency boundary required by the business example test.

## GREEN

- `cargo test --test e2e_matrix --test business_examples`
  - Result: passed.
  - Evidence: 3 passed, 0 failed.

- `cargo test --test errors`
  - Result: passed.
  - Evidence: 4 passed, 0 failed after aligning `ValidationError.recoverable` with A006.

## REFACTOR / FULL VALIDATION

- `cargo fmt -- --check`
  - Result: passed.

- `git diff --check`
  - Result: passed.

- `cargo test`
  - Result: passed.
  - Evidence: 46 integration tests passed across CLI, init, manifest, inspect, runtime, errors, artifacts, permissions, consumer guards, instruction-only, MCP, pack, E2E matrix, and business examples.

## Command-Level E2E Validation

- `cargo run -- init refund --python --output tmp/e2e-full`
  - Result: passed.

- `cargo run -- manifest --cwd tmp/e2e-full/refund`
  - Result: passed.

- `cargo run -- inspect --cwd tmp/e2e-full/refund`
  - Result: passed.

- `cargo run -- test --cwd tmp/e2e-full/refund`
  - Result: passed.

- `cargo run -- run --cwd tmp/e2e-full/refund --input examples/default.input.json`
  - Result: passed.

- `cargo run -- serve --mcp --cwd tmp/e2e-full/refund --dry-run`
  - Result: passed.

- `cargo run -- pack --cwd tmp/e2e-full/refund`
  - Result: passed.
  - Evidence: generated `tmp/e2e-full/refund/dist/refund-0.1.0.skr` and printed that `.skr` does not vendor dependencies.

## Governance Validation

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T011`
  - Result: passed.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
  - Result: passed.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.

## Rereview Validation

- `cargo fmt -- --check`
  - Result: passed during T011 rereview.

- `git diff --check`
  - Result: passed during T011 rereview.

- `cargo test --test e2e_matrix --test business_examples --test errors`
  - Result: passed during T011 rereview.
  - Evidence: 7 passed, 0 failed.

- `cargo test`
  - Result: passed during T011 rereview.
  - Evidence: 46 integration tests passed.

- Full command chain during T011 rereview:
  - `cargo run -- init refund --python --output tmp/e2e-full`: passed.
  - `cargo run -- manifest --cwd tmp/e2e-full/refund`: passed.
  - `cargo run -- inspect --cwd tmp/e2e-full/refund`: passed.
  - `cargo run -- test --cwd tmp/e2e-full/refund`: passed.
  - `cargo run -- run --cwd tmp/e2e-full/refund --input examples/default.input.json`: passed.
  - `cargo run -- serve --mcp --cwd tmp/e2e-full/refund --dry-run`: passed.
  - `cargo run -- pack --cwd tmp/e2e-full/refund`: passed.

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2 --task-id T011`
  - Result: passed during T011 rereview.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.

- `python D:\data\ai-rd-skill\ai-delivery-governor\scripts\validate_delivery_artifacts.py --root D:\data\skillrunv2`
  - Result: passed during T011 rereview.
  - Evidence: `ok: delivery artifacts passed lightweight validation`.
