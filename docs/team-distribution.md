# Team Distribution Route

**文档状态**：product route baseline
**最后更新**：2026-05-26

## 目标不变

SkillRun 的目标不变：它仍然是可执行 Agent Skills 的 runtime / contract / evidence 层。

路线调整在于外部入口和产品叙事：

```text
底层能力：Skill Capsule / .skr / Manifest / Router / run evidence
外部入口：团队分发 MCP tools 和 Agent Skills
```

`.skr` 继续是分发 artifact，不是用户必须先理解的新标准。对新用户更容易理解的入口是：

```text
把团队的 MCP tools 和 Agent Skills 分发、安装、更新、挂载到本地 AI 客户端。
```

## 为什么支持团队分发

直接解释 `.skr`、Manifest 和 executable skill runtime 的认知成本较高。团队分发有更直接的使用场景：

- 团队里已经有人写了内部脚本、SOP、MCP tool 或 Agent Skill。
- 普通成员只想安装、检查、启用和挂载，不想理解打包细节。
- 团队负责人需要知道每个人安装的是哪个版本，能力声明了什么权限，能否被本地 MCP client 调用。
- 作者需要一种比复制文件夹、README 或手写 MCP 配置更稳定的交付方式。

因此，团队分发应该成为 go-to-market route；Skill Capsule 仍然是底层产品原子。

## 产品心智

面向用户：

```text
Install team-shared AI skills and MCP tools into your local AI client.
```

面向团队：

```text
Package once, share with the team, mount into any MCP client.
```

面向作者：

```text
Turn SOP-backed scripts into inspectable, testable, evidenced Skill Capsules.
```

中文表达：

```text
团队 AI 能力分发器：把 MCP tools、Agent Skills 和可执行 SOP 能力打包、检查、分发、安装并挂载到每个人的本地 AI 客户端。
```

## 三层模型

```text
Agent Skills
  authoring / discovery layer
  SKILL.md, scripts, references, assets

SkillRun
  runtime / contract / evidence layer
  Manifest, adapters, preflight, envelopes, run records, .skr

MCP
  invocation surface
  skillrun router serve --mcp
```

团队分发不是第四个 runtime。它是围绕这三层建立的消费和安装体验。

## 第一阶段：Team Library，不做 Public Marketplace

第一阶段应该支持团队库，而不是公开市场。

Team Library 可以来自：

- 本地 catalog 文件。
- GitHub repository 或 GitHub Release asset。
- 团队内部静态文件服务器。
- 后续可选的私有 artifact storage。

第一阶段不做：

- 公开搜索 marketplace。
- SkillRun 托管第三方能力。
- 支付、评分、推荐或生态排名。
- 远程代码自动执行。
- 自动安装 Python、Node、npm package 或系统依赖。
- OS sandbox 或“安全执行任意第三方代码”的承诺。

## 体验目标

Desktop 最终应该支持一个 Team Library 页面：

- 添加团队 catalog。
- 浏览团队能力。
- 查看能力类型：Agent Skill、MCP tool、Skill Capsule。
- 查看版本、作者、来源、checksum、更新时间和说明。
- 查看权限、host requirements、MCP exposure preview 和 trust notes。
- 安装 `.skr`。
- 更新已安装 capsule。
- 启用 / 禁用本地 exposure intent。
- 通过 Router 挂载到 MCP client。
- 跳转到 run evidence explorer。

Core 应该先提供稳定、可自动化的底层合同，再让 Desktop 消费。Desktop 不能直接读取 catalog 私有缓存或 `.skillrun/` 内部结构。

## Catalog 方向

后续可以设计 `team.catalog.v1`，但本文件不冻结具体 JSON contract。

一个 catalog item 至少需要表达：

- `id`
- `name`
- `description`
- `kind`：例如 `skill_capsule`、`agent_skill`、`mcp_server`
- `version`
- `publisher`
- `source`
- `download_url`
- `sha256`
- `homepage` / `repository`
- `tags`
- `requirements`
- `permissions_summary`
- `mcp_mount_hint`
- `trust_note`

关键原则：

- catalog 只描述可发现和可下载的条目，不是信任证明。
- `sha256` 是完整性检查，不是作者身份认证。
- signed capsule / provenance / trust metadata 是后续高风险设计议题。
- 如果 catalog 指向 `.skr`，安装仍应走 `skillrun import` / `import --replace`。
- 如果 catalog 指向 plain Agent Skill 或 MCP server，必须先定义明确的包装、导入或挂载边界，不能让 Desktop 绕过 Core 执行任意目录。

## 信任边界

团队分发必须诚实表达当前能力：

- SkillRun 可以让能力可检查、可打包、可版本化、可留证。
- SkillRun 当前不让任意第三方代码自动变安全。
- `registry` 是本地 inventory，不是 trust store。
- `switchboard enabled=true` 是本地 exposure intent，不是信任证明。
- `mount apply` 挂载的是 SkillRun Router，不是 `.skr` 文件。
- Host requirements 只诊断当前机器环境，不自动安装依赖，不使用 Docker。

任何涉及签名、provenance、publisher identity、远程更新通道或 sandbox 的实现，都必须作为单独高风险设计通过 review gate。

## 推荐推进顺序

1. 官网先调整叙事：主入口从“理解 `.skr`”转成“团队分享 MCP tools 和 Agent Skills”，但保留 SkillRun runtime / evidence 的真实边界。
2. Core 设计 team catalog inspect / install 的合同草案，先只处理 `.skr` item 和本地 / HTTPS catalog metadata。
3. Desktop 增加 Team Library 页面，消费 Core 稳定 JSON surface，不自行实现 trust 或安装逻辑。
4. 再考虑签名、provenance、publisher identity 和 stronger trust distribution。

## 非目标

- 不改变 SkillRun 的核心目标。
- 不把 SkillRun 讲成 Agent Skills 替代标准。
- 不把 `.skr` 直接暴露为 MCP runtime entry。
- 不把团队库称为 marketplace。
- 不承诺 OS sandbox。
- 不引入隐藏 daemon。
- 不自动安装依赖。
- 不让 Desktop 越过 Core 边界。
