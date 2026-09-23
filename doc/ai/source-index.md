# 事实与来源索引

## 维护约定

核验日期 `verified_at`：2026-09-23。维护责任 `owner`：本仓库维护者与当前变更作者。
本页每条记录继承该日期和责任；更新部分记录时单独注明日期。
`review_cadence`：相关任务开始前复核；`update_trigger`：客户端、工具链、协议升级，弃用、加载失败或行为改变。
滚动文档标记 rolling，只证明文档内容已查阅，不证明本机版本的所有功能已经运行验收。

## 本地基线

- `source_version`：`656c7e3d44efee978334e0364eb7de0fe3c8778c`；`method`：Git、文件清单及源码静态阅读。
- 起始 `git status --short` 无输出；无上级或仓库级 AGENTS 规则，无已有 Skills/Plan，也无 Cargo 工程、测试目录或 CI。
- `README.md` 定义项目为 xresconv-conf 转换列表 CLI，xresloader 是导出后端。
  `HISTORY.md` 的 1.4.5 记录多输出矩阵，1.4.3 记录后端出错仍返回成功的历史缺陷。
- 现有实现的 `load_xml_file` 递归处理 include；`load_global_options` 和 `load_list_item_nodes` 处理合并与筛选；
  `worker_func` 启动 Java `--stdin` 批处理并汇总退出状态。上述是源码证据，尚未重放兼容样本。
- include 相对其所在文件解析；工作目录以主列表目录为起点再应用 `work_dir`。
  Java 启动参数与逐行 stdin 命令分开构造。Rust 实现必须先确定相应合同，而非照搬实现细节。
- 可用下列只读定位命令复查原始证据；本页不复制旧实现内容。

```text
git show --stat 656c7e3d44efee978334e0364eb7de0fe3c8778c
git grep -n -e load_xml_file -e load_global_options -e load_list_item_nodes -e worker_func 656c7e3
```

- `.kilo/worktrees/torch-nasturtium` 是 Git 列出的独立 detached worktree；不纳入本轮修改或验证范围。
- 文本规则：`.gitattributes` 指定 Markdown 为 text/CRLF；许可证为 MIT。
- `impact`：根规则、Rust Skill 和 [Plan.md](../../Plan.md)。源码或目录变化时更新相应状态。

## 外部结论

| ID | claim / scope | source_url / source_version | method / status | impact |
| --- | --- | --- | --- | --- |
| A1 | AGENTS.md 为共享 Markdown 工程规则入口 | [AGENTS.md][agents] / rolling | 官方正文已核验；加载以客户端为准 | 根规则 |
| A2 | Skill 必需 name/description，名称匹配目录；正文与资源可按需读取 | [Skills 规范][skills] / rolling | 官方正文已核验；本地格式检查另记 | 三个 Skills |
| C1 | Codex 项目规则按根到 cwd 构建，每层优先 AGENTS.override.md，再 AGENTS.md | [Codex 规则][codex-rules] / rolling | 文档已核验；新规则启动加载未验证 | 共享根入口 |
| C2 | Codex 从 cwd 向仓库根发现 .agents/skills；同名不会合并 | [Codex Skills][codex-skills] / rolling | 文档已核验；本轮 catalog 已列出最初两个 Skill，完整路由对照未验证 | Skills 权威位置 |
| K1 | Kilo 支持根 AGENTS.md，CLI 支持 .agents/skills；禁用外部 Skills 会影响发现 | [Kilo 规则][kilo-rules]、[Skills][kilo-skills] / rolling | 文档已核验；CLI 根/子目录 Skill 发现通过，规则注入未验证 | 不复制专属规则 |
| R1 | Cargo test 默认执行单元、集成和适用的文档测试；--no-run 只编译 | [cargo test][cargo-test] / rolling | 文档已核验；无工程，项目运行未验证 | Rust 验证流程 |
| R2 | rustfmt 支持检查模式；Clippy 支持 Cargo 工程与 lint 级别设置 | [rustfmt][rustfmt]、[Clippy][clippy] / rolling | 官方正文和本机帮助已核验；项目门禁未执行 | 未来 Cargo 命令 |
| X1 | xresconv-conf 提供转换列表规范及 include 示例 | [xresconv-conf][conf] / main，未固定提交 | 仅核验仓库概览；完整样本合同留待 P1 | 迁移合同 |

## 安装与兼容范围

`installed_version` 为本机快照，不是项目最低版本或团队统一版本；探测退出码均为 0。

| 对象 | 本机版本 / 检测方式 | 当前结论 |
| --- | --- | --- |
| Codex CLI | `0.155.0-alpha.16`，`codex --version` | 本会话客户端；新文件的独立启动加载尚未验收 |
| Kilo Code CLI | `7.4.21`，`kilo --version` | 两处 cwd 的 Skill 发现已验证；团队使用清单待确认 |
| PowerShell | `7.6.6`，`$PSVersionTable.PSVersion` | 本次实际 shell |
| Rust / Cargo | `1.98.0`，`rustc --version` / `cargo --version` | 已安装，不代表项目已选定工具链或最低版本 |
| Rustup | `stable-x86_64-pc-windows-msvc`，`rustup show active-toolchain` | 本机默认工具链，无项目固定配置 |
| Node.js | `24.21.0`，`node --version` | 仅用于复用已有文档校验工具，不是 CLI 产品依赖 |
| markdownlint-cli2 | `0.23.2`，已有安装的 package.json | 不在 PATH；从相邻工作区已安装目录调用 CLI，未安装依赖 |

Codex 与 Kilo 共用根规则和 `.agents/skills/`，目前不需要复制文件或建立专属配置。
本机版本不能代表 Kilo 编辑器扩展；其他客户端未验收，不推断它们能加载当前文件。
接入新客户端时先记录版本、会话、开关和根/子目录加载结果，再决定兼容层。
本次 `kilo --pure debug skill` 禁用了外部插件，只验证原生发现；不能外推到有插件的完整团队会话。

## 初始化适用性审查

逐项审查以 [要求覆盖表](requirements-coverage.md) 为权威入口，包含十项原则和 28 项要求。
“未采用”不代替条件流程；相关规则已放入按需材料，实际安装、配置和运行仍由任务需要决定。
上一轮仅有简表，遗漏了多项执行细则；本轮补齐，不把上一轮格式通过当作内容完整性通过。

## 本轮补全的来源记录

以下各记录继承本页 verified_at/owner/review_cadence/update_trigger；链接文件中列出实际 source_url 和具体 scope。
这些参考文件与本索引共同保存来源元数据，避免再复制完整客户端/工具表。

| ID | claim / scope 与 source_url 所在位置 | source_version / method / installed_version / status | impact |
| --- | --- | --- | --- |
| T1 | [PowerShell 细则][ps-ref]：参数、Legacy、编码与退出状态 | PowerShell 7.5 官方正文 + 本机 7.6.6；运行样例另记；5.1 未执行 | 根规则、终端 Skill |
| T2 | [现代 CLI][tools-ref]：用途、回退、来源、安装边界 | 官方 rolling/main/master 概览与 Get-Command；已安装版本见清单；候选安装/运行未验证 | 终端 Skill |
| T3 | jq/yq/rg/zstd/curl 身份与版本 | 本机 --version；版本分别为 1.8.2 / mikefarah 4.53.6 / 15.2.0 / 1.5.7 / 8.21.0 | 输出和退出码规则 |
| C3 | [13 客户端参考][clients-ref]：入口、版本条件与发现边界 | 官方 rolling，OMP 为 main；除本页本机记录外均未探测版本，运行未验证 | 维护 Skill、未来兼容层 |
| G1 | [维护流程][maintenance-ref]：OpenSpec/Superpowers 的条件使用 | OpenSpec main，Superpowers v6.4.1；已查官方正文，未安装/执行 | 设计和归档流程 |
| G2 | [维护流程][maintenance-ref]：ClawHub API 和审计边界 | 官方 rolling；文档核验，未检索候选、安装或运行 | 自我改进、第三方 Skill 审计 |
| S1 | [安全与交付](security-and-delivery.md)：MCP 传输/授权/状态/工具限制 | 固定 2026-07-28 规范及安全文档；本仓库无 SDK/服务运行验收 | 条件接入流程 |
| S2 | [安全与交付](security-and-delivery.md)：秘密、CI、幂等与发布 | OWASP/GitHub/AWS 官方 rolling；文档核验，无生产验收 | 本地调试、P4 发布准备 |
| E1 | [工程流程](engineering-workflow.md)：任务、设计、验证、交接与完成标准 | 用户附件 + 本仓库 Rust 方向；项目约定，不宣称来自某客户端硬性规则 | 根规则与 Rust Skill |

用户原附件的完整文本用于覆盖审查，不把其中滚动版本声明自动当作本轮官方验证结果。
下一次局部修改只复查相关来源；不要因为本表较完整就全量联网、安装或升级。

[agents]: https://agents.md/
[skills]: https://agentskills.io/specification
[codex-rules]: https://learn.chatgpt.com/docs/agent-configuration/agents-md
[codex-skills]: https://learn.chatgpt.com/docs/build-skills
[kilo-rules]: https://kilo.ai/docs/customize/agents-md
[kilo-skills]: https://kilo.ai/docs/customize/skills
[cargo-test]: https://doc.rust-lang.org/cargo/commands/cargo-test.html
[rustfmt]: https://github.com/rust-lang/rustfmt
[clippy]: https://doc.rust-lang.org/stable/clippy/usage.html
[conf]: https://github.com/xresloader/xresconv-conf
[ps-ref]: ../../.agents/skills/terminal-tooling/references/powershell.md
[tools-ref]: ../../.agents/skills/terminal-tooling/references/modern-cli-tools.md
[clients-ref]: ../../.agents/skills/ai-guidance-maintenance/references/client-compatibility.md
[maintenance-ref]: ../../.agents/skills/ai-guidance-maintenance/references/maintenance-workflows.md
