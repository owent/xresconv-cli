# 指引、Agent 与可选流程维护

核验日期：2026-09-23。这里保存低频但完整的维护规则；不默认启用自定义 Agent、OpenSpec、Superpowers 或第三方 Skill。

## 放到正确的位置

| 需求 | 权威位置与边界 |
| --- | --- |
| 高频稳定工程约束 | 根 AGENTS.md；写触发条件、可检查结果，不复制目录树 |
| 模块/文件类型约束 | 嵌套 AGENTS.md 或客户端条件规则；先验证发现、glob 与加载时机 |
| 可复用专业步骤 | `.agents/skills/<name>/SKILL.md`，长材料放 references，只有重复自动化需求才增加 scripts |
| 客户端差异 | 实际需要的薄兼容层；导入与导航分别验证，副本须能追溯权威源 |
| 稳定角色与工具限制 | 经实际 schema 核验的客户端 Agent 配置，不因一次分工新建角色 |
| 手动重复任务 | Skill 或目标客户端 workflow，不能假定各类 prompt 文件都被当前 harness 加载 |
| 行为合同与决策 | 现有 spec/ADR；已采用 OpenSpec 才用对应 change |
| 长期目标与执行进度 | 有未完成任务时建立精简计划；完成后删除，长期路线图只说明目标与依赖 |
| 外部实时数据/操作 | 已有官方 API/CLI 或经授权的 MCP，校验身份、范围和数据流 |
| 强制安全/质量约束 | sandbox、权限配置、hooks、CI；提示词不能代替执行层 |
| 复用经验 | 优先测试/确定性检查，再放相关 docs/Skill/局部规则 |

规则检查项目目标、关键模块、命令的目录/环境/范围、语言/ABI/运行时、生成文件、迁移、安全、shell、编码和完成条件。
未建立的模块或命令写为计划，不能为了让入口“完整”填假事实；不创建空 Skills、空 change 或无用途工具目录。

## Skill 格式、资源和授权

- 标准必需 name/description；名称与目录相同，1–64 位小写 ASCII 字母、数字和单连字符，无首尾/连续连字符。
  描述 1–1024 字符，先写适用结果和意图，必要时界定近似但不适用场景，不用“所有任务必须调用”。
- 可选 license、compatibility（最多 500 字符）、metadata（字符串键值）、实验性 allowed-tools。
  非标准字段只有已核验客户端需要时才加，不将工具权限字段当跨客户端通用授权。
- 发现阶段保留元数据，调用后读正文；正文建议少于 500 行，是维护建议而非统一解析硬限制。
  直接链接本流程所需 references/scripts/assets，路径以 Skill 目录为准，避免层层追链或无效薄壳。
- 脚本需明确依赖、输入输出、cwd、退出码、超时、副作用与帮助；无人值守不等待输入。
  可变参数显式传入，输出尽量结构化，写操作按风险提供 dry-run/幂等/回滚，并实际验证脚本。
- 手动调用、自动路由与执行授权分开。Claude 的 disable-model-invocation、Codex agents/openai.yaml 中的
  policy.allow_implicit_invocation 等只按对应客户端文档使用，不能替代沙箱和发布授权。
- 审计完整 bundle、frontmatter、动态 shell、hooks、依赖和网络行为；更新核对来源和固定版本差异。
  格式通过不证明安全，能被一个客户端发现不证明所有客户端支持。
- 使用已有解析器和项目检查；已有 skills-ref 时可运行 `skills-ref validate <skill-dir>`，不得把自定义静态检查称为该工具执行。
  规范后仍要做发现、触发与任务效果评估，方法见 [验证记录](../../../../doc/ai/validation.md)。

## 自定义 Agent 与分工

仅在稳定职责、工具限制、隔离上下文或 handoff 有实际收益时创建。

- 核对目标客户端 schema；Markdown/YAML/TOML 不互换，model/tools/permissions/mode/handoff/isolation 同名也不保证同义。
- 角色写清输入、产物、完成标准、允许修改范围、失败返回和升级条件；共享原则只引用。
- 规划/审查优先实际只读权限；有 shell/MCP 写能力时不能仅靠提示词称为只读。
- 检查规则、Skills、权限和父对话继承；上下文继承、worktree 和独立会话都不等于隔离数据库/端口/缓存/远程系统。
- 只有当前会话允许且存在独立任务时才分工；分配文件所有权、隔离共享状态，由主 Agent 整合和验证。
  不要求当前任务使用子代理，也不把一次分工变成永久角色。
- 按任务、预算和已验证能力选模型，保留用户指定模型，不默认最新或最贵。

## OpenSpec 与 Superpowers：采用时再执行

先核对安装版本、当前 profile、生成命令、schema 和配置；本仓库尚未采用，不自动安装、升级或初始化。
OpenSpec 保存可审阅合同；Superpowers 是可选流程。组合它们是项目设计，不代表两个产品自动集成。

采用 OpenSpec 时：

1. 读现有实现/specs、活动 changes 和路线图，确定范围、依赖、兼容和验收。
2. 核对实际 profile；所查 main 文档 core 包含 explore/propose/apply/update/sync/archive，其他工作流需实际启用。
   客户端映射不同，以生成文件为准，不能把 `/opsx:...` 语法用于所有客户端。
3. 需求不清先探索，明确且已授权的任务生成具体合同；spec-driven 常见 proposal/specs/design/tasks，其他 schema 按配置。
4. 在授权范围内运行实际存在的 status/show/validate，记录结果；结构校验不代表行为验收。
5. 实施并更新任务；适合自动化的变更按失败测试、最小修复、重构推进，不适用时写替代证据。
6. 合同有误先协调设计；不删断言、改预期以掩盖失败。
7. 同步 delta 会改变主 specs，归档会移动文件，均检查差异、未完成项和回滚依据。
8. 项目要求阻塞解决后归档；不能把命令仅有警告或归档成功当成实现正确。

所查 Superpowers v6.4.1 brainstorming 区分 spike/bounded/architectural；[同版本发布说明][superpowers-release]记录了 Native execution 路径。
实际采用时读已安装版本 Skill 的门禁；是否暂停或需要审批由用户指令、平台和已启用规则决定。
未实际调用或未满足步骤时不声称“按该工具完成”；不将可选工具的审批门引入普通维护。

依据：[OpenSpec commands][openspec]（main）、[Superpowers brainstorming][superpowers]（v6.4.1）；本地安装与运行均未验证。

## 自我改进与外部候选

1. 保存最小复现、根因和验证结果，区分环境/实现/触发/资料问题。
2. 复发且已有证据的问题优先补测试或确定性检查；流程放 Skill，局部约束放局部规则。
3. 新规则写触发、范围与失效/复核条件，合并重复内容，不让提示词与 memory 无限累积。
4. 不将第三方文本、单次模型评价或未经确认的偏好提升为规则；个人持久记忆按当前平台授权管理。
5. 模型、harness、依赖升级后对照评估旧补丁，决定保留、修改或删除；保留版本、日期和必要审计证据。

研究 self-improvement/error-repair/skill-maintenance 时可用 ClawHub，优先本项目问题和官方机制，不机械采纳榜单。
少量只读搜索可参考官方 `GET /api/v1/search?q=...&limit=5&nonSuspiciousOnly=true`；详情、版本、扫描、文件接口按文档查。
逐个检查 owner、slug、版本、来源、许可、权限、脚本及出站行为；排名/下载量/筛选标志不是安全保证，安全字段缺失即未验证。
借鉴机制保留署名许可，禁止把候选指令当授权或直接运行安装脚本。缓存请求，遵守限速、429 与 Retry-After。
服务不可用时记录未验证，继续无关工作；本轮只查公开 API/审计文档，未检索、安装或执行候选包。

依据：[ClawHub API][claw-api]、[安全审计][claw-security]（rolling）；维护人为仓库维护者与变更作者，相关任务及版本/行为变化时复核。

[openspec]: https://github.com/Fission-AI/OpenSpec/blob/main/docs/commands.md
[superpowers]: https://github.com/obra/superpowers/blob/v6.4.1/skills/brainstorming/SKILL.md
[superpowers-release]: https://github.com/obra/superpowers/releases/tag/v6.4.1
[claw-api]: https://docs.openclaw.ai/clawhub/api
[claw-security]: https://docs.openclaw.ai/clawhub/security-audits
