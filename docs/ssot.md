# SkillRun SSOT: SOP-backed Skill Capsule Runtime

**文档状态**：v0.1.0 architecture baseline；v0.2.0 是第一版 public release candidate，v0.1 未单独公开发布。
**核心定位**：SkillRun 是用 Rust 实现的本地 CLI/Core，把 `SOP + code + schema + examples + permissions` 编译成可检查、可测试、可运行、可分发 Skill Manifest。  
**一句话**：**用一份 SOP 和一个 Action，把业务经验变成 Agent 可调用、可验证、可分发的技能。**

---

## 1. 核心判断

Agent 时代的真实缺口不是“如何把函数暴露成工具”。FastMCP、手写 MCP server、OpenAI tools、LangChain tools 都能解决这个问题。

SkillRun 不抢这个战场。

SkillRun 解决的是另一个问题：

> **如何把一套业务经验和一个可执行动作绑定成可信的 AI Skill 单元。**

也就是说：

- FastMCP 把函数变成 tool。
- SkillRun 把 SOP-backed capability 变成 skill。
- MCP 是 SkillRun 的一种对外接口，不是 SkillRun 的全部价值。

SkillRun 的核心用户不是“只想把 Python 函数给大模型调用”的开发者。那类用户应该直接用 FastMCP。

SkillRun 的目标用户是：

- 想把团队 SOP 变成 Agent 可复用能力的 AI engineer。
- 想把业务规则、脚本、测试样例和权限边界打包分发的平台工程师。
- 想让 Agent 调用工具时不丢失业务上下文的产品/业务系统团队。
- 想在本地、CI、MCP client 和后续 Hub 中复用同一份技能资产的开发者。

### 1.1 与现有 Skill 的关系

SkillRun 不替代 Skill，也不重新定义所有 Skill。

正确分层：

| 层 | 职责 | 默认是否可执行 |
| --- | --- | --- |
| Skill | 给人和 Agent 读的任务方法、流程、上下文和资源 | 否 |
| SkillRun Capsule | 一个 Skill 的可运行封装形态 | 是，前提是存在显式 Action |
| Action | Capsule 内被明确绑定的执行入口 | 是 |
| Manifest | 从 Skill、Action、schema、examples、permissions 编译出的运行 IR | 否 |

普通 Skill 仍然可以只是 instruction-only。只有作者显式加入 `action.py` / `action.mjs`、schema、examples 和可生成 Manifest 的边界后，它才成为 SkillRun Capsule。

SkillRun 必须遵守三条兼容规则：

- 不要求所有 Skill 都变成可执行对象。
- 不把 `SKILL.md` 当成 runtime config 或脚本入口。
- 不扫描 `scripts/`、`references/`、`assets/` 或 Markdown code block 后隐式执行。

因此，SkillRun-enabled Skill 是 Skill 的增强形态，不是替代形态。Skill 负责认知层，SkillRun 负责运行层；运行层不能反过来污染认知层。

---

## 2. 非目标

SkillRun 明确不做：

- 不做 FastMCP 替代品。
- 不做通用 Agent framework。
- 不做任意 Markdown 自动执行器。
- 不做 Skill npm / marketplace 的第一版。
- 不做 OpenAPI-to-MCP 的包装器主卖点。
- 不做企业 AI 中台。
- 不把 YAML 暴露成默认作者体验。
- 不从 stdout 猜测结构化结果。
- 不承诺自然语言 SOP 自动变成硬约束。

这些边界比功能列表更重要。边界不清，项目会退化成“脚本包装器 + MCP 导出器 + 伪安全 YAML”。

---

## 3. 产品原子：Skill Capsule

SkillRun 的原子单元不是单个 Markdown 文件，也不是一段代码，而是 **Skill Capsule**。

一个 Skill Capsule 对用户心智是一个技能；对内部工程可以是多文件目录。

最小作者形态：

```text
refund/
  SKILL.md
  action.py
  examples/
    default.input.json
  skillrun.config.json        # optional
  .skillrun/
    manifest.generated.yaml   # generated
    runs/
    artifacts/
```

对外心智：

```text
refund = refund policy SOP + executable refund action + examples + permissions
```

关键原则：

> **用户心智原子化，内部工程不强行单文件化。**

单文件 `.run.md` 可以作为未来 authoring syntax sugar，但不能成为 runtime 安全边界。

---

## 4. 三层架构

SkillRun 分为 Core、Adapter、SDK 三层。Core 必须使用 Rust 实现；Adapter 可以桥接 Python、Node 等用户 Action 生态，但不能把 SkillRun 本体变成对应语言的应用。

| 层 | 职责 | 不做什么 |
| --- | --- | --- |
| SkillRun Core | 读取 Manifest、分配 IPC 路径、运行进程、校验权限、生成 MCP tool、记录 run/artifact | 不内嵌 Python runtime，不理解业务 SOP |
| Language Adapter | 从语言生态提取 metadata，包装运行入口，连接 Core IPC | 不决定产品语义，不绕过 Manifest |
| Language SDK | 给开发者提供 `Context`、`preflight`、typed input/output、artifact helper | 不变成应用框架 |

Core 只认 Manifest 和 IPC。

语言知识全部关进 Adapter。开发者体验全部通过 SDK 降低摩擦。

---

## 5. Manifest 是 IR，不是用户主入口

`manifest.generated.yaml` 是 SkillRun 的中间表示与运行清单。

它不是默认让用户手写的源文件。

生成位置：

```text
.skillrun/manifest.generated.yaml
```

Manifest 顶部必须声明来源：

```yaml
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
```

Manifest 包含：

- skill metadata。
- SOP hash 和摘要。
- tool description。
- input JSON Schema。
- output JSON Schema。
- runtime adapter。
- command。
- declared permissions。
- preflight capability。
- IPC protocol version。
- examples。
- artifact policy。
- error envelope support。

原则：

- Author Mode 可以重新生成 Manifest。
- Consumer Mode 只读取 Manifest，不动态 import 源码。
- Manifest 缺失或 source hash 不匹配时，Consumer Mode fail closed。
- 不做双向同步。
- `.run.md`、CLI、GUI 未来都只能编译到 Manifest；Manifest 不反向生成 authoring source。

---

## 6. Author Mode 与 Consumer Mode

SkillRun 必须明确区分作者体验和消费安全。

### 6.1 Author Mode

Author Mode 面向本地开发者，优化 Time-to-First-Value。

允许：

- 通过 convention 发现 `SKILL.md` 和 `action.py` / `action.mjs`。
- 通过受控 adapter metadata 子进程提取本地代码 schema。
- 自动生成 `.skillrun/manifest.generated.yaml`。
- 快速运行 `skillrun test`、`skillrun inspect`、`skillrun serve --mcp`。

约束：

- metadata phase 不注入 secrets。
- metadata phase 有 timeout。
- metadata phase 应尽量禁网、限制写目录。
- action module 顶层必须 side-effect free。

### 6.2 Consumer Mode

Consumer Mode 面向使用别人分发的 skill。

只允许：

- 读取已生成 Manifest。
- 根据 Manifest 暴露 MCP tool。
- 由 Rust Core 按 Manifest 启动 adapter 子进程。
- 校验 IPC output、artifact、permissions。

禁止：

- 为提取 schema 而动态 import 未信任源码。
- 在 Manifest 缺失或过期时猜测运行方式。
- 以 stdout 作为成功结果兜底。

一句话：

> **Zero-config 只对本地作者体验成立；零信任消费场景只认静态 Manifest。**

---

## 7. Convention 与 Override

v0 不引入 `eject`。

完整暴露 Manifest 会制造后向兼容孤儿项目。用户只想改 timeout，不应该接管整套 IR。

v0 使用：

```text
Convention defaults
+ skillrun.config.json override
+ adapter defaults
= .skillrun/manifest.generated.yaml
```

最小 override：

```json
{
  "runtime": {
    "timeout": "60s",
    "env": ["STRIPE_API_KEY"]
  },
  "permissions": {
    "network": ["api.stripe.com"]
  }
}
```

未来可以支持 `SKILL.md` frontmatter override，但 v0 优先使用 `skillrun.config.json`，避免污染 SOP 文本。

---

## 8. Language Convention

SkillRun Core 不内置所有语言。

v0.1/v0.2 只交付 Rust Core + Python Action blessed path。Node/JS path 是第一个扩展目标，必须在 Rust Core、Manifest、IPC、pack 和 MCP 暴露路径被 Python Action slice 验证后再进入实现。

v0.3 可以把 JS Action Alpha 作为第一个 adapter generalization，但边界必须很窄：稳定作者路径是 canonical ESM `action.mjs`，不是完整 TypeScript 工具链。

CLI 语言语义必须分阶段：

- `init` 是模板选择，必须显式传 `--python` / `--py` / `--js`。
- `--py` 只是 `--python` 的短别名，生成相同 Python capsule。
- `skillrun init refund` 不设置隐式默认语言。
- `manifest` 是 Author Mode 编译，先看 `skillrun.config.json`，再用唯一 action 文件 convention。
- `test`、`run`、`serve --mcp`、`pack` 是 Consumer Mode 路径，只读 Manifest，不接受语言选择 flag。

长期 blessed paths：

| 文件 | Adapter |
| --- | --- |
| `action.py` | Python adapter |
| `action.mjs` | Node adapter，v0.3 JS Action Alpha |
| `action.ts` | 不作为 v0.3 稳定 runtime 入口；作者可自行编译到 `action.mjs` |

探测规则必须浅、确定、可解释：

- `skillrun.config.json` 显式声明 `runtime.adapter` / `runtime.entrypoint` 时优先使用 config。
- 无 config runtime override 时，找到唯一 `action.py`：使用 Python adapter。
- 无 config runtime override 时，v0.2 阶段找到唯一 `action.mjs`：提示 Node adapter 尚未启用。
- 无 config runtime override 时，v0.3 JS Action Alpha 启用后找到唯一 `action.mjs`：使用 Node adapter。
- 找到 `action.ts`：不直接运行；提示作者先编译到 `action.mjs`，或等待后续 TypeScript support 设计。
- 多个候选文件：报 ambiguous，不猜。
- 其他语言：post-MVP 要求 `skillrun.config.json` 显式指定 command，或暂不支持。

原则：

> **Zero-config 不是猜遍全世界，而是给每种语言一条唯一、稳定、可解释的黄金路径。**

---

## 9. Schema 策略

Schema 来源优先级：

```text
code-native schema > lightweight config schema > examples validation only
```

不能从 `examples/` 反推 schema 作为主方案。反推 schema 会低质，导致 Agent 频繁传错参数。

### 9.1 Python

Python 使用 Pydantic 作为推荐 schema 来源。

示例：

```python
from typing import Literal
from pydantic import BaseModel
from skillrun import Context, Artifact

class Input(BaseModel):
    order_id: str
    reason: Literal["damaged", "duplicate", "wrong_item"]

class Output(BaseModel):
    status: str
    amount: int
    receipt: Artifact | None = None

def preflight(input: Input, ctx: Context):
    ctx.require_skill_hash()

def run(input: Input, ctx: Context) -> Output:
    ...
```

### 9.2 Node

Node 使用 Zod 作为推荐 schema 来源。

示例：

```js
import { z } from "zod";
import { artifact } from "@skillrun/sdk";

export const input = z.object({
  order_id: z.string(),
  reason: z.enum(["damaged", "duplicate", "wrong_item"])
});

export const output = z.object({
  status: z.string(),
  amount: z.number(),
  receipt: artifact().nullable()
});

export async function preflight(input, ctx) {
  ctx.requireSkillHash();
}

export async function run(input, ctx) {
  ...
}
```

### 9.3 Lightweight Config Schema

不想引入 Pydantic/Zod 时，允许在 `skillrun.config.json` 中声明极简 input/output：

```json
{
  "input": {
    "order_id": "string",
    "reason": ["damaged", "duplicate", "wrong_item"]
  },
  "output": {
    "status": "string",
    "amount": "number"
  }
}
```

这不是完整 JSON Schema 替代品，只服务 v0 的低摩擦路径。

---

## 10. Metadata Extraction

如果 schema 写在图灵完备代码里，SkillRun 要提前拿到 schema，只有两类方案：

- 静态解析 AST：脆弱，不可靠。
- 动态加载 metadata：真实可行，但必须限制场景。

SkillRun 采用 adapter metadata phase：Rust Core 启动对应语言的 metadata 子进程，读取 `action.py` 或未来的 `action.mjs`，把 schema 和 metadata 写回 Manifest 生成流程。具体 argv 是 Rust Core 的实现细节，不作为 v0.1 稳定用户接口。

规则：

- 只在 Author Mode 自动执行 metadata phase。
- Consumer Mode 不动态加载源码。
- metadata phase 只提取 schema、function metadata、SDK capabilities，不执行业务 `run`。
- metadata phase 不注入 secrets。
- metadata phase 有 timeout。
- Manifest 记录 source hashes。

这不是完美安全，而是正确分层：作者信任自己的本地代码；消费者只信任已生成 Manifest。

必须明确：Consumer Mode 的 v0 安全收益是“不为 metadata import 未信任源码”和“按 Manifest 限制 env、IPC、artifact 边界”，不是完整 sandbox。运行别人分发的 action 仍然意味着执行其代码；真正的强隔离属于后续 sandbox / signed package / reproducible runtime 阶段。

---

## 11. SOP 绑定机制

`SKILL.md` 不是 README。它是 Skill Capsule 的 Cognitive Contract。

但自然语言 SOP 不会自动变成硬约束。SkillRun 只提供三层绑定：

| 层 | 实现 | 强度 |
| --- | --- | --- |
| Description | 将 SOP 摘要、禁忌、前置条件编译进 tool description | 软约束 |
| Schema | 将关键决策字段结构化进 input/output schema | 半强制 |
| Preflight | 用户通过 SDK 实现 `preflight(input, ctx)` | 硬约束 |

核心原则：

> **不要相信 Agent 会主动读 Resource；必须把 SOP 编译进 tool 行为边界。**

MCP Resource 仍然暴露 `SKILL.md`，但不能依赖 Agent 自觉读取。

Manifest 必须包含：

- `skill_hash`
- SOP summary
- tool description generated from SOP
- resource reference
- preflight flag

运行记录必须包含：

- `skill_hash`
- manifest hash
- action source hash
- preflight result

---

## 12. MCP Tool 生成

SkillRun 生成的 MCP tool description 不能只是脚本说明。

它必须包含：

- Action 做什么。
- 适用场景。
- 禁止场景。
- 关键 SOP 摘要。
- 必要时提示 Agent 读取 Resource。
- 输入字段含义。
- 失败时如何恢复。

示例：

```text
Execute the refund decision skill.

This tool is bound to refund/SKILL.md sha256:...
Use it only to decide whether a refund should be approved under the current policy.
Do not commit money movement unless the returned decision allows it.
If the tool returns PolicyViolation, follow the llm_hint instead of retrying blindly.
```

Tool description 是软约束。真正的业务边界必须由 schema 和 preflight 承担。

---

## 13. Runtime IPC

SkillRun 不解析 stdout。

stdout/stderr 只作为日志。

结构化输入、输出和 artifact 通过文件 IPC。

Rust Core 创建：

```text
SKILLRUN_CONTEXT_JSON
SKILLRUN_INPUT_JSON
SKILLRUN_OUTPUT_JSON
SKILLRUN_ARTIFACT_DIR
```

Adapter 负责：

- 读取 `SKILLRUN_CONTEXT_JSON`。
- 读取 `SKILLRUN_INPUT_JSON`。
- 调用 SDK 包装的 `preflight` 和 `run`。
- 将结果写入 `SKILLRUN_OUTPUT_JSON`。
- 将 artifact 写入 `SKILLRUN_ARTIFACT_DIR`。

SDK 负责让开发者只需要：

```python
def run(input: Input, ctx: Context) -> Output:
    ...
```

而不是手写 IPC 文件协议。

---

## 14. Output Envelope

所有成功结果必须写入 `SKILLRUN_OUTPUT_JSON`。

成功 envelope：

```json
{
  "ok": true,
  "result": {
    "status": "success",
    "amount": 100
  },
  "artifacts": [
    {
      "name": "receipt",
      "kind": "pdf",
      "path": "receipt.pdf",
      "mime": "application/pdf"
    }
  ],
  "display": {
    "markdown": "Refund completed."
  }
}
```

Core 必须校验：

- output file 存在。
- `ok` 是布尔值。
- `result` 符合 output schema。
- artifact path 位于 `SKILLRUN_ARTIFACT_DIR` 内。
- artifact 文件存在。
- artifact hash、size、mime 可记录。

如果 output file 缺失或格式错误：

```text
ProtocolViolation
```

绝不把 stdout 当作成功结果兜底。

---

## 15. Error Envelope

错误必须面向 Agent 可恢复，而不是只面向人类调试。

业务代码可以通过 SDK 抛出 structured error。

错误 envelope：

```json
{
  "ok": false,
  "error": {
    "code": "PolicyViolation",
    "message": "Refund amount exceeds the policy limit of 500.",
    "llm_hint": "Ask the user for manager approval before retrying.",
    "recoverable": true,
    "details": {
      "limit": 500,
      "requested_amount": 1200
    }
  },
  "display": {
    "markdown": "Refund blocked by policy. Manager approval is required."
  }
}
```

错误类型：

| Code | 含义 | Agent 行为 |
| --- | --- | --- |
| `ValidationError` | 输入不符合 schema | 修改参数后重试 |
| `PolicyViolation` | SOP/preflight 拒绝 | 按 `llm_hint` 询问用户或补充审批 |
| `PermissionDenied` | Manifest 权限不足 | 不应重试，提示配置问题 |
| `DependencyError` | 外部依赖失败 | 可稍后重试或报告依赖失败 |
| `ProtocolViolation` | Adapter/SDK 未遵守 IPC | 不应重试，报告 skill 实现错误 |
| `RuntimeError` | 未分类运行失败 | 报告失败，保留日志 |

原则：

- Stack trace 只能进入 debug logs。
- MCP tool result 应优先返回 structured error。
- `llm_hint` 是给 Agent 的恢复建议，不是给终端用户的最终话术。

---

## 16. Artifact

Artifact 是一等公民，不是 stdout 附属品。

Artifact 必须通过 output envelope 声明，并写入 `SKILLRUN_ARTIFACT_DIR`。

Artifact record：

```json
{
  "artifact_id": "art_...",
  "run_id": "run_...",
  "name": "receipt",
  "kind": "pdf",
  "mime": "application/pdf",
  "path": ".skillrun/runs/run_.../artifacts/receipt.pdf",
  "sha256": "...",
  "size": 12345
}
```

v0 支持 artifact kinds：

- `json`
- `markdown`
- `html`
- `pdf`
- `text`
- `file`

后续可扩展：

- `docx`
- `image`
- `diagram`
- `changeset`
- `review-report`
- `task-graph`

---

## 17. Run Record

每次执行生成 run record。

```text
.skillrun/runs/
  run_20260511_000001/
    input.json
    output.json
    context.json
    stdout.log
    stderr.log
    run.json
    artifacts/
```

`run.json`：

```json
{
  "run_id": "run_20260511_000001",
  "skill": "refund",
  "skill_hash": "...",
  "manifest_hash": "...",
  "action_hash": "...",
  "adapter": "python",
  "status": "completed",
  "started_at": "...",
  "ended_at": "...",
  "preflight": {
    "ran": true,
    "status": "passed"
  },
  "artifacts": ["art_..."]
}
```

---

## 18. Permissions

v0 权限模型必须小而硬。

Manifest 声明：

```yaml
permissions:
  files:
    read: []
    write:
      - ".skillrun/runs/**"
  network:
    outbound:
      - "api.stripe.com"
  env:
    read:
      - "STRIPE_API_KEY"
```

Core 最少要做：

- 只向子进程注入声明过的 env。
- 为 run 分配专属 artifact dir。
- 校验 artifact path 不越界。
- 在 inspect 中展示声明权限。
- 在 run record 中记录使用的声明权限。

v0 不承诺完整 sandbox，但必须从第一天建立正确模型。

---

## 19. CLI

v0.1 MVP 命令：

```bash
skillrun init refund --python
skillrun manifest
skillrun inspect
skillrun test
skillrun run --input examples/default.input.json
skillrun serve --mcp
skillrun pack
```

命令职责：

| Command | 作用 |
| --- | --- |
| `init` | 生成最小 Skill Capsule |
| `manifest` | Author Mode 下生成或刷新 Manifest |
| `inspect` | 展示 SOP、tool、schema、permissions、adapter、examples |
| `test` | 使用 examples 运行并校验 IPC/output/artifacts |
| `run` | 执行一次真实输入 |
| `serve --mcp` | 从 Manifest 暴露 MCP tool |
| `pack` | 生成可分发 skill package |

---

## 20. Distribution Primitive

v0 必须提供第一种分发体验。

不能只说“传 Git 仓库”。Git 仓库是开发协作形态，不是 skill 分发原语。

v0 使用：

```bash
skillrun pack
```

生成：

```text
dist/refund-0.1.0.skr
```

`.skr` 是 tar.gz archive，包含：

```text
SKILL.md
action.py
skillrun.config.json          # if exists
.skillrun/manifest.generated.yaml
examples/
README.md                     # optional generated summary
```

v0 不打包完整 Python/Node 依赖环境。

依赖策略：

- Python Action capsule：允许包含 `requirements.txt` 或 capsule-local `pyproject.toml`。
- Node：允许包含 `package.json` 和 lockfile。
- Consumer 安装时由 adapter 检查依赖是否满足。

未来再考虑：

- signed package
- private registry
- dependency vendoring
- reproducible runtime image

Consumer Mode 加载 `.skr` 时：

- 解包到本地 skill cache。
- 读取 Manifest。
- 校验 source hashes。
- 不动态 import 源码提取 metadata。
- 运行时只按 Manifest 由 Rust Core 调用 adapter。

---

## 21. MCP 行为

`skillrun serve --mcp` 基于 Manifest 暴露：

- tools：每个 capsule v0 默认一个 primary action。
- resources：`SKILL.md`、examples、artifact references。

v0 不主打多 action 编排。

如果需要多 action，应优先拆成多个 Skill Capsule，除非它们共享同一个 SOP 和生命周期。

原则：

> **外部一个稳定 job-to-be-done，内部可以复杂。**

---

## 22. MVP

v0.1 MVP 只验证一件事：

> **一个 SOP-backed Skill Capsule 可以被生成 Manifest、inspect、test、run、serve as MCP、pack 分发。**

最小成功路径：

```bash
skillrun init refund --python
cd refund
# edit SKILL.md
# edit action.py
skillrun manifest
skillrun test
skillrun serve --mcp
skillrun pack
```

最小验收标准：

- `init` 由 Rust CLI 生成可运行 Python Action capsule。
- `manifest` 从 Pydantic 提取 input/output schema。
- `inspect` 展示 SOP hash、schema、permissions、adapter、tool description。
- `test` 通过 `examples/default.input.json`，生成 output envelope 和 run record。
- `serve --mcp` 使用 Manifest 暴露 tool，不重新 import 源码提取 schema。
- `pack` 生成 `.skr`，包含 Manifest 和 source hashes。
- ProtocolViolation 不被 stdout 兜底掩盖。
- Preflight error 以 Error Envelope 返回。

---

## 23. 推荐叙事

不要主打：

- One decorator MCP。
- Executable Markdown。
- YAML action runtime。
- OpenAPI-to-MCP。
- Skill marketplace。

主打：

> **Turn one SOP and one action into a tested MCP skill.**

中文：

> **用一份 SOP 和一个 Action，生成一个可测试、可检查、可给 Agent 调用的技能。**

更完整：

> **SkillRun compiles SOP-backed code into a manifest-driven skill package with typed input/output, preflight checks, structured artifacts, LLM-oriented errors, and MCP exposure.**

中文：

> **SkillRun 将绑定 SOP 的代码编译成 Manifest 驱动的技能包，提供类型化输入输出、前置校验、结构化产物、面向 Agent 的错误恢复，以及 MCP 调用接口。**

---

## 24. 最终模型

```text
Skill Capsule = SOP + action code + schema + examples + overrides
Manifest = compiled runtime IR
Core = Rust manifest-driven runtime and MCP server
Adapter = language bridge
SDK = low-friction developer API
IPC = file-based input/output/artifact protocol
Package = immutable .skr distribution artifact
```

SkillRun 的核心不是“让文档能执行”，也不是“让脚本变 MCP tool”。

它的核心是：

> **让业务经验以 Skill Capsule 的形式被编译、验证、运行和分发。**
