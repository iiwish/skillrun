# 测试策略

SkillRun 的测试策略围绕 Manifest 驱动的运行契约展开：CLI 输出、capsule 生成、Manifest、runtime、错误结构、artifact 边界、consumer guard、MCP 暴露和 package 行为都需要可重复验证。

## 本地基线

提交前优先运行：

```bash
cargo fmt --check
cargo test
```

涉及 lint-sensitive 或共享 runtime 代码时运行：

```bash
cargo clippy --all-targets -- -D warnings
```

涉及 CLI、Manifest、runtime、MCP 或 packaging 行为时，至少运行对应测试文件或全量测试：

```bash
cargo test --test cli
cargo test --test manifest
cargo test --test runtime
cargo test --test mcp_server
cargo test --test pack
```

## Release Validation

release candidate 至少需要：

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- README 中列出的核心 `cargo run -- ...` 命令抽样验证
- release notes 和 release policy 检查

## 测试设计原则

- 优先验证用户可观察行为，而不是内部实现细节。
- 对 fail-closed 行为写回归测试：缺失 Manifest、stale Manifest、instruction-only skill、artifact escape、结构化错误等。
- 对每个新增 adapter 或 Manifest 字段补 contract 测试。
- 不把 stdout 当结构化成功来源；测试必须检查 output/error envelope。
- 新增 fixtures 应保持小而可读，不引入无关生成产物。
