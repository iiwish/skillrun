# SkillRun MVP: Rust-first SkillRun Runtime

**文档状态**：Confirmed  
**版本**：v0.1.0 MVP contract；v0.1 未单独公开发布，v0.2.0 是第一版 public release candidate，v0.3.0 是 JS Action Alpha 的本地 release handoff
**来源**：`docs/ssot.md`  
**最后更新**：2026-05-11  
**审批记录**：2026-05-11 用户批准进入 Plan / Work Graph 阶段；项目名称统一为 SkillRun / `skillrun`。

---

## 1. MVP 判断

SkillRun MVP 只验证一个判断：

> **用 Rust 实现的 SkillRun，可以把一份 SOP 和一个显式 Action 编译成 Manifest 驱动的 Skill Capsule，并完成 inspect、test、run、serve as MCP、pack 分发。**

MVP 不证明 SkillRun 是通用 Action Runtime，不证明它能替代 FastMCP，不证明它能治理所有企业 API。MVP 只证明一个更尖锐的能力：

> **业务经验可以从文档和代码的松散组合，变成可测试、可检查、可运行、可分发的 Agent skill 单元。**

### 1.1 实现语言边界

SkillRun 本体必须使用 Rust 实现，包括 CLI、Core、Manifest 读写、IPC 编排、Consumer Mode guard、MCP 暴露和 `.skr` pack。MVP 的首个可运行 Action 格式是 Python `action.py`，这只是用户 skill 的 adapter 目标，不是 SkillRun 本体实现语言。

---

## 2. 产品切口

### 2.1 首个用户

MVP 只服务一个核心用户：

- 熟悉命令行、SOP 工程化和 Python Action 编写的 AI engineer / platform engineer。
- 已经有一份团队 SOP。
- 已经有或愿意写一个可执行 action。
- 想把这项能力给 Agent、MCP client、CI 或其他本地流程复用。

### 2.2 首个使用场景

推荐示例：`refund`。

原因：

- SOP 明确：退款政策、限制、禁忌、审批边界。
- Action 明确：输入订单与原因，输出决策或错误。
- 风险明确：不能把自然语言政策当成硬约束。
- 能展示 preflight、schema、structured error、artifact、run record。

### 2.3 一句话叙事

> **Turn one SOP and one action into a tested, manifest-bound Skill Capsule with a Rust CLI/Core.**

中文：

> **用 Rust 实现的 SkillRun，把一份 SOP 和一个显式 Action 生成一个可测试、可检查、可给 Agent 调用、可打包分发的技能。**

### 2.4 与普通 Skill 的兼容边界

MVP 不接管普通 Skill。一个只有 `SKILL.md`、`references/`、`assets/` 或 `scripts/` 的目录仍然是 instruction-only Skill，不是 SkillRun Capsule。

MVP 行为约束：

- `SKILL.md` 只提供 SOP 和认知上下文，不是 runtime config。
- `scripts/` 中的文件不会因为存在而自动变成 Action。
- Markdown code block 不会被扫描或执行。
- 没有 `action.py` 和有效 Manifest 的目录不能 `run`、`serve --mcp` 或 `pack`。
- `skillrun inspect` 遇到 instruction-only Skill 时，应明确提示：这是普通 Skill，不是可运行 Capsule。

这条边界比兼容性声明更重要。它防止 SkillRun 把 Skill 生态从“可读能力说明”污染成“潜在可执行目录”。

---

## 3. MVP 非目标

MVP 明确不做：

- Node adapter。
- OpenAPI-to-MCP。
- MCP proxy / MCP composition。
- HTTP server。
- schedule / workflow。
- marketplace / registry。
- signed package。
- dependency vendoring。
- reproducible runtime image。
- 完整 sandbox。
- 多 action 编排。
- GUI。
- `.run.md` authoring sugar。
- 从 examples 反推高质量 schema。
- 让自然语言 SOP 自动变成硬约束。

这些不是永远不做，而是不能进入首个可验证切口。

---

## 4. 标准 Capsule

MVP 的唯一标准目录：

```text
refund/
  SKILL.md
  action.py
  examples/
    default.input.json
  skillrun.config.json        # optional
  .skillrun/
    manifest.generated.yaml
    runs/
    artifacts/
```

`SKILL.md` 是认知契约。  
`action.py` 是唯一可执行入口。  
`manifest.generated.yaml` 是运行 IR。  
`.skillrun/runs/` 是执行证据。  
`.skillrun/artifacts/` 或 run-local `artifacts/` 是产物边界。  

---

## 5. 用户路径

### 5.1 最小成功路径

```bash
skillrun init refund --python
cd refund
# edit SKILL.md
# edit action.py
skillrun manifest
skillrun inspect
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp
skillrun pack
```

### 5.2 每一步的产品承诺

| 命令 | 用户问题 | MVP 承诺 |
| --- | --- | --- |
| `init` | 怎么开始 | Rust CLI 生成可运行 Python Action capsule |
| `manifest` | 这个 skill 的机器契约是什么 | Rust Core 从 SOP、Pydantic metadata、config、examples 生成 Manifest |
| `inspect` | 它会做什么、读写什么、需要什么权限 | 展示 SOP hash、schema、adapter、permissions、examples、MCP tool 摘要 |
| `test` | 这个 skill 是否可控运行 | 使用默认 example 执行并校验 output envelope、artifact、run record |
| `run` | 用真实输入跑一次 | 产生结构化 output、日志、run record 和 artifacts |
| `serve --mcp` | Agent 能否调用 | 基于 Manifest 暴露 MCP tool，不重新 import 源码提取 schema |
| `pack` | 能否分发 | 生成 `.skr` 包，包含 Manifest 和 source hashes |

### 5.3 Agent 调用前学习路径

SkillRun Capsule 必须能被 AI assistant 在调用前快速学习，而不是只被人类阅读。公开文档和示例应提供一段可直接交给 Agent 的学习说明。链接或 repo path 必须指向具体 Capsule 文件夹，不能只指向项目首页。

```text
请先学习这个 SkillRun Capsule，再使用它：
<capsule-folder-url-or-repo-path>

1. 阅读 SKILL.md，理解 purpose、SOP、prohibited behavior、required context 和 recovery guidance。
2. 阅读 skillrun.config.json 和已生成的 Manifest，确认 adapter 与 entrypoint。
3. 只把 action.py 或 action.mjs 当成该 capsule 的 action contract；不要推断未声明语言或 package-manager 行为。
4. 阅读 examples/default.input.json，理解调用时需要的输入形态。
5. 如果你能访问工作区，运行 `skillrun inspect --cwd <capsule>`、`skillrun doctor --cwd <capsule>` 和 `skillrun test --cwd <capsule>`。
6. 调用 MCP tool 时，不要从 stdout 推断成功。只看 output/error envelope、artifacts 和 run record。
```

该模块的产品承诺是：用户可以把一个 Capsule 文件夹链接直接交给 Agent。这个链接可以是官网学习页，也可以是包含 `SKILL.md`、`skillrun.config.json`、`action.py` 或 `action.mjs`、`examples/` 的 GitHub 文件夹。Agent 先学习 `SKILL.md` 的认知契约、Manifest/配置中的 adapter 边界、action entrypoint 的输入输出和 preflight、`examples/` 的调用形态；有本地环境时再运行 `inspect`、`doctor` 和 `test`。它不能绕过 schema、preflight、output/error envelope、artifact 和 run record。

---

## 6. 核心功能需求

### FR-001：初始化 Capsule

`skillrun init refund --python` 必须生成：

- `SKILL.md`，包含可编辑 SOP 模板。
- `action.py`，包含 Pydantic `Input`、`Output`、`preflight` 和 `run` 示例。
- `examples/default.input.json`，能驱动默认 test。
- `skillrun.config.json`，可选；若生成，必须只包含最小 override 示例。

验收：

- 初始化后无需新增文件即可运行 `skillrun manifest`。
- 默认示例不能依赖真实外部 API、真实密钥或网络。

### FR-002：生成 Manifest

`skillrun manifest` 必须：

- 读取 `SKILL.md`。
- Rust Core 在 Author Mode 通过受控 Python metadata 子进程读取本地 `action.py`，提取 Pydantic input/output schema。
- 读取可选 `skillrun.config.json`。
- 生成 `.skillrun/manifest.generated.yaml`。
- 写入 `SKILL.md`、`action.py`、`skillrun.config.json` 的 source hash。

验收：

- Manifest 缺少 source hash 时失败。
- `SKILL.md` 或 `action.py` 修改后，旧 Manifest 在 Consumer Mode 下被判定 stale。
- metadata phase 不注入用户 secrets。

### FR-003：Inspect

`skillrun inspect` 必须展示：

- skill name。
- SOP hash。
- SOP summary。
- input JSON Schema。
- output JSON Schema。
- adapter：`python`。
- runtime command。
- declared permissions。
- examples。
- preflight 是否存在。
- MCP tool description 摘要。

验收：

- inspect 不执行业务 `run`。
- inspect 输出不能要求用户阅读 Manifest 才理解核心风险。

### FR-004：Test

`skillrun test` 必须：

- 使用 `examples/default.input.json`。
- 创建一次 test run。
- 执行 `preflight` 和 `run`。
- 校验 output envelope。
- 校验 artifact path 不越界。
- 生成 run record。

验收：

- stdout 只能进入日志，不能作为成功结果兜底。
- output file 缺失时返回 `ProtocolViolation`。
- Pydantic validation 失败时返回 `ValidationError`。

### FR-005：Run

`skillrun run --input examples/default.input.json` 必须：

- 校验 input schema。
- 创建 run-local input、context、output、stdout、stderr、artifact 目录。
- 注入 Manifest 声明过的 env。
- Rust Core 启动 Python Action adapter 子进程。
- 校验 output schema 和 artifacts。
- 输出机器可读结果，并保留人类可读 display markdown。

验收：

- 成功返回 `ok: true` envelope。
- 失败返回 `ok: false` error envelope。
- 每次 run 都有唯一 run id。

### FR-006：Structured Error

MVP 必须支持这些错误码：

| Code | 必需场景 |
| --- | --- |
| `ValidationError` | input 不符合 schema |
| `PolicyViolation` | preflight 或业务代码拒绝 |
| `PermissionDenied` | env 或 artifact 权限不满足 |
| `ProtocolViolation` | adapter 未写合法 output |
| `RuntimeError` | 未分类运行失败 |

验收：

- error envelope 包含 `code`、`message`、`recoverable`。
- `llm_hint` 可选，但 `PolicyViolation` 推荐提供。
- stack trace 只能进入 debug logs。

### FR-007：MCP 暴露

`skillrun serve --mcp` 必须：

- 只从 Manifest 暴露 tool。
- 不重新 import `action.py` 提取 schema。
- 暴露 `SKILL.md` 为 MCP resource。
- 生成包含 SOP summary、禁止场景、输入字段含义、错误恢复建议的 tool description。

验收：

- Manifest stale 时 fail closed。
- MCP tool input schema 来自 Manifest。
- MCP tool 调用结果使用 output/error envelope。

### FR-008：Pack

`skillrun pack` 必须生成：

```text
dist/refund-0.1.0.skr
```

`.skr` 是 tar.gz archive，包含：

- `SKILL.md`
- `action.py`
- `skillrun.config.json`，如果存在
- `.skillrun/manifest.generated.yaml`
- `examples/`
- `README.md`，可选生成

验收：

- pack 前 Manifest 必须存在且 source hashes 匹配。
- `.skr` 不包含 `.skillrun/runs/`。
- `.skr` 不承诺内置依赖环境。

### FR-009：Instruction-only Skill Guard

当目标目录只有 `SKILL.md`，或只有 `SKILL.md` 加 `references/`、`assets/`、`scripts/`，但没有 `action.py` 和有效 Manifest 时，SkillRun 必须把它当作 instruction-only Skill。

验收：

- `skillrun inspect` 可以展示 instruction-only 状态和原因。
- `skillrun manifest` 不得从 Markdown、scripts 或 examples 猜测 Action。
- `skillrun run` 必须拒绝执行，并说明需要显式 `action.py`。
- `skillrun serve --mcp` 不得暴露 tool。
- `skillrun pack` 不得生成 `.skr`。

---

## 7. Manifest 最小字段

MVP Manifest 至少包含：

```yaml
manifest_version: "0.1.0"
generated_by: "skillrun@0.1.0"
generated_at: "2026-05-11T00:00:00Z"
sources:
  skill:
    path: "SKILL.md"
    sha256: "..."
  action:
    path: "action.py"
    sha256: "..."
  config:
    path: "skillrun.config.json"
    sha256: "..."
skill:
  name: "refund"
  sop_summary: "..."
  skill_hash: "..."
tool:
  name: "refund"
  description: "..."
schemas:
  input: {}
  output: {}
runtime:
  adapter: "python"
  entrypoint: "action.py"
  host: "skillrun-rust"
  timeout: "30s"
permissions:
  files:
    read: []
    write: [".skillrun/runs/**"]
  network:
    outbound: []
  env:
    read: []
ipc:
  protocol_version: "0.1.0"
examples:
  - id: "default"
    input: "examples/default.input.json"
artifacts:
  allowed_kinds: ["json", "markdown", "html", "pdf", "text", "file"]
errors:
  envelope: true
```

---

## 8. IPC 合同

Rust Core 为 adapter 子进程注入：

```text
SKILLRUN_CONTEXT_JSON
SKILLRUN_INPUT_JSON
SKILLRUN_OUTPUT_JSON
SKILLRUN_ARTIFACT_DIR
```

约束：

- adapter 必须从 `SKILLRUN_INPUT_JSON` 读取输入。
- adapter 必须向 `SKILLRUN_OUTPUT_JSON` 写入 envelope。
- artifact 必须写入 `SKILLRUN_ARTIFACT_DIR` 内部。
- stdout/stderr 只作为日志。
- output file 缺失、非法 JSON、schema 不匹配都属于 `ProtocolViolation`。

---

## 9. Output 合同

### 9.1 成功

```json
{
  "ok": true,
  "output": {
    "status": "approved"
  },
  "artifacts": [],
  "display": {
    "markdown": "Refund approved."
  }
}
```

### 9.2 失败

```json
{
  "ok": false,
  "error": {
    "code": "PolicyViolation",
    "message": "Refund amount exceeds the policy limit.",
    "recoverable": true,
    "llm_hint": "Ask for manager approval before retrying."
  },
  "display": {
    "markdown": "Refund blocked by policy."
  }
}
```

---

## 10. 权限模型

MVP 权限只做小而硬的边界：

- 只向子进程注入 Manifest 声明过的 env。
- 为每次 run 创建独立 artifact dir。
- 校验 artifact path 不越界。
- inspect 展示声明权限。
- run record 记录声明权限。

MVP 不承诺：

- 阻止被运行的 Python Action 读取任意本地文件。
- 阻止被运行的 Python Action 访问网络。
- OS 级 sandbox。
- 签名验证。
- 供应链隔离。

必须在文档、CLI warning 和 pack/install 流程中诚实表达这个边界。

---

## 11. 测试策略与业务示例

MVP 测试设计以 `docs/testing.md` 为准。旧的 A001-A013 简表不再作为唯一测试设计，只保留为顶层 release gate；每个 A-case 必须下钻到 Unit、Contract、Integration、Negative/Security、E2E 和 Business Example 的具体测试。

### 11.1 测试分层

| Layer | 目的 | 典型覆盖 |
| --- | --- | --- |
| Unit | 验证纯逻辑和边界判断 | hashing、Manifest model、schema extraction、path containment、envelope validation |
| Contract | 验证外部契约稳定 | CLI exit code/output、Manifest 最小字段、MCP dry-run schema、`.skr` archive list |
| Integration | 验证模块协作 | `init -> manifest -> inspect`、runtime IPC、Consumer Mode guard、pack/unpack inspect |
| Negative/Security | 验证可信边界 | stale Manifest、stdout 假成功、artifact traversal、env 注入、instruction-only guard |
| E2E Acceptance | 验证 MVP 用户路径 | A001-A013 全矩阵 fresh command evidence |
| Business Examples | 证明业务价值 | refund hero example，以及 `docs/business-examples.md` 中 support triage、access request、vendor risk 的文档级示例 |

### 11.2 顶层验收矩阵

| ID | 场景 | 必须断言 |
| --- | --- | --- |
| A001 | 初始化 capsule | 标准文件存在；默认 example 无网络/密钥；重复初始化非空目录失败 |
| A002 | 生成 Manifest | Manifest 含 schema、source hashes、permissions、adapter、tool description；hash 与文件一致 |
| A003 | Inspect runnable capsule | 不执行 `run`；展示 SOP hash、schema、permissions、adapter、examples、preflight、MCP 摘要 |
| A004 | 默认测试成功 | 生成 `ok: true` envelope、run record、stdout/stderr logs；run id 唯一 |
| A005 | 真实运行成功 | output file 存在；output 符合 schema；display markdown 存在；run record 可追溯到 Manifest hash |
| A006 | 输入非法 | 返回 `ValidationError`；`recoverable=true`；不调用业务 `run` |
| A007 | SOP/preflight 拒绝 | 返回 `PolicyViolation`；包含 `llm_hint`；stdout 不影响错误判断 |
| A008 | Adapter 协议违规 | 返回 `ProtocolViolation`；stdout 中出现成功文本也不能兜底 |
| A009 | Artifact 越界 | `../`、绝对路径、Windows drive path 或不存在文件均不得被记录为成功 artifact |
| A010 | Stale Manifest | 修改 `SKILL.md`、`action.py` 或 config 后，Consumer Mode fail closed |
| A011 | MCP 暴露 | tool schema 来自 Manifest；resource 指向 `SKILL.md`；不 import `action.py` 提取 metadata |
| A012 | Pack 分发 | `.skr` 包含 source 和 Manifest；不包含 `.skillrun/runs/`；解包后可 inspect |
| A013 | Instruction-only 保护 | inspect 展示 instruction-only；manifest/run/serve/pack 拒绝隐式执行 |

### 11.3 经典业务示例

MVP 必须实现一个完整可运行的 hero example，并用另外三个文档级经典示例证明 SkillRun 的业务价值边界。

| ID | 示例 | v0.1 责任 | 证明的业务价值 |
| --- | --- | --- | --- |
| B001 | Refund Decision | 完整实现 | SOP、schema、preflight、structured error 和 audit trail 共同约束 Agent 决策 |
| B002 | Support Triage | 文档级示例 | SOP summary、stable routing labels 和 missing-context recovery 避免 Agent 自由分流 |
| B003 | Access Request Approval | 文档级示例 | approval boundary、declared env 和 audit note 让权限类工作可控 |
| B004 | Vendor Risk Review | 文档级示例 | artifact 和 risk summary 证明 SkillRun 不只是 stdout wrapper |

### 11.4 Release Gate

MVP 完成前必须满足：

- A001-A013 全部有 fresh command evidence。
- B001 完整实现并通过 E2E。
- B002-B004 至少在 README 或 `docs/business-examples.md` 中以经典示例形式解释业务价值。
- Negative/Security Matrix 中的高风险边界有 automated tests 或明确 documented exception。
- `cargo test` 通过。
- `skillrun pack` 生成的 `.skr` 可解包并通过 Manifest inspect。

---

## 12. 实现顺序

### Milestone 1：Capsule 与 Manifest

- Rust CLI skeleton。
- `init --python`。
- Python template。
- Pydantic schema extraction。
- Manifest generation。
- source hash。

完成标准：

- `skillrun init` 后能生成 Manifest。

### Milestone 2：IPC 与 Run

- run directory。
- context/input/output 文件。
- Python adapter runtime。
- output/error envelope。
- stdout/stderr logs。
- artifact path 校验。

完成标准：

- `skillrun test` 能跑通默认 example。

### Milestone 3：Inspect 与 Failure Discipline

- inspect renderer。
- stale manifest 检测。
- protocol violation。
- validation error。
- policy violation。

完成标准：

- 测试矩阵 A003、A006、A007、A008、A010 通过。

### Milestone 4：MCP 与 Pack

- MCP server。
- tool generation from Manifest。
- `SKILL.md` resource。
- `.skr` pack。

完成标准：

- `serve --mcp` 和 `pack` 都不重新生成 Manifest。

---

## 13. 成功指标

MVP 成功不是 star 数，也不是功能数量。

必须满足：

- 新用户在 5 分钟内完成 `init -> test`。
- 示例 capsule 在干净环境中可重复生成 Manifest。
- MCP tool schema 只来自 Manifest。
- stdout 不会被当成成功输出。
- Manifest stale 时 Consumer Mode fail closed。
- `.skr` 能作为独立分发 artifact 被 inspect。
- 文档能让用户理解：SkillRun 不是 FastMCP 替代品。

---

## 14. 主要风险

| 风险 | 影响 | MVP 处理 |
| --- | --- | --- |
| metadata phase 运行 Python Action metadata 子进程 | 作者侧副作用 | 只在 Author Mode；不注入 secrets；加 timeout；文档明确信任边界 |
| 用户误以为有完整 sandbox | 安全误导 | CLI、docs、pack 明确 v0 非 sandbox |
| Pydantic 版本差异 | schema extraction 不稳定 | 固定支持 Pydantic v2；错误信息明确 |
| MCP 行为变化 | serve 不稳定 | MCP 层保持薄；核心测试不依赖 MCP |
| `.skr` 不含依赖环境 | 分发体验不完整 | 明确它是 skill package，不是 reproducible runtime |
| scope 膨胀到 OpenAPI / Node | MVP 延迟 | Node、OpenAPI、HTTP 全部 post-MVP |

---

## 15. Definition of Done

MVP 完成必须同时满足：

- `docs/ssot.md` 与本文件对 v0.1 MVP scope 一致。
- `docs/testing.md` 中 release validation 通过，并有 fresh command evidence。
- 默认 `refund` capsule 可以被初始化、测试、运行、MCP 暴露和打包。
- Consumer Mode 不从源码重新提取 metadata。
- `ProtocolViolation`、`ValidationError`、`PolicyViolation` 至少各有一个可复现测试。
- Negative/Security Matrix 中的高风险边界有 automated tests 或明确 documented exception。
- B001 `refund` hero example 完整实现，B002-B004 作为 README 或 `docs/business-examples.md` 级业务价值示例出现。
- `pack` 生成的 `.skr` 可被解包并通过 Manifest inspect。
- README 可以用同一条叙事解释项目：`Turn one SOP and one action into a tested, manifest-bound Skill Capsule with a Rust CLI/Core.`

---

## 16. User Review Gate

本 MVP 文档已于 2026-05-11 获得用户批准，状态为 `Confirmed`，可以进入 implementation plan / task breakdown。若要改变首版范围，只允许在以下两种方向中选择：

- 更窄：去掉 `pack` 或 `serve --mcp` 中的一个，保留 Core + Manifest + test/run。
- 更尖：保留当前范围，但只做一个高质量 `refund` 示例。

不建议更宽。更宽会把 SkillRun 拉回普通 MCP/action wrapper 战场。
