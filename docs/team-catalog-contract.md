# Team Catalog Contract Draft

**文档状态**：Partially Implemented
**最后更新**：2026-05-26

## 目标

Team Catalog 是 Team Library 的 Core 前置合同：让团队可以用一个可审查的 catalog 文件描述要分发给成员的 AI 能力，同时让 Desktop 和自动化只消费 SkillRun Core 的稳定 JSON surface。

第一阶段目标很窄：

- catalog 描述团队可发现的能力。
- `.skr` item 可以进入 plan / install / update 路径。
- install / update 仍复用现有 `.skr` import / `import --replace` 语义。
- Desktop 不直接下载、解包、复制或执行能力。

这份文档是 Team Catalog 的分阶段合同。当前已实现 `inspect`、`install plan` 和本地 `file` source 的 `install apply` 最小 Core surface；HTTPS download 仍待后续显式 downloader。

## 非目标

- 不做 Public Marketplace。
- 不做 SkillRun 托管服务。
- 不自动安装 Python、Node、npm、pip 或系统依赖。
- 不远程执行未知代码。
- 不引入 daemon、后台同步或自动更新。
- 不把 checksum 说成作者身份或信任证明。
- 不承诺 OS sandbox、signed package trust、provenance 或 publisher identity。
- 不让 Desktop 越过 Core 稳定 JSON surface。
- 不把 plain Agent Skill 或 MCP server 目录直接变成 runnable SkillRun capsule。

## 三层边界

```text
Team Catalog
  发现与分发描述：团队有哪些能力、版本、来源、checksum、requirements、trust note

SkillRun Core
  校验与生命周期：catalog inspect、install plan/apply、import、replace、registry、switchboard

Desktop
  可视化与确认：展示 catalog、展示 plan、让用户显式安装、启用、挂载
```

Team Catalog 不替代 Manifest。运行时权威仍来自 `.skr` 内的 Manifest、adapter metadata、preflight、schema、envelope 和 run evidence。

## Catalog 文件

建议文件 schema version：

```json
{
  "schema_version": "team.catalog.v1",
  "catalog_id": "acme.internal",
  "name": "Acme AI Capabilities",
  "updated_at": "2026-05-26T10:00:00Z",
  "items": []
}
```

约束：

- `schema_version` 必须等于 `team.catalog.v1`。
- `catalog_id` 是 catalog 本身的稳定 id，不是 trust identity。
- `updated_at` 必须是 RFC3339 时间，用于展示 freshness，不作为安全依据。
- `items` 可以为空。
- unknown fields 应被保留 / 忽略，不应导致老客户端无法读取 catalog。

Draft JSON Schema：[`contracts/team-catalog.draft.schema.json`](contracts/team-catalog.draft.schema.json)。

## Item 类型

第一阶段只允许 `kind = "skillrun.skr"` 进入 install / update 路径。

```json
{
  "id": "refund",
  "kind": "skillrun.skr",
  "name": "Refund Decision",
  "description": "Evaluate refund requests with policy limits and approval preflight.",
  "version": "0.1.0",
  "publisher": {
    "name": "Acme Operations"
  },
  "source": {
    "type": "https",
    "url": "https://example.com/skillrun/refund-0.1.0.skr",
    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
  },
  "requirements": [
    {
      "kind": "python",
      "summary": "Python 3.11+"
    }
  ],
  "permissions_summary": [
    "reads provided refund input",
    "writes run-local receipt artifact"
  ],
  "mcp": {
    "exposes_tools": true,
    "mount": "skillrun-router"
  },
  "trust_note": "Internal operations example. Review before enabling.",
  "tags": ["ops", "refund"]
}
```

Required item fields:

- `id`：catalog 内稳定 item id；建议与 import id 一致。
- `kind`：第一阶段只支持 `skillrun.skr` install。
- `name`
- `description`
- `version`
- `source.type`
- `source.url`

Installable `https` source 必须提供 `source.sha256`。`file` source 可以在本地开发 catalog 中缺省，但 Core 应在 plan 中给出 warning；团队分发 catalog 应提供 checksum。

Optional display fields:

- `publisher.name`
- `publisher.url`
- `homepage`
- `repository`
- `requirements`
- `permissions_summary`
- `mcp`
- `trust_note`
- `tags`

## Source 类型

第一阶段建议支持：

- `file`：本地 `.skr` 路径。
- `https`：HTTPS `.skr` 下载 URL。

暂不支持：

- Git clone。
- GitHub API 私有 release 自动认证。
- npm / PyPI / package manager source。
- archive 内多 capsule 自动发现。
- catalog 递归引用另一个 catalog。

`sha256` 对 `https` source 必填。`file` source 也应支持 `sha256`，但本地开发 catalog 可以允许缺省并显示 warning；如果进入团队分发，应要求 checksum。

## Proposed Core Commands

当前实现：

```bash
skillrun team catalog inspect <catalog> --json
skillrun team catalog install plan <catalog> <item-id> --json
```

```bash
skillrun team catalog install apply <catalog> <item-id> --json
```

原则：

- `inspect` 只读取 catalog，校验 schema，汇总 item，不下载或执行 item。
- `install plan` 可以解析目标 item，检查 source 类型、checksum 是否存在、当前 registry 是否已有同 id entry，并给出将调用 import 还是 import --replace。
- `install apply` 才可以读取 `.skr`，必须先校验 `sha256`，再调用现有 import / import --replace 语义。
- 当前 `install apply` 仅支持 `file` source；`https` source fail closed，等待后续显式 Core downloader。
- `install apply` 不自动 `switchboard enable`。
- `install apply` 不自动 mount MCP client。
- `install apply` 不自动安装 host dependencies。
- `install apply` 不运行 action、test、validate 或 MCP server。

## Proposed JSON Surfaces

### Inspect

JSON Schema：[`contracts/team-catalog-inspect.schema.json`](contracts/team-catalog-inspect.schema.json)。

```json
{
  "command": "team catalog inspect",
  "schema_version": "team.catalog.inspect.v1",
  "ok": true,
  "catalog": {
    "catalog_id": "acme.internal",
    "name": "Acme AI Capabilities",
    "updated_at": "2026-05-26T10:00:00Z",
    "items": 1
  },
  "items": [
    {
      "id": "refund",
      "kind": "skillrun.skr",
      "name": "Refund Decision",
      "version": "0.1.0",
      "installable": true,
      "warnings": []
    }
  ],
  "error": null
}
```

### Install Plan

JSON Schema：[`contracts/team-catalog-install-plan.schema.json`](contracts/team-catalog-install-plan.schema.json)。

```json
{
  "command": "team catalog install plan",
  "schema_version": "team.catalog.install_plan.v1",
  "ok": true,
  "catalog_id": "acme.internal",
  "item": {
    "id": "refund",
    "kind": "skillrun.skr",
    "version": "0.1.0",
    "source_type": "https",
    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
  },
  "registry": {
    "installed": false,
    "source_type": null
  },
  "actions": [
    {
      "type": "import",
      "replace": false
    }
  ],
  "warnings": [
    {
      "code": "trust.not_proven",
      "message": "sha256 verifies integrity, not publisher identity."
    }
  ],
  "error": null
}
```

### Install Apply

JSON Schema：[`contracts/team-catalog-install-apply.schema.json`](contracts/team-catalog-install-apply.schema.json)。

```json
{
  "command": "team catalog install apply",
  "schema_version": "team.catalog.install_apply.v1",
  "ok": true,
  "catalog_id": "acme.internal",
  "item_id": "refund",
  "download": {
    "source_type": "file",
    "package_path": "/path/to/refund-0.1.0.skr",
    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "sha256_verified": true
  },
  "import": {
    "schema_version": "import.v1",
    "id": "refund",
    "path": "/path/to/.skillrun/imports/refund",
    "source_type": "imported_skr",
    "enabled": false,
    "replaced": false
  },
  "next_steps": [
    "skillrun switchboard enable refund",
    "skillrun router status --json",
    "skillrun mount plan --client claude-desktop --json"
  ],
  "error": null
}
```

## Error Codes

Suggested error code families:

- `catalog.read_failed`
- `catalog.schema_unsupported`
- `catalog.schema_invalid`
- `catalog.item_not_found`
- `catalog.item_not_installable`
- `catalog.source_unsupported`
- `catalog.source_checksum_required`
- `catalog.download_failed`
- `catalog.sha256_mismatch`
- `catalog.registry_read_failed`
- `catalog.registry_conflict`
- `catalog.replace_refused`
- `catalog.import_failed`

`catalog.registry_conflict` 应在以下场景 fail closed：

- catalog item `id` 已存在，但 registry entry 是 `local_path`。
- catalog item `id` 已存在，但 existing source 不允许 `import --replace`。
- 用户未显式传入允许 replace 的参数，且 plan 判断会替换已安装 `.skr`。

## Desktop Consumption Rules

Desktop Team Library 必须：

- 调用 Core 的 `team catalog inspect --json` 展示 catalog。
- 调用 Core 的 plan surface 展示 install / update 影响。
- 只在用户确认后调用 apply。
- 安装完成后引导用户显式 enable 和 mount。
- 把 `sha256` 展示为 integrity check，不展示为 trust proof。
- 展示 `trust_note`、requirements、permissions summary 和 warnings。

Desktop Team Library 不得：

- 直接读取 `.skillrun/` 私有目录。
- 直接下载 `.skr` 并自行解包 import。
- 对 plain Agent Skill 或 MCP server item 执行本地命令。
- 自动 enable、自动 mount 或自动安装依赖。
- 自行实现签名 / provenance / publisher identity 判断。

## Relationship To Existing Commands

第一阶段应复用既有生命周期：

```text
catalog item -> .skr source -> sha256 verify -> skillrun import [--replace]
             -> registry entry -> switchboard enable -> router serve --mcp
```

Team Catalog 不应绕开：

- `skillrun import <package.skr> --json`
- `skillrun import <package.skr> --replace --json`
- `skillrun registry`
- `skillrun switchboard`
- `skillrun router status --json`
- `skillrun mount plan/apply`

## Open Questions

- 是否使用顶层 `team catalog ...`，还是 `catalog ...`。
- 是否允许 catalog 级 `default_client_mount`，还是只在 Desktop 中处理 client 选择。
- `file` source 缺少 `sha256` 时是否允许 install apply，还是只允许 inspect。
- 未来签名 / provenance 应属于 catalog metadata、`.skr` metadata，还是独立 trust store。
