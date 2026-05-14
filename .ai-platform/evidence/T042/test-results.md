# T042 Test Results

Commands run:

- `cargo run -- manifest --cwd examples/wecom_team_notice`: passed
- `cargo run -- test --cwd examples/wecom_team_notice`: passed with `decision=preview`
- `cargo run -- run --cwd examples/wecom_team_notice --input examples/urgent_requires_approval.input.json`: returned structured `PolicyViolation`
- `cargo run -- run --cwd examples/wecom_team_notice --input examples/send.input.json`: returned structured `DependencyError` without `WECOM_WEBHOOK_URL`
- `cargo run -- serve --mcp --cwd examples/wecom_team_notice --dry-run`: passed
