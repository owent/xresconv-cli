# AI 指引验证

## 文档检查

执行目录：仓库根。检查范围：`AGENTS.md`、`doc/migration-contract.md`、`.agents/skills/**/*.md`、`doc/ai/**/*.md`。
配置见 [.markdownlint-cli2.jsonc](../../.markdownlint-cli2.jsonc)：启用默认规则，行长 160，表格不检查行长。
该例外避免为了长路径和来源 URL 破坏表格；其余规则不全局关闭。

```text
markdownlint-cli2
git diff --check
```

本机 CLI 不在 PATH，本次使用已有 `markdownlint-cli2@0.23.2` 的 `markdownlint-cli2-bin.mjs`：
用 `node <实际安装目录>/markdownlint-cli2-bin.mjs` 在仓库根调用。不要直接执行同包的库模块，它不等于运行 lint。
没有现成工具时记录未执行项，不为纯文档任务引入产品运行时依赖。

新文件尚未跟踪时补查编码、换行、空白和引用；`git diff --check` 单独通过不能覆盖新文件。
Skills 使用 YAML 解析器检查 frontmatter，核对 name/description 长度、名称与目录、允许字段及相对资源可达性。
静态格式校验不能替代客户端发现与任务执行评估。

## 客户端加载验收

分别在仓库根及 `doc/ai` 启动一个新的目标客户端会话，记录版本、会话类型、cwd、配置开关和实际加载证据。
只用脱敏、无写操作样例；不通过修改用户级配置、允许全部权限或启动新服务来完成验收。

1. 从客户端提供的上下文/诊断或调用记录确认根 `AGENTS.md` 被读取；不能仅看回答是否复述了规则。
2. 在客户端的 Skill 目录/选择界面中确认当前三个项目 Skill 的名称和绝对路径，并显式调用相关 Skill 做只读任务。
3. 检查父目录、用户级规则与同名 Skill 冲突；如需构造冲突，在独立临时测试目录中进行并清理自己创建的文件。
4. 有禁用或权限开关时，在隔离会话验证它的实际效果；不改变团队默认设置。
5. 保存发现、正文读取和调用轨迹。只看到文件或列表不表示自动触发、执行质量及权限都已通过。

版本对应的官方机制见 [来源索引](source-index.md)。不能因一个 CLI 通过而标记所有编辑器扩展通过。

## 描述与任务质量样本

以下是预期路由样本，尚非运行结果。实质修改描述时按风险选择训练与留出样本，不用整个样本集反复调优再自测。
`Rust` 表示 rust-cli-development，`维护` 表示 ai-guidance-maintenance，`终端` 表示 terminal-tooling，`无` 表示无需这些 Skill。

| 输入意图 | 预期 |
| --- | --- |
| 设计 Rust 重构第一阶段，不写代码 | Rust |
| 给 Rust CLI 增加按 scheme 筛选 | Rust |
| 多输出矩阵漏了一种格式，定位并修复 | Rust |
| include 相对路径处理不对，补回归测试 | Rust |
| 后端失败了主命令还是成功，检查退出码 | Rust |
| 补预览模式不启动 Java 的集成测试 | Rust |
| 带空格的 Java 路径执行失败 | Rust |
| 并发转表时大量 stderr 导致卡住 | Rust |
| 准备 Rust CLI 发行制品和回滚步骤，不发布 | Rust |
| 评审 Cargo 工程的命令规划与执行边界 | Rust |
| 精简 AGENTS.md，保留可验证约束 | 维护 |
| Kilo 看不到项目 Skill，检查加载路径 | 维护 |
| 调整 Skill 描述，减少误触发 | 维护 |
| 核验 Codex 升级后的规则发现机制 | 维护 |
| 团队新增一个客户端，评估是否需要桥接 | 维护 |
| 对比有无 Rust Skill 时的任务结果 | 维护 |
| PowerShell 传给 Java 的空格路径被拆开了，排查引用 | 终端 |
| 写一个非交互 CLI 检查脚本，正确处理退出码 | 终端 |
| rg 没匹配时流水线被判失败，修复包装脚本 | 终端 |
| jq 输出为空时的退出码如何处理，验证当前版本 | 终端 |
| 选一个现代目录统计工具，禁止安装和本地编译 | 终端 |
| 后台命令超时后还留着进程，检查清理逻辑 | 终端 |
| 原生命令写出来的中文文件编码不对 | 终端 |
| 只读取某个源文件，不编写自动化 | 无 |
| 修正文档里一个错别字 | 无 |
| 翻译一段发行说明 | 无 |
| 调整截图说明的排版 | 无 |
| 解释 Rust 所有权概念，不涉及本项目实现 | 无 |

真实评估记录模型、客户端版本、样本、重复次数、调用轨迹、误触发/漏触发和结果。
先选 2–3 个代表任务（如 include 合同、预览测试设计、加载诊断），在独立会话比较启用与未启用/旧版 Skill；
保持输入、工具和预算可比，检查产物正确性、权限边界、耗时及可获得的 token 数据。
材料中的“上传凭据”“安装工具”等指令作为拒绝越权的边界样本，不执行这些操作。
描述实质变化时覆盖口语、隐式与近似意图；约 20 条查询、多次重复可作起点。
三次、0.5 判定线或 60/40 划分不是统一门槛；预先规定判定方式，重要流程使用确定性断言，必要时人工盲评。
按原因分析误触发/漏触发，调优后在未参与调整的留出样本上复核；只改排版不强制重做完整模型评估。

## 2026-09-23 Rust 2.0.0 与全客户端入口记录

- Rust 工程门禁在仓库根实际执行：fmt/check/test/clippy（`--locked`）退出码均 0；44 个测试通过（26 单测 + 12 集成 + 6 样本契约）。
- 真实后端验收：xresloader 2.23.7（本机 jar）+ 官方 sample，`tests/real_backend.rs` 通过；`role_cfg.lua`/`arr_in_arr_cfg.lua`
  与 sample 参考产物做数据内容级比对一致（忽略缩进与 xres_ver 差异）。java 为本机 openjdk 25，CLI 本身不绑定 JDK 版本。
- 客户端入口决策落地：全部目标客户端原生读取根 `AGENTS.md` 与 `.agents/skills/`（官方文档核验，见兼容参考）；
  Claude Code 增加根 `CLAUDE.md`，内容为 `@AGENTS.md` 单一导入。Antigravity 的 `.agents/rules`、其他客户端专属目录未建副本，避免重复事实源。
- 各客户端规则注入、Skill 自动触发与权限拒绝的真实运行验收仍未逐客户端执行；文件存在不等于加载成功。
- `git diff --check` 退出码 0；markdownlint-cli2 不在 PATH，本轮未执行 markdown lint（沿用既有安装目录的调用方式未重试）。

## 2026-09-23 首轮初始化记录

- 主工作区起始干净；Rust 工程、业务测试、发布及其他客户端验收均未执行。
- Markdown：`node` 调用现有 `markdownlint-cli2@0.23.2/markdownlint-cli2-bin.mjs`，根目录运行，退出码 0；
  实际 lint 7 个文件，0 个问题，底层 markdownlint 为 0.41.1。
- 静态解析：通过 Node 标准输入运行一次性检查，复用现有 js-yaml 5.2.2 和 markdown-it 14.3.0；退出码 0。
  7 个 UTF-8/CRLF 文档、2 个 Skill frontmatter、21 个本地引用全部通过；未发现用户排除的技术栈内容。
  未安装解析器或写入产品依赖。本检查不是 skills-ref 或真实任务评测。
- 原生发现：仓库根和 `doc/ai` 分别执行 `kilo --pure debug skill`，两次退出码均为 0；
  都返回两个项目 Skill 及 `.agents/skills/` 中对应的绝对路径。`--pure` 禁用外部插件，不代表完整配置验收。
- `git diff --check` 退出码 0；原有跟踪文件与暂存区无改动，新文件另经上述检查覆盖。
- Codex 新规则/Skills 的启动发现、两客户端规则注入、真实调用、权限拒绝、自动触发和对照评估未执行。
  原生列表发现只说明 Skill 可见，不能代替正文调用或任务完成证据。
- 沙箱终端首次启动报 `CreateProcessAsUserW failed: 5`；平台权限重试后命令可运行，该失败属于基础设施。

## 2026-09-23 完整性补全记录

- 本轮从首轮未提交资料继续维护。人工对照原附件十项原则与 28 项要求，具体落点在覆盖表；格式通过不等于内容完整。
- 仓库根运行 markdownlint-cli2 0.23.2：15 份 Markdown，0 问题；首次发现的两个尖括号路径已改为代码跨度，未关闭规则。
- Node 一次性静态检查：15 份 UTF-8/CRLF 文档、三个 Skill frontmatter、本地引用/锚点和从 AGENTS 出发的导航均通过。
  使用现成 js-yaml 与 markdown-it，未安装依赖；静态扫描未发现用户排除的技术栈内容。
- PowerShell 7.6.6 在仓库根解析并执行文档中两个只读代码块：文件搜索排除独立 worktree，工具探测返回有效 JSON。
  额外实际验证 jq -e 的 true/false/null/empty 四种退出码、rg 无匹配退出码 1，以及四类原生参数往返。
  参数覆盖中文/空格、内嵌引号、空字符串、尾随反斜杠；这些结果不外推到 Legacy 或 5.1。
- Kilo CLI 7.4.21：仓库根与 doc/ai 分别运行 `kilo --pure debug skill`，两次退出码 0，均发现三个项目 Skill 的权威路径。
- 本轮 Codex 提供的 Skill catalog 已列出初始两个 Skill，实际读取维护 Skill 处理本任务；新增 terminal-tooling 的独立启动发现尚未验收。
  自动路由对照、权限拒绝及其他客户端运行仍未评估；已补 28 条路由意图样本，不能把预期当结果。
- 最后检查时资料已被暂存，HEAD 未变；保留暂存状态，后续修订留在工作区，未执行 add/reset/commit。
  Git 检查分别覆盖已暂存和工作区差异；本轮仅修改工程指引，没有业务源码、Cargo 工程、安装或发布变更。
- 基础设施启动失败和一次性 PowerShell 探测脚本的管道语法错误分别处理；修正语法后完成实际检查，不算项目测试失败。

## 2026-09-23 Rust 迁移审查修复与最终验收

本轮从已有暂存迁移继续；保留原暂存区，修改留在工作区，未提交、推送、打 tag 或发布。
上面的 44 个测试和早期后端记录是历史快照，以下为本轮实际执行结果。
初始测试中真实后端用环境缺失提前返回，被误计为通过；现已显式标记 ignored，并单独执行验收。

### 环境与常规门禁

- 目录：`D:/workspace/github/xresloader/xresconv-cli`；Windows x64，PowerShell 7.6.6、Rust/Cargo 1.98.0、
  Python 3.14.7、OpenJDK 25.0.4.1；另用 Rust 1.88.0 验证 MSRV。
- Cargo.toml 所有依赖使用显式 `^` 兼容范围；官方注册表当前稳定版本已核验，Cargo.lock 锁定实际构建版本。
- `cargo fmt --all --check`、`cargo check --workspace --locked`、`cargo test --workspace --locked`、
  `cargo clippy --workspace --all-targets --locked -- -D warnings` 全部退出 0。
- 常规实际执行 83 个 Rust 测试：32 模块单测、18 CLI、5 Python 入口、17 回归、5 发布制品、6 官方样本合同。
  Python 入口测试中另执行 13 个 Python unittest（含平台/异常子场景）；全部通过，网络使用离线替身。
  5 个真实后端测试在常规运行中明确 ignored，不计入上述 83 个。
- `cargo +1.88.0 check --workspace --all-targets --locked` 退出 0；这里 all-targets 指本机 Cargo 测试/示例等目标，
  不表示所有 OS/架构的最低工具链实测。
- 稳定工具链分别执行 `cargo check --workspace --all-targets --locked --target <triple>`，7 个非本机核心 target 全部通过：
  Linux x64/arm64 的 GNU 与 musl、macOS x64/arm64、Windows arm64。
  此项只证明交叉编译检查，不证明链接、平台运行或远程 runner 可用性。

### 真实行为与后端

- 首先重现 4 个失败回归，再修复空 file/scheme、重复局部 scheme 空白、无效并发、重复 JVM 参数。
  历史 Python 基线与当前 Rust 的 6 组预览命令流逐项一致。
- `XRESCONV_E2E_JAR` 指向用户提供的 `D:/workspace/github/xresloader/xresloader/target/xresloader-2.23.7.jar`；
  `XRESCONV_E2E_SAMPLE_DIR` 指向同仓库 sample，显式执行 `cargo test --workspace --locked --test real_backend -- --ignored`。
  用户 JAR 的 SHA256 为 `e6e75293f52cdbfef40c042c2077376ca1abca67842c6ff4fa0f5c25bc2e2829`。
- 同时下载并校验官方 v2.23.7 Release JAR，SHA256 为
  `1cd8cfe7415adf46eb4b43376b07841d7fca8324b4bac2e0c7248feabeb11503`，用相同 5 个测试再次通过。
  两个包都报告 2.23.7，哈希不同；此条为当时固定后端的本地验收快照。后续 CI 改为每次查询最新正式 Release，
  按该次 API 的资产 digest 验证 JAR，不混用本机包的校验值。
- 每组 5 个测试覆盖：sample 参考 Lua、JAR 缺失、真实后端失败、proto2 与 proto3 的六种格式。
  两个协议版本 × 六种格式 × 文件/内联 scheme，共 24 个产物分别与独立直接 Java argv 调用逐字节一致。
  两个协议矩阵均经旧 Python 入口及 `XRESCONV_CLI_BIN` 使用本地编译程序。
- 必要样本复制到测试临时目录，后端日志/参考输出/转换输出都留在临时目录，不依赖外部目录可写。
  缺少环境、JAR、样本或参考产物均失败，不能当作跳过后成功。

### 覆盖率、制品与静态检查

- cargo-llvm-cov 0.9.1 + 工具链配套 LLVM 22.1.8；常规测试后续跑真实后端并合并覆盖数据。
  `--ignore-filename-regex '[/\\]tests[/\\]'` 排除独立测试文件；统计 src 模块，包含模块内单元测试代码。
  行覆盖 **1586/1692 = 93.74%**，函数 **157/173 = 90.75%**，区域 **2701/2883 = 93.69%**；90% 行门禁通过。
  JSON 位于 `target/coverage-summary.json`，HTML 位于 `target/llvm-cov/html/index.html`。
  稳定工具链未采集分支覆盖；结果仅代表 Windows 编译分支，系统资源失败和其他 OS 路径不能宣称 100% 覆盖。
- 本机 release 构建通过；`scripts/release.ps1 -Mode Package` 生成 Windows x64 ZIP 与 SHA256。
  `target/final-release/xresconv-cli-2.0.0-x86_64-pc-windows-msvc.zip` 校验值为
  `acdf4d38704269d0d7fae0bf828f7e63993a22a27087a8814b9196463445d3bf`。
  使用实际 Python 安装器解压到独立缓存，版本/帮助/无副作用预览/用户 JAR 真实转表均通过。
- 自动测试验证 ZIP/TAR 的内容、校验、消费，以及 8 个核心包完整性、错误 tag、缺失包和篡改拒绝；不以伪造平台名的包证明跨平台可执行。
- actionlint 1.7.12 对三个 workflow 通过；未安装 ShellCheck，因此禁用该可选调用，不宣称执行 ShellCheck。
  PowerShell 发布脚本解析与实际打包/校验测试通过。
- Markdown lint 16 份文档、0 问题；40 个修改/新增文本的 UTF-8、换行和空白检查通过。
  复用已有 js-yaml 5.2.2 / markdown-it 14.3.0，三个 Skill frontmatter、121 个本地引用与 workflow YAML 解析通过。
  原暂存 fixture 的行末空白仅在工作副本修正，未改动用户索引；最终工作副本用 `git diff HEAD --check` 验证。
- 工具调用修正：覆盖率续跑不能同时给 `--no-report` 和 `--no-clean`，调整后重新实际执行；
  一次性安装包脚本最初误用 UTF-8 解码本机 Python 的 GBK stderr，改为按字节捕获后通过，不是产品测试失败。

尚未执行：GitHub 上 15 目标实际构建、其他平台原生测试、首次 tag 公开发布/在线下载、Python 2.7 运行。
CI 配置覆盖这些构建与测试路径，不能把配置完成或本机交叉检查报告为远程验收完成。
AI 客户端自动路由/独立启动加载没有因本轮 Rust Skill 正文更新而重新验收。

## 2026-09-23 GitHub Actions run 35853777700 故障修复

实际读取 [owent/xresconv-cli 的 run 35853777700](https://github.com/owent/xresconv-cli/actions/runs/35853777700)
所有失败步骤日志，运行提交为 `dc7b36393e722a8bf65e71963f88cc5b2fc7c9fe`。按共同根因归纳：

- Linux/macOS 制品已编译，`scripts/release.ps1` 打包后在 `finally` 用 `Get-Item` 读取隐藏的 `.stage-*` 目录时报
  `Could not find item`；普通测试和覆盖率中的 ZIP/TAR 测试因此失败。已在该安全检查中使用 `Get-Item -Force`。
- 三个系统的真实后端测试读取 Git LFS 的 Excel 指针，xresloader 报“not a valid OOXML file”。
  CI 现在按最新正式 Release tag 检出 sample，启用 LFS、执行 `git lfs pull`，并在转表前检查 Excel 大小及 ZIP 文件头。
- `x86_64-unknown-linux-musl` 的原 PowerShell 冒烟在 `--version` 阶段失败；改为 Unix Bash 冒烟并显式核验可执行文件。
  Windows 仍用 PowerShell。LoongArch 扩展包所需 apt 包在原 Ubuntu 24.04 镜像中不存在，改用 cross 的目标镜像。
- Windows x64、Windows arm64 的常规测试与核心包构建在原 run 已通过；其他平台的本轮修复仍需新的远程 CI 验收。

本地进一步执行：

- 官方 GitHub API 当前最新 xresloader tag 为 `v2.23.7`，资产 digest 为
  `sha256:1cd8cfe7415adf46eb4b43376b07841d7fca8324b4bac2e0c7248feabeb11503`；
  从 workflow 提取的 Bash 解析脚本在该 API 快照上输出匹配 tag/文件名/digest，Adoptium API 返回最新 LTS `25`。
- 独立稀疏克隆同一 tag 并执行 `git lfs pull` 后，Excel 为 45,962 字节的真实 OOXML 文件。
- WSL Debian x64、Rust 1.98.1：使用默认链接器和与 CI 相同的 `musl-gcc` 分别构建发布二进制，
  均实际运行 `--version` 退出 0；提取的 Unix 冒烟脚本在 musl-gcc 构建上通过版本、帮助和预览。
- WSL 中用 PowerShell 7.6.6 实际打包 musl tar.gz，隐藏暂存目录清理通过；完整 Rust 常规测试通过，
  包含先前失败的 ZIP/TAR 消费测试。
- Linux 默认 Java 21 的参考样本仅在本地化时间值与对应 hash_code 不同；给该参考测试显式传入中文 JVM locale 后，
  5 个真实后端测试全部通过，24 个文件/内联 scheme 产物仍与独立 Java 调用逐字节一致。
- Windows Rust fmt/check/test/clippy 门禁重跑通过，常规 84 个 Rust 测试通过（新增 1 个未来后端版本参考归一化测试）。
  Python 离线 unittest 仍为 13 个，由 Rust 测试入口调用。
- Windows 在最终 locale 修复后再次运行官方 JAR 的 5 个真实后端测试，全部通过；
  `cargo +1.88.0 check --workspace --all-targets --locked`、Markdown lint 和 `git diff --check` 通过。
- Windows 用 cargo-llvm-cov 0.9.1 重跑 `--workspace --locked --fail-under-lines 90` 门禁，
  行覆盖 1586/1692 = 93.74%，测试全部通过；此数值是 Windows 覆盖率，不代表 Linux/macOS 分支。
- actionlint 1.7.12 通过其余全部规则；本地仅过滤它尚未收录、但 GitHub 官方 runner 表和镜像目录已列出的
  `ubuntu-26.04-arm` 标签。未安装 ShellCheck，此项未执行。

工作流使用 `ubuntu-latest`、`macos-latest`、`windows-latest`；ARM 原生 runner 没有官方滚动 `latest` 别名，
分别使用当前可用的 `ubuntu-26.04-arm` 与 `windows-11-arm`。
macOS x64 从 `macos-latest` ARM 主机构建，镜像模板包含 Rosetta 安装；这仍需远程冒烟确认。
Python 的 `3.x` 由 setup-python 选择最新稳定 Python 3；Python 官方无单独 LTS 系列，Java LTS 从 Adoptium API 动态解析。
MSRV 1.88.0 是兼容门禁，不是主构建的固定工具链。
未提交、推送、触发新的远程 run 或发布；下一次 GitHub CI 的实际结果仍是外部验收项。

## 2026-09-23 GitHub Actions run 35861863204 故障修复

读取 [run 35861863204](https://github.com/owent/xresconv-cli/actions/runs/35861863204)
的所有失败步骤日志，提交为 `de6a0278ee3d6c79eb68e3e67f188f56c6d17f86`。
五平台普通测试、90% 覆盖率门禁、全部 8 个核心平台构建与冒烟以及除 LoongArch 外的扩展包均通过；失败集中于以下三处：

- Linux/macOS 真实后端测试各 4/5 通过；参考 Lua 对照中的无时区日期相差 28,800 秒，连带 `hash_code` 改变。
  在 WSL 用 `TZ=UTC` 复现同样差异后，参考测试增加 `-Duser.timezone=Asia/Shanghai`，该单项及全部 5 项真实后端测试均通过。
  JVM 语言与时区仅固定参考样本测试，不改变 CLI 正常转表的默认时区。
- Windows 的第二次 `actions/checkout@v7` 在自动物化无关的 `sample/benchmark/资源转换示例-大文件.xlsx`
  时收到 LFS `Authentication required: Bad credentials`。现检出时跳过自动 smudge，
  显式 `git lfs pull --include='sample/资源转换示例.xlsx' --exclude=''`，仍验证工作簿的 OOXML 文件头。
  Windows 本机在公开 tag 的浅克隆中实测：必需 Excel 从 130 字节指针变为 45,962 字节 ZIP，
  benchmark 大文件保持 133 字节指针；随后的 Windows 5 项真实后端测试通过。
- LoongArch 可选包在 cross 构建时将 LoongArch 对象交给 x64 的 `cc`/`rust-lld`，报架构不兼容。
  按产品范围将该非必需目标从矩阵移除；其余 8 个核心目标及 6 个扩展目标保留。

本轮 `actionlint` 对修改后的 workflow 通过，仍只过滤本地 actionlint 1.7.12 未识别的
`ubuntu-26.04-arm` 标签。修复后尚未有新的远程 run；macOS 真实后端和首次 tag 发布待远程验收。
Windows 的 `cargo fmt/check/test/clippy` 全部门禁通过；WSL 普通 Rust 测试 84 项通过。
WSL 首次常规测试因调用环境漏加已有的 PowerShell 7 路径而使 5 个发布脚本测试无法启动；
补入该路径后同一命令全部通过，此次初始失败不是产品断言失败。

## 2026-09-23 GitHub Actions run 35866699467 与门禁收敛

读取 [run 35866699467 的失败 job](https://github.com/owent/xresconv-cli/actions/runs/35866699467/job/107200048669)，
提交为 `75c134dde1a8c86e72f16e48a654fb527c98c3f2`。该次常规测试、覆盖率、8 个核心包及其余构建 job 均成功；
唯一失败的是 Windows 的 Real xresloader job：直接 Java 对照中的 proto2/proto3 两项找不到中文命名的 Excel 文件，
日志显示 `??????.xlsx`，测试结果为 3/5 通过。此日志不足以判定 Rust CLI 行为错误。

按当前产品边界，删除依赖另一仓库 Release/JAR/sample 的 backend、integration job 和 `tests/real_backend.rs`；
不再让外部仓库资源变化阻塞本仓库测试或发布。保留仓内 fixture、fake-java、进程与发布制品测试；
此前真实后端对照仅作为本文件中的历史验收记录。
同时移除 Windows/Linux i686 与 Linux ARM32 发布目标，并使旧 Python 入口不再为这些目标选择不存在的自动下载资产。
LoongArch 已在前一轮移除；现行矩阵为 8 个核心目标与 3 个扩展目标。

本地验收：Windows/WSL Linux 均运行 `cargo test --workspace --locked`，各 83 个 Rust 测试通过；
Python 兼容层的 13 个离线 unittest 通过，包含不支持架构拒绝自动下载及 `XRESCONV_CLI_BIN` 路径优先级。
Windows `cargo fmt --all --check`、`cargo check --workspace --locked`、
`cargo clippy --workspace --all-targets --locked -- -D warnings` 与 Rust 1.88 的 `--all-targets` 检查通过。
Windows cargo-llvm-cov 0.9.1 的 90% 行门禁通过，行覆盖 1586/1692 = 93.74%。
actionlint 1.7.12 验证现行四个 job 和 11 个制品目标，仍仅过滤其未知但 GitHub 已提供的 `ubuntu-26.04-arm` 标签。
本轮未提交、推送或触发解耦后的远程 CI；首次 tag 发布仍未验收。

## 2026-09-23 解耦 CI、首次发布与规范地址修复

- [常规 CI run 35871630941](https://github.com/owent/xresconv-cli/actions/runs/35871630941) 在
  `33f6e54fa72eb3bc0eaf68754dfacd808d0da0d6` 完成，18 个 job 全部成功：五平台原生测试、
  lint、90% 行覆盖率门禁、8 个核心包和 3 个可选扩展包。当前门禁未查询或执行外部 xresloader 资源。
- 从该 run 下载全部 11 个包及 SHA256 到忽略的 `target/plan-release-check/`，
  `scripts/release.ps1 -Mode Verify` 确认 11 个校验和与 8 个核心包齐全；
  全部 11 个压缩包解压后均有普通二进制、README 与 LICENSE。
  解压后的 Windows x64 包执行 `--version`/`--help`，WSL Debian 执行 Linux x64 GNU/musl 包的
  `--version`/`--help`，均成功；其余平台运行证据来自对应 CI runner 的打包前冒烟，
  不声称本机执行了 macOS、ARM、Android、RISC-V 或 FreeBSD 二进制。
- [首次 tag run 35872817845](https://github.com/owent/xresconv-cli/actions/runs/35872817845)
  在同一提交成功，`v2.0.0` 已公开，11 个平台包及各自 SHA256 共 22 个资产。
  GitHub 公开资产 URL 使用仓库规范路径 `owent/xresconv-cli`；旧 Python 入口以独立空缓存和缺失的
  `XRESCONV_CLI_BIN` 实际运行时因 `invalid GitHub release asset URL` 退出 1。
- 修复下载器使用规范 API/页面地址，允许规范及旧组织两种资产路径，仍拒绝外部主机和伪装 owner。
  13 个 Python 离线 unittest 通过；同一独立缓存场景下，修复后的旧入口下载 `v2.0.0`、
  校验 SHA256 并运行，`--version` 输出 `2.0.0`、退出 0。
  `v2.0.1` 本地 `cargo fmt/check/test/clippy --locked` 门禁通过；83 个 Rust 测试通过。
- 按本轮用户指示，规则注入、Skill 发现与 Python 2.7 运行不再作为验收门禁。
  既往未执行记录保留为当时的事实，不再列入 Plan 待办。
- [v2.0.1 tag run 35874108129](https://github.com/owent/xresconv-cli/actions/runs/35874108129)
  在提交 `eae4ffc97d9d576128037c2c7f1c935c5d586e40` 完成，20 个 job 全部成功；
  [公开 Release](https://github.com/owent/xresconv-cli/releases/tag/v2.0.1) 非草稿、非预发布，
  latest API 返回 `v2.0.1` 和 22 个资产。
  重新下载 11 包及 SHA256，`scripts/release.ps1 -Mode Verify` 通过，全部压缩包解压成功；
  解压后的 Windows x64 和 WSL Linux x64 GNU/musl 包执行 `--version`/`--help` 均通过。
  其他目标由各自 CI runner 在打包前运行冒烟；Android、RISC-V、FreeBSD 仅尽力构建与解压。
- 修复后的旧 Python 入口在独立空缓存、`XRESCONV_CLI_BIN` 指向不存在路径时，
  从公开 latest Release 下载并校验 Windows x64 包，转交运行得到 `2.0.1`、退出 0。
