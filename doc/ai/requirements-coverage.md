# 原始维护要求覆盖表

依据：用户于 2026-09-23 提供的 AI 工程维护附件及“补全，不遗漏”的修正要求。
“已覆盖”仅表示有具体规则/步骤，不代表相关产品、安装、部署或模型评估已经运行通过。
Rust 是目标方向；新增资料不扩展被用户排除的技术栈。

## 十项原则

| ID | 原始原则 | 权威落点与可检查结果 |
| --- | --- | --- |
| P01 | 基于证据 | [根规则][root]、[工程流程][engineering]：源码/调用方/测试/锁文件/配置/官方正文，区分版本与未验证 |
| P02 | 遵守授权 | [根规则][root]、[工程流程][engineering]：持续完成授权工作，审批对应具体产物，无机械确认 |
| P03 | 保护现状 | [根规则][root]：Git 状态、暂存区、上级/就近规则与独立 worktree 边界 |
| P04 | 单一事实来源 | [维护流程][maintenance]：规则、设计、计划、兼容层的权威位置和真实差异 |
| P05 | 精简上下文 | [根规则][root]、[维护流程][maintenance]：常驻约束与按需资源；不以精简删除适用要求 |
| P06 | 能力发现先行 | [根规则][root]、[终端 Skill][terminal]：真实工具/模式/权限/模型，不编造接口 |
| P07 | 验证与风险匹配 | [工程流程][engineering]、[验证记录][validation]：行为、文档、发现和运行各有证据 |
| P08 | 数据与指令分离 | [根规则][root]、[安全与交付][security]：第三方数据不扩大权限，凭据不外泄 |
| P09 | 保留可追溯性 | [来源索引][sources]、[维护流程][maintenance]：日期/版本/历史/审计/回滚不因去重丢失 |
| P10 | 先用已有工具 | [工具清单][tools]：已安装、候选、回退与安装边界分别标注 |

## 全部板块

| ID | 附件板块或具体要求 | 落点与适用边界 |
| --- | --- | --- |
| C01 | 目标：理解、开发、修复、测试、文档、部署、路线图与计划 | [工程流程][engineering]、[Rust Skill][rust]、[Plan][plan]；Rust 实现已建立 |
| C02 | 调研和实时更新：范围、索引、版本、冲突、方案、事实变化、收尾 | [工程流程][engineering]；来源含 claim/scope、URL/version、日期/method、installed/status、周期/触发、impact/owner |
| C03 | 推荐结构与文件选择决策 | [维护流程][maintenance]；最小入口、模块规则、Skill/Agent、兼容/工作流、设计/MCP、强制约束各有位置 |
| C04 | 跨工具兼容 | [兼容参考][clients] 覆盖附件 13 个客户端；只为实际需要创建配置，不能视为运行验收 |
| C05 | AGENTS.md 内容与加载边界 | [根规则][root]、[维护流程][maintenance]：目标、真实命令、兼容/生成/迁移、平台、导航、完成标准 |
| C06 | CLAUDE.md 条件兼容 | [兼容参考][clients]：版本/会话、优先入口、@ 导入、四跳、paths、加载证明及删除条件 |
| C07 | Skill 标准字段、描述、渐进加载与资源 | [维护流程][maintenance]：必需/可选字段、长度、命名、路由边界、直接资源引用 |
| C08 | Skill 脚本、调用策略与供应链 | [维护流程][maintenance]、[终端 Skill][terminal]：帮助、非交互、参数、退出/超时/副作用、完整 bundle 审计 |
| C09 | Skill 触发与质量评估 | [验证记录][validation]：正/负/近似/隐式样本，实际调用轨迹，对照、隔离、留出、未测说明 |
| C10 | 自定义 Agent | [维护流程][maintenance]：创建条件、schema、输入输出、权限、继承、分工隔离、整合与模型选择 |
| C11 | 任务分流与新增功能 | [工程流程][engineering]：局部改动、修复、新功能、安全/迁移、需求不清分别规定证据 |
| C12 | OpenSpec / Superpowers | [维护流程][maintenance]：未采用但保留采用步骤、profile/schema、门禁、同步/归档及验证边界 |
| C13 | MCP 版本、认证和生命周期 | [安全与交付][security]：协议/SDK 区分、HTTP/STDIO、输入/路径/出站、状态句柄、取消与退出 |
| C14 | MCP 最小权限、审计和注入 | [安全与交付][security]：Origin、监听/认证、stdout/stderr、限速/输出上限、annotations 非授权 |
| C15 | 本地调试、临时目录、秘密 | [安全与交付][security]：任务路径、忽略/访问权限、无真值模板、受控凭据、日志与清理 |
| C16 | 部署、CI 和供应链 | [安全与交付][security]、[Plan][plan]：短期凭据、最小权限、完整 SHA、不可信 PR 隔离、制品/迁移/健康/回滚 |
| C17 | 测试、lint、质量门禁 | [工程流程][engineering]、[Rust Skill][rust]：关键分支/失败、回归、mock 边界、实际命令/数量/结果、既有失败 |
| C18 | Markdown 与图表 | [工程流程][engineering]、[验证记录][validation]：明确 lint、链接/围栏/模板，图表语法和可读性 |
| C19 | 超时和重试 | [PowerShell 细则][powershell]：等待窗口/进程/总预算、有限重试、根因、退避/抖动、Retry-After、幂等与非交互 |
| C20 | 现代 CLI 场景和回退 | [工具清单][tools]：搜索、阅读、JSON/YAML、修改/diff、HTTP/日志/进程、基准/统计/压缩与 Rust 工具 |
| C21 | 工具安装、输出与退出语义 | [工具清单][tools]：来源/版本/架构/校验、binstall 编译回退、mise/aqua 差异、fzf、rg 限制、jq -e |
| C22 | PowerShell 解释器、引用、参数和管道 | [PowerShell 细则][powershell]：7+、NoProfile、别名、单引号/here-string、LiteralPath、& 语句块、原生参数/Legacy/--% |
| C23 | PowerShell 编码、错误、进程和基础设施 | [PowerShell 细则][powershell]：UTF-8/BOM/换行、5.1 区分、ErrorAction/LASTEXITCODE、隐藏窗口、PID 与路径清理、启动失败 |
| C24 | 文档、路线图、计划和交接 | [工程流程][engineering]、[Plan][plan]：按需索引、职责区分、同步引用/图示、未实现状态与交接证据 |
| C25 | 自我改进 | [维护流程][maintenance]：复现/分类、确定性检查、触发/范围/失效、持久记忆授权、升级后对照评估 |
| C26 | ClawHub 候选调研 | [维护流程][maintenance]：只读 API、少量筛选、owner/版本/许可/扫描/权限/网络、限速和缺失字段；本轮未安装候选 |
| C27 | 完成前检查、首次初始化与持续维护 | [工程流程][engineering]、[维护 Skill][maint-skill]、本覆盖表；不适用写原因，未执行不能标通过 |
| C28 | 来源和版本记录 | [来源索引][sources] 与各参考文末：实际核验范围、rolling/固定版本、本机状态、影响、维护人和复核触发 |

## 不应误判为遗漏或已完成的事项

- 客户端清单的完整参考不等于安装全套客户端；自定义 Agent/MCP/hooks/可选流程没有实施需求，不创建占位配置。
- 原附件链接的独立“高性能命令行工具清单”内容未提供；本地已建立可核验清单，不能声称读过或逐项覆盖未知外部内容。
- 本仓库已有 Cargo 工程；构建、业务测试与 CI 结果以 [验证记录][validation] 为准，不将其外推为生产运行验收。
- 新 Skill 的格式检查与已有发现记录见 [验证记录][validation]；按 2026-09-23 用户要求，规则注入、Skill 发现与真实效果不再作为本轮验收门禁。

[root]: ../../AGENTS.md
[engineering]: engineering-workflow.md
[security]: security-and-delivery.md
[sources]: source-index.md
[validation]: validation.md
[plan]: ../../Plan.md
[rust]: ../../.agents/skills/rust-cli-development/SKILL.md
[maint-skill]: ../../.agents/skills/ai-guidance-maintenance/SKILL.md
[terminal]: ../../.agents/skills/terminal-tooling/SKILL.md
[tools]: ../../.agents/skills/terminal-tooling/references/modern-cli-tools.md
[powershell]: ../../.agents/skills/terminal-tooling/references/powershell.md
[clients]: ../../.agents/skills/ai-guidance-maintenance/references/client-compatibility.md
[maintenance]: ../../.agents/skills/ai-guidance-maintenance/references/maintenance-workflows.md
