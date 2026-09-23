# 事实与来源索引

## 维护约定

核验日期 `verified_at`：2026-09-23。维护责任 `owner`：本仓库维护者与当前变更作者。
本页每条记录继承该日期和责任；更新部分记录时单独注明日期。
`review_cadence`：相关任务开始前复核；`update_trigger`：客户端、工具链、协议升级，弃用、加载失败或行为改变。
滚动文档标记 rolling，只证明文档内容已查阅，不证明本机版本的所有功能已经运行验收。

## 本地基线

- `source_version`：`656c7e3d44efee978334e0364eb7de0fe3c8778c`；`method`：Git、文件清单及源码静态阅读。
- 指引初始化阶段的 `git status --short` 无输出；当时无上级或仓库级 AGENTS 规则。
  2026-09-23 更新：Rust 2.0.0 工程已建立（根 `Cargo.toml`/`Cargo.lock`、`src/`、`tests/`、`.github/workflows/`），门禁全绿。
- `README.md` 定义项目为 xresconv-conf 转换列表 CLI，xresloader 是导出后端。
  `HISTORY.md` 的 1.4.5 记录多输出矩阵，1.4.3 记录后端出错仍返回成功的历史缺陷。
- 历史 Python 的 `load_xml_file` 递归处理 include；`load_global_options` 和 `load_list_item_nodes` 处理合并与筛选；
  `worker_func` 启动 Java `--stdin` 批处理并汇总退出状态。本轮已重放 6 组差分预览及真实后端样本。
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
| R1 | Cargo test 默认执行单元、集成和适用的文档测试；--no-run 只编译 | [cargo test][cargo-test] / rolling | 文档已核验；项目门禁已实际执行（2026-09-23，cargo 1.98.0） | Rust 验证流程 |
| R2 | rustfmt 支持检查模式；Clippy 支持 Cargo 工程与 lint 级别设置 | [rustfmt][rustfmt]、[Clippy][clippy] / rolling | 官方正文和本机帮助已核验；门禁 fmt/clippy -D warnings 已执行通过 | Cargo 门禁 |
| X1 | xresconv-conf 提供转换列表规范及 include 示例 | [xresconv-conf][conf] / `dab714ae4ca6a0dc8af5410087ce90e2dfbe687c` | GitHub API 固定 main 快照；fixture 合同测试 | 迁移合同 |
| R3 | clap 4.6.7、roxmltree 0.21.1、regex 1.13.1、windows-sys 0.61.2、tempfile 3.27.0；新增 ctrlc 3.5.2、encoding_rs 0.8.41、libc 0.2.189 | [crates.io API][crates-io] / 2026-09-23 max_stable_version | 最新稳定版已查询；Cargo 更新锁文件，encoding_rs 的 MSRV 1.88 已实测 | Cargo.toml/Cargo.lock |
| X2 | xresloader v2.23.7 release 资产含 `xresloader-2.23.7.jar`，SHA256 见验证记录；样本 Excel 由 Git LFS 管理 | GitHub API / v2.23.7，仓库 `.gitattributes` | 本机包与官方包各跑 5 个真实测试；run 35853777700 的 LFS 指针失败已定位 | 本地 E2E 基线；CI 改为解析最新 Release |
| CI1 | checkout v7、upload-artifact v7、download-artifact v8、setup-python v7、setup-java v6、stale v11 | [checkout][checkout-action]、[Python setup][setup-python]、[Java setup][setup-java] | 用户要求随大版本接收更新；源码内 uses 不再钉 SHA，版本策略需在每次 CI 验收 | .github/workflows |
| CI2 | `ubuntu-latest`/`macos-latest`/`windows-latest` 为滚动标签；ARM 原生 runner 仍需架构专用标签 | [官方 runner 表][runners]、[镜像标签][runner-images] / rolling | GitHub 文档核验；第二次 CI 的 macOS Intel 交叉包与 Rosetta 冒烟通过 | build.yml 矩阵 |
| X3 | 本地 2.23.7 后端 stdin 不支持反斜杠转义 | [Main.java][backend-parser] / `7263367d99f04af8fca8b1fd80cac29b05f2f6b0` | 本地源码逐行核对 + 真实 JAR 24 产物对照；CI 最新版仍须每次验证 | 参数编码与 CI 最新版行为风险 |
| CI4 | setup-python 的 `3.x` 取最新稳定 Python 3；Python 无单独 LTS 系列；Adoptium API 有 `most_recent_lts` | [Python setup][setup-python]、[Python 版本状态][python-versions]、[Adoptium API][adoptium-api] | 官方文档与实时 API 核验；CI 动态解析 Temurin LTS | build.yml 环境选择 |
| CI5 | GitHub Release API 资产提供 `digest`；LFS 支持跳过检出时下载并按路径物化 | [Release API][release-api]、[Git LFS pull][lfs-pull]、[Git LFS 配置][lfs-config] | 第二次 CI 的 Windows 检出因无关 benchmark LFS 大文件失败；本机 Windows 仅下载所需 Excel 后 OOXML 文件头通过 | 最新 JAR 与样本一致性 |
| CI3 | Linux、macOS、Windows 的 x64/arm64 原生 runner 标签 | [官方 runner 表][runners] / 2026-09-23 | 第二次 CI 普通测试和 8 个核心包构建/冒烟通过；三平台真实后端测试待修复后复跑 | build.yml 核心测试矩阵 |
| T4 | cargo-llvm-cov 0.9.1 / actionlint 1.7.12 | [覆盖率工具][llvm-cov]、[actionlint][actionlint] | crates.io / GitHub API 核验；本机运行，actionlint ZIP 校验官方 asset digest | 覆盖率与 CI 静态检查 |

Cargo.toml 使用显式 `^` 兼容更新范围；Cargo.lock 固定已验证的实际版本。
两个官方 XML fixture 仅规范换行和行末空白，配置内容保持上述固定提交版本。
JAR 校验与运行记录见 [验证记录](validation.md)：用户本机包与官方 Release 包的 SHA256 不同，分别实测，不混称同一文件。

## 安装与兼容范围

`installed_version` 为本机快照，不是项目最低版本或团队统一版本；探测退出码均为 0。

| 对象 | 本机版本 / 检测方式 | 当前结论 |
| --- | --- | --- |
| Codex CLI | `0.155.0-alpha.16`，`codex --version` | 本会话客户端；新文件的独立启动加载尚未验收 |
| Kilo Code CLI | `7.4.21`，`kilo --version` | 两处 cwd 的 Skill 发现已验证；团队使用清单待确认 |
| PowerShell | `7.6.6`，`$PSVersionTable.PSVersion` | 本次实际 shell |
| Rust / Cargo | `1.98.0`，`rustc --version` / `cargo --version` | 本地常规工具链；项目 MSRV 1.88.0 已额外安装并实测 |
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
[crates-io]: https://crates.io/api/v1/crates
[backend-parser]: https://github.com/xresloader/xresloader/blob/7263367d99f04af8fca8b1fd80cac29b05f2f6b0/src/org/xresloader/core/Main.java
[runners]: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
[llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov
[actionlint]: https://github.com/rhysd/actionlint/releases/tag/v1.7.12
[checkout-action]: https://github.com/actions/checkout/blob/main/README.md
[setup-python]: https://github.com/actions/setup-python/blob/main/docs/advanced-usage.md
[setup-java]: https://github.com/actions/setup-java/blob/main/README.md
[runner-images]: https://github.com/actions/runner-images/blob/main/README.md
[python-versions]: https://devguide.python.org/versions/
[adoptium-api]: https://github.com/adoptium/api.adoptium.net/blob/main/docs/cookbook.adoc
[release-api]: https://docs.github.com/en/rest/releases/releases
[lfs-pull]: https://github.com/git-lfs/git-lfs/blob/main/docs/man/git-lfs-pull.adoc
[lfs-config]: https://github.com/git-lfs/git-lfs/blob/main/docs/man/git-lfs-config.adoc
[ps-ref]: ../../.agents/skills/terminal-tooling/references/powershell.md
[tools-ref]: ../../.agents/skills/terminal-tooling/references/modern-cli-tools.md
[clients-ref]: ../../.agents/skills/ai-guidance-maintenance/references/client-compatibility.md
[maintenance-ref]: ../../.agents/skills/ai-guidance-maintenance/references/maintenance-workflows.md
