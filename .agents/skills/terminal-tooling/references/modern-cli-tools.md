# 现代 CLI 选择与回退

核验日期：2026-09-23。优先 harness 原生能力与已安装工具；本清单不授权安装，也不把某种实现语言当性能保证。
原始任务提到的外部工具清单未随附件提供；以下为本仓库独立核验的任务相关清单，不声称复刻未读取的外部文件。

## 选择表

“候选”表示已核对官方用途、当前 PATH 未发现；具体平台、版本和参数仍需使用前核验。
所有未实际执行的能力都不能当作已验收功能。

| 场景 | 工具与本机状态 | 使用边界与回退 |
| --- | --- | --- |
| 文本/文件搜索 | [ripgrep][rg] 15.2.0，已运行；[fd][fd] 候选 | 首选 rg/rg --files；文件名遍历可用 fd；回退 Select-String/Get-ChildItem |
| 文件阅读 | [bat][bat] 候选 | 关闭分页/颜色，按行读取；原生读取或 Get-Content 已足够时不引入工具 |
| JSON | [jq][jq] 1.8.2，已运行 | 结构化选择，不用文本替换解析 JSON；回退 ConvertFrom-Json/ConvertTo-Json |
| YAML/配置 | [mikefarah/yq][yq] 4.53.6，已运行 | 先确认实现身份和版本，不把同名工具视为同一语法；回退项目已有解析库 |
| 文本修改 | harness 补丁；[sd][sd] 候选 | 预览限定范围，保留编码/换行，写后检查差异；不全仓替换 |
| 差异比较 | git diff；[difftastic][difft]、[delta][delta] 候选 | 结构差异/视觉展示为辅助；机器证据保留原始 diff，关闭 pager/颜色 |
| 非交互筛选 | [fzf][fzf] 候选 | 自动化按版本使用 --filter；不启动要求 TTY 的选择器或不受控 preview 命令 |
| HTTP | curl.exe 8.21.0，已运行；[xh][xh] 候选 | 设置超时/输出上限，核对重定向与目标；凭据不进入 argv/日志；回退 Invoke-WebRequest |
| 进程与诊断 | [procs][procs]、[bottom][bottom] 候选 | 无人值守用可终止的快照；TUI 不适合自动化阻塞任务；回退 Get-Process |
| 目录列表 | [eza][eza] 候选 | 平台支持使用前确认；脚本优先结构化 Get-ChildItem，避免解析彩色列布局 |
| 磁盘占用/容量 | [dust][dust]、[duf][duf] 候选 | 限定目录和扫描成本；回退 Get-ChildItem/Measure-Object/Get-PSDrive |
| 性能基准 | [hyperfine][hyperfine] 候选 | 固定数据/环境、考虑预热和多次测量；不得对有副作用命令盲目重复；回退受控计时 |
| 代码统计 | [tokei][tokei] 候选 | 排除生成文件和独立 worktree；数量不代表质量，不用统计替代测试 |
| 压缩 | [zstd][zstd] 1.5.7，已运行版本探测 | 使用前验证解压、校验、目标空间及消费方兼容；不覆盖唯一原件 |
| 任务入口 | [just][just] 候选 | 有重复跨平台任务才引入；不为一次命令新建任务体系，优先现有 Cargo 入口 |
| Rust 测试调度 | [cargo-nextest][nextest] 候选 | Cargo 工程建立且有收益时评估；核对测试类型覆盖，不默认替代全部 cargo test 门禁 |
| Rust 依赖政策 | [cargo-deny][deny] 候选 | 配置许可、来源、重复依赖等政策后使用；报告与实际项目配置绑定 |
| Rust 安全公告 | [cargo-audit / RustSec][audit] 候选 | 绑定 Cargo.lock 和公告库日期；更新可能联网；无公告不等于没有风险 |
| 二进制获取 | [cargo-binstall][binstall] 候选 | 可能回退本地编译；禁止编译时明确排除 compile 策略并核验结果 |
| 工具版本管理 | [mise][mise]、[aqua][aqua] 候选 | 按后端/注册表/平台判断安装行为，不承诺统一免编译；无需要不增加管理层 |

## 输出与搜索合同

- 限定路径、glob、编码、最大范围和输出规模，按能力关闭颜色与分页；结构化输出优先。
- `rg` 默认可能跳过隐藏文件、忽略项和二进制；确有需要才显式扩展，不用“没搜到”证明文件不存在。
  `rg --max-count` 是每个文件的匹配行数限制，不是总输出上限。
- 工程搜索默认排除 `.git`、`.kilo/worktrees` 和生成产物；需要当前修改集合时先查 Git 的已跟踪/未跟踪范围。
- `jq -e` 的 0/1/4 与 `rg` 的 0/1/2 分别处理，细节见 [PowerShell 退出码](powershell.md#错误与退出码)。
- `jq`/`yq` 只是解析工具，不是秘密传输渠道；对错误输出、调试日志和外发目标同样做数据边界检查。
- 差异工具和基准输出辅助判断，最终还需源码、合同、实际测试及可重现输入。

fzf 非交互筛选参数的依据为 [官方手册](https://raw.githubusercontent.com/junegunn/fzf/master/man/man1/fzf.1)。

## 按需安装与更新

1. 先确认现成工具/平台原生回退能否满足任务；没有必要就不安装。
2. 真正需要时明确产品/包身份、目标版本、架构、平台、官方发行来源和写入位置；包管理器名称可能不同，查实际 manifest。
3. 在现有授权范围内选官方 release、已用包管理器或已核验的 Cargo 安装路径。
   验证校验和/签名及来源；不执行陌生下载脚本，不自动修改全局 PATH/profile。
4. `cargo binstall` 可能找不到预编译制品而回退 `cargo install`。有“禁止本地编译”约束时，先读当前 `--help`，
   选择不含编译的策略；找不到制品就使用回退工具或报告不可用，不违反约束继续安装。
5. mise/aqua 的后端与包配置不同；检查解析后的获取/构建步骤，而不是把工具名字视为安装方式保证。
6. 安装/升级后核验实际可执行文件、版本和一个无副作用场景，记录退出码及回滚方式；不升级无关依赖。

## 核验范围

本机通过 Get-Command 探测表中工具；rg/jq/yq/zstd/curl.exe 实际版本命令均成功。
其余为 PATH 未发现，不代表系统所有位置都未安装。候选能力依据官方仓库/文档概览，安装与运行未验收。
nextest 文档站本轮获取失败，已改查官方源码仓库；不把站点失败写成工具问题。
`source_version`：官方 rolling/main/master；`installed_version` 见表；维护人：仓库维护者与变更作者。
相关任务或工具升级、行为/参数变化时复核，不在常规开发前全量重查。

[rg]: https://github.com/BurntSushi/ripgrep/blob/master/GUIDE.md
[fd]: https://github.com/sharkdp/fd
[bat]: https://github.com/sharkdp/bat
[jq]: https://jqlang.org/manual/
[yq]: https://github.com/mikefarah/yq
[sd]: https://github.com/chmln/sd
[difft]: https://github.com/Wilfred/difftastic
[delta]: https://github.com/dandavison/delta
[fzf]: https://github.com/junegunn/fzf
[xh]: https://github.com/ducaale/xh
[procs]: https://github.com/dalance/procs
[bottom]: https://github.com/ClementTsang/bottom
[eza]: https://github.com/eza-community/eza
[dust]: https://github.com/bootandy/dust
[duf]: https://github.com/muesli/duf
[hyperfine]: https://github.com/sharkdp/hyperfine
[tokei]: https://github.com/XAMPPRocky/tokei
[zstd]: https://github.com/facebook/zstd
[just]: https://just.systems/man/en/
[nextest]: https://github.com/nextest-rs/nextest
[deny]: https://embarkstudios.github.io/cargo-deny/
[audit]: https://rustsec.org/
[binstall]: https://github.com/cargo-bins/cargo-binstall
[mise]: https://mise.jdx.dev/
[aqua]: https://aquaproj.github.io/
