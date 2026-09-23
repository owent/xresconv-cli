# 客户端兼容与接入

2026-09-23 核对官方文档。格式可移植、文件发现、正文加载、自动触发和实际权限是不同层次，须分别验证。
本仓库当前使用证据仅涵盖 Codex 会话及已安装 Kilo CLI；完整团队清单待确认，不为候选客户端创建目录。
下表为接入时的参考：除 [来源索引](../../../../doc/ai/source-index.md) 已记录版本外，其他客户端版本未探测、运行均未验证。

## 规则与 Skills

| 客户端 | 入口与边界 | 官方依据 |
| --- | --- | --- |
| Codex | 根 AGENTS.md；每目录优先 AGENTS.override.md；启动按根到 cwd 构建，默认总上限 32 KiB。Skills 从 cwd 向仓库根扫描 .agents/skills，同名不合并 | [规则][codex-rules]、[Skills][codex-skills] |
| Claude Code | CLAUDE.md、.claude/rules、.claude/skills；原生 AGENTS.md 有版本/会话条件，详见下节 | [Memory][claude-memory]、[Skills][claude-skills] |
| VS Code Copilot | 根 AGENTS.md、.github/copilot-instructions.md；Skills 可位于 .github/skills、.claude/skills、.agents/skills；嵌套规则受实验开关控制 | [Instructions][vscode-rules]、[Skills][vscode-skills] |
| OpenCode | AGENTS.md；.opencode/skills、.claude/skills、.agents/skills；项目 Skills 向上到 Git worktree 边界，规则回退与权限按其文档核验 | [Rules][opencode-rules]、[Skills][opencode-skills] |
| Kilo Code | AGENTS.md（AGENT.md 备选）、.kilo/skills 与 .agents/skills；区分 CLI/扩展和外部 Skills 开关，审计可信位置的嵌入 shell | [规则][kilo-rules]、[Skills][kilo-skills] |
| Pi | AGENTS.md/AGENTS.override.md；.pi/skills、用户 .pi/agent/skills、.agents/skills；项目信任影响加载，context 发现不等于沙箱 | [Configuration][pi-config]、[Skills][pi-skills] |
| Oh My Pi | 共享 AGENTS.md 和多来源 Skills；原生 .omp/AGENTS.md、.omp/RULES.md 从最近非空 .omp 目录读取，缺文件不继续向上补齐 | [Context][omp-context]、[Skills][omp-skills] |
| Command Code | AGENTS.md；.commandcode/skills 与 .agents/skills；同名来源顺序及命令冲突需单独检查，不能套用其他客户端 | [Memory][command-memory]、[Skills][command-skills] |
| Zoo Code | 根 AGENTS.md/AGENT.md、.roo/rules 与 `.roo/rules-<mode>`；检查 roo-cline.useAgentRules，不自行改名 .zoo 或推断共享 Skills 支持 | [Custom instructions][zoo] |
| OpenClaw | workspace AGENTS.md；workspace skills、.agents/skills 等多级来源；来源优先级、可见性 allowlist、library/workshop 和执行 workspace 分别检查 | [Skills][openclaw] |
| Hermes Agent | 项目 context 按类型 first-match，支持 .hermes.md/HERMES.md、AGENTS.override.md、AGENTS.md 等；项目根 .hermes/skills 与 .agents/skills；外部个人目录单独配置 | [Context][hermes-context]、[Skills][hermes-skills] |
| Devin Desktop | AGENTS.md、.devin/rules；.devin/skills 优先，兼容 .agents/skills 和旧 .windsurf/skills；区分 Devin Local 与 legacy Cascade | [Memories][devin-memory]、[Skills][devin-skills] |
| Antigravity | workspace .agents/rules、.agents/skills；2.0、CLI、IDE 的全局路径有差异；不能由此推断独立 Gemini CLI 的规则发现 | [Rules][antigravity-rules]、[Skills][antigravity-skills] |

客户端 Markdown/YAML/TOML、glob、`@import` 和同名优先级不能跨产品套用。
Pi 的 SYSTEM.md 覆盖和 APPEND_SYSTEM.md 追加、OMP 常驻 RULES 与按需 skill://、OpenClaw 状态目录和 sandbox 副本，使用前查实际版本。
Command Code Skills 的项目专属、项目共享、用户专属、用户共享顺序，不应外推到其 Agent 定义。

VS Code 的 `chat.useNestedAgentsMdFiles` 仍是实验开关。
Agent Host 不加载已弃用的 prompt files；Local agent 暂仍支持，重复任务迁往 Skills，不能把 `.github/prompts/` 当统一入口。[Prompt files][vscode-prompts]

自定义 Agent 的独立格式亦需按客户端核验：Codex 用 `.codex/agents/*.toml`；Claude 用 `.claude/agents/`；
VS Code 用 `.github/agents/*.agent.md`，其 `infer` 字段已弃用；OpenCode 用 `.opencode/agents/`；Command Code 用 `.commandcode/agents/`。
来源：[Codex][codex-agents]、[Claude][claude-agents]、[VS Code][vscode-agents]、[OpenCode][opencode-agents]、[Command Code][command-agents]。
其他产品的 Agent 专属路径不从产品名称或 Skills 目录推断；创建条件与权限检查见 [维护流程](maintenance-workflows.md#自定义-agent-与分工)。

## Claude 兼容决策

- 原生 AGENTS.md 要求 v2.1.277+，并受 feature flags、内置 agents-md 插件、provider、telemetry 和项目指令设置影响。
  首次安装/升级后的会话也可能尚未生效；不能笼统声称永不支持或所有会话默认支持。
- 默认可能优先采用 cwd/祖先的 CLAUDE.md、.claude/CLAUDE.md 或 CLAUDE.local.md。
  先查实际加载文件和 Project instructions；需要回退才创建根 CLAUDE.md。
- 兼容内容最小化为 `@AGENTS.md`，只追加真实 Claude 专属差异，不复制共享全文。
  相对导入基于声明文件；最多四跳，导入正文仍消耗启动上下文。普通 Markdown 链接不是导入。
- 条件规则使用其 `.claude/rules/` 的 paths；Skills 采用该客户端实际支持的目录或兼容机制。
  共享资源必须可达，生成副本需标记权威源并验证一致性。
- `/context` 等可用界面与调用记录用于核对加载；仅当所有目标会话原生可读且无专属内容时，才移除仍有用途的桥接。

本项目当前没有 CLAUDE.md；上述是接入流程，不是创建兼容文件或修改全局设置的指令。

## 验收与版本记录

每个实际采用的客户端记录版本、会话类型、cwd、开关、冲突与来源，分别检查根目录和相关子目录。
检查规则注入、Skill 元数据发现、正文读取、显式/自动调用及权限拒绝；保存真实轨迹，不以文件存在或口头复述代替。
共享资源、同名覆盖、禁用开关和配置变更回滚都要覆盖；用无副作用样例，未执行步骤明示。
步骤与结果见 [验证记录](../../../../doc/ai/validation.md)。

来源版本为 rolling，Oh My Pi 为 main；这里只核验文档，已安装版本支持须单独证明。
维护人为仓库维护者与变更作者；接入、升级、弃用、发现失败或行为变化时复核。

[codex-rules]: https://learn.chatgpt.com/docs/agent-configuration/agents-md
[codex-skills]: https://learn.chatgpt.com/docs/build-skills
[claude-memory]: https://code.claude.com/docs/en/memory
[claude-skills]: https://code.claude.com/docs/en/skills
[vscode-rules]: https://code.visualstudio.com/docs/agent-customization/custom-instructions
[vscode-skills]: https://code.visualstudio.com/docs/agent-customization/agent-skills
[vscode-prompts]: https://code.visualstudio.com/docs/agent-customization/prompt-files
[opencode-rules]: https://opencode.ai/docs/rules
[opencode-skills]: https://opencode.ai/docs/skills
[kilo-rules]: https://kilo.ai/docs/customize/agents-md
[kilo-skills]: https://kilo.ai/docs/customize/skills
[pi-config]: https://pi.dev/docs/latest/configuration
[pi-skills]: https://pi.dev/docs/latest/skills
[omp-context]: https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md
[omp-skills]: https://github.com/can1357/oh-my-pi/blob/main/docs/skills.md
[command-memory]: https://commandcode.ai/docs/memory
[command-skills]: https://commandcode.ai/docs/skills
[zoo]: https://docs.zoocode.dev/features/custom-instructions
[openclaw]: https://docs.openclaw.ai/tools/skills
[hermes-context]: https://hermes-agent.nousresearch.com/docs/user-guide/features/context-files
[hermes-skills]: https://hermes-agent.nousresearch.com/docs/user-guide/features/skills
[devin-memory]: https://docs.devin.ai/desktop/cascade/memories
[devin-skills]: https://docs.devin.ai/desktop/cascade/skills
[antigravity-rules]: https://antigravity.google/docs/rules-workflows
[antigravity-skills]: https://antigravity.google/docs/skills
[codex-agents]: https://learn.chatgpt.com/docs/agent-configuration/subagents
[claude-agents]: https://code.claude.com/docs/en/sub-agents
[vscode-agents]: https://code.visualstudio.com/docs/agent-customization/custom-agents
[opencode-agents]: https://opencode.ai/docs/agents/
[command-agents]: https://commandcode.ai/docs/agents
