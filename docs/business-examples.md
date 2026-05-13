# SkillRun 经典业务示例

**文档状态**：Confirmed  
**版本**：v0.4.1 example catalog
**最后更新**：2026-05-13
**审批记录**：2026-05-11 文档审核无 blocking findings，作为 v0.1 业务示例目录继续推进；2026-05-13 增加 v0.4.1 `wecom_team_notice` 正式示例设计。

---

## 1. 示例定位

SkillRun 的业务价值不是“把 Python 函数暴露给 Agent”。它证明的是：

> 一份业务 SOP 可以和一个可执行 Action 绑定成可检查、可测试、可运行、可分发的 Skill Capsule。

本文件列出几个经典业务示例，用来解释 SkillRun 适合什么场景。v0.1 MVP 只要求完整实现 `refund`；v0.4.1 开始增加 `wecom_team_notice` 作为第一个更贴近日常工作流的正式可运行示例。换句话说：v0.1 MVP only implements the refund capsule；v0.4.1 adds an official WeCom team notice capsule without expanding SkillRun into a WeCom adapter。

---

## 2. B001: Refund Decision

**状态**：v0.1 hero example，必须完整实现。

### 业务问题

退款政策通常包含金额限制、原因分类、审批边界和禁忌动作。Agent 如果只读自然语言政策，很容易在缺少审批时继续执行。

### SkillRun 价值

SkillRun 把退款 SOP、typed input/output、preflight、structured error、run record 和 Manifest 绑定在一起，让 Agent 只能拿到结构化决策，而不是凭自由文本猜测是否能退款。

### Capsule 输入

- `order_id`
- `amount`
- `reason`
- `customer_tier`
- `manager_approval_id`

### Capsule 输出

- `decision`: `approved`、`rejected`、`needs_approval`
- `amount`
- `reasoning_summary`
- `audit_note`
- optional artifact: markdown decision receipt

### 必须证明

- 合规退款返回 `ok: true`。
- 超额或缺少审批返回 `PolicyViolation`。
- invalid reason 返回 `ValidationError`。
- tool description 明确禁止 Agent 在返回允许前移动资金。
- run record 可以追溯到 skill hash、manifest hash 和 action hash。

---

## 3. B002: Support Triage

**状态**：v0.1 docs-level example。

### 业务问题

客服 Agent 常常需要把 ticket 分流到退款、bug、账务、安全风险或人工支持。如果只靠 prompt，很容易出现标签漂移和升级标准不一致。

### SkillRun 价值

SkillRun 可以把客服 SOP 编译进 tool description、schema 和 preflight，确保输出是稳定 routing label，并在缺少上下文时返回可恢复错误。

### Capsule 输入

- `ticket_text`
- `customer_tier`
- `region`
- `product_area`

### Capsule 输出

- `route`: `refund`、`bug`、`billing`、`risk`、`human_support`
- `priority`
- `reasoning_summary`
- `required_follow_up`

### 证明点

- SOP summary 不依赖 Agent 自己去读长文档。
- schema 强制稳定分流标签。
- `llm_hint` 可以告诉 Agent 询问缺失的订单号、地区或客户等级。

---

## 4. B003: Access Request Approval

**状态**：v0.1 docs-level example。

### 业务问题

权限申请通常要求 ticket、manager approval、系统角色、数据级别和审计备注。Agent 如果绕过审批边界，风险很高。

### SkillRun 价值

SkillRun 用 preflight 把审批边界变成硬约束：没有 approval id、ticket id 或必要上下文时，Action 不应继续。

### Capsule 输入

- `requester`
- `system`
- `role`
- `data_access_level`
- `manager_approval_id`
- `ticket_id`

### Capsule 输出

- `decision`: `approved`、`blocked`、`needs_review`
- `required_approvals`
- `audit_note`

### 证明点

- `PolicyViolation` 不只是退款场景有价值，也适用于审批类工作。
- declared env 和 run record 支撑企业审计。
- SkillRun 可以把“允许做决策”和“执行外部副作用”分开。

---

## 5. B004: Vendor Risk Review

**状态**：v0.1 docs-level example。

### 业务问题

供应商准入需要按 SOP 评估数据访问级别、地区风险、安全问卷和人工复核边界。结果通常需要可审计摘要，而不是一行 stdout。

### SkillRun 价值

SkillRun 可以让 Action 产出结构化风险结论和 markdown/pdf artifact，把审核结果变成可追溯产物。

### Capsule 输入

- `vendor_name`
- `country`
- `data_access_level`
- `security_answers`
- `reviewer_notes`

### Capsule 输出

- `risk_level`: `low`、`medium`、`high`
- `decision`: `approved`、`blocked`、`review_required`
- `review_required`
- `summary_artifact`

### 证明点

- Artifact 是一等公民，不是 stdout 附属品。
- `.skr` 可以分发同一套审核 SOP 和 action。
- `.skr` does not vendor dependencies；它分发的是 source、Manifest 和 example contract，不是 runtime image。
- run record 让审核结果可复查。

---

## 6. v0.1 范围边界

- `refund` 必须实现为完整可运行 capsule。
- `support_triage`、`access_request_approval`、`vendor_risk_review` 只作为 README/docs 级示例。
- 这些业务示例不能要求额外 runtime scope：除 v0.3 已定义的 JS Action Alpha (`action.mjs`) 之外，不引入 Node/TypeScript 工具链、OpenAPI wrapper、HTTP server、schedule/workflow 或 marketplace scope。

---

## 7. B005: WeCom Team Notice

**状态**：v0.4.1 official runnable example。

### 业务问题

本地 Agent 很容易把“发企业微信通知”理解成裸工具调用：给一个 `text`，直接发送。这会丢掉团队通知 SOP、审批边界、禁发内容、dry-run 预览和审计记录。

### SkillRun 价值

SkillRun 把“团队通知发布 SOP”封装成一个 Skill Capsule：Agent 通过 Manifest-derived MCP tool 调用，用户可以先 dry-run 预览，再显式确认真实发送。`SKILL.md`、schema、`preflight`、declared env、artifact 和 run record 一起约束这个能力。

### Capsule 输入

- `title`
- `summary`
- `audience`: `team`、`project`、`incident`、`all_hands`
- `urgency`: `normal`、`high`、`critical`
- `dry_run`
- `approval_id`
- `mentioned_mobile_list`

### Capsule 输出

- `decision`: `preview`、`sent`、`blocked`
- `message_preview`
- `wecom_response`
- `audit_note`
- artifact: markdown notice receipt

### 证明点

- `dry_run=true` 不需要真实 webhook，适合 CI 和首次体验。
- `dry_run=false` 需要 declared env `WECOM_WEBHOOK_URL`。
- 高优先级、critical 或 all-hands 通知必须有 `approval_id`。
- 疑似 secrets 会返回 `PolicyViolation`。
- 缺少 webhook 会返回结构化 `DependencyError`。
- Agent 使用路径是 MCP tool，不是让 Agent 自己猜测 `skillrun run` 命令。
- 该示例不是企业微信 adapter、OpenAPI-to-MCP 或企业微信 CLI wrapper。
