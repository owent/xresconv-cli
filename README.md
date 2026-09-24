xresconv-cli
==========

![xresconv-cli：表格批量转换与命令行调度](assets/branding/repository-banner.png)

这是一个符合 [xresconv-conf](https://github.com/xresloader/xresconv-conf) 规范的CLI转表工具，并且使用 [xresloader](https://github.com/xresloader/xresloader) 作为数据导出工具后端。

自 2.0.0 起使用 Rust 实现，提供各平台预编译二进制，不再依赖 Python 运行时。（Python 入口保留为兼容转发层，见下文。）

安装
------

从 [GitHub Releases](https://github.com/owent/xresconv-cli/releases) 下载对应平台的预编译包：

| 平台 | 制品 |
| --- | --- |
| Linux x64 | `xresconv-cli-<version>-x86_64-unknown-linux-gnu.tar.gz`（或 `-musl` 静态链接版） |
| Linux arm64 | `xresconv-cli-<version>-aarch64-unknown-linux-gnu.tar.gz`（或 `-musl` 静态链接版） |
| macOS arm64 (Apple Silicon) | `xresconv-cli-<version>-aarch64-apple-darwin.tar.gz` |
| macOS x64 (Intel) | `xresconv-cli-<version>-x86_64-apple-darwin.tar.gz` |
| Windows x64 | `xresconv-cli-<version>-x86_64-pc-windows-msvc.zip` |
| Windows arm64 | `xresconv-cli-<version>-aarch64-pc-windows-msvc.zip` |

以下扩展平台在发布流水线中尽力构建（失败不阻塞发布，不保证每个版本都有产物）：
Linux RISC-V 64、Android arm64、FreeBSD x64。
Windows/Linux i686、Linux ARM32 和 LoongArch 不提供预编译包；旧 Python 入口仍可通过
`XRESCONV_CLI_BIN` 转交用户自行提供的可执行文件。
这些平台仍需能够运行兼容的 Java/xresloader；不提供无独立 CLI/JVM 运行环境的 iOS 包。

解压后将 `xresconv-cli`（Windows 为 `xresconv-cli.exe`）放入 PATH 即可。
运行转表需要 `java` 可执行程序（在 PATH 中，或配置 `JAVA_HOME`，或用 `-J` 指定）。
本工具不校验也不绑定 JDK 版本；实际支持的 JDK 范围由后端 xresloader 决定。

也可以从源码构建（需要 Rust 1.88+ 工具链，最新 XML 编码依赖要求此版本）：

源码中的图标、截图和二进制资源使用 [Git LFS](https://git-lfs.com/)；安装 Git LFS 后，在克隆的仓库中执行：

```bash
git lfs install --local
git lfs pull
```

```bash
cargo build --release --locked
# 二进制位于 target/release/xresconv-cli
```

Windows 构建会将应用图标和版本信息嵌入 `.exe`，需要 Windows SDK 资源编译器；Linux/macOS 保持命令行程序形态。

使用说明
------

```bash
xresconv-cli [本脚本选项]... <转换列表文件> [-- [附加xresloader选项]...]
本脚本选项:
-h, --help                                  帮助信息
-s, --scheme-name <要转换的scheme名称>      按scheme名称指定要转换的表
-v, --version                               显示版本号并退出
-t, --test                                  测试模式（显示运行的脚本，不实际执行）
-p, --parallelism <number>                  正整数并发上限；默认不超过2，实际性能需用自己的数据测量
-j, --java-option                           转递给java的参数（可多个）。比如 -j Xmx2048m
-J, --java-path                             java可执行程序路径
-a, --data-version <version>                数据版本号，将写入到导出的数据文件中。传任意字符串都可以
```

Python 兼容入口
----------------------

`xresconv_cli.py` / `__main__.py` / `xresconv-cli.py` 保留为兼容转发层：打印升级提示后把全部参数转发给 Rust 可执行文件，并透传退出码，旧流程（如 `python xresconv_cli.py ...`）可继续运行。二进制按以下顺序解析：

1. 环境变量 `XRESCONV_CLI_BIN` 指定的可执行文件路径；不存在时提示并继续尝试缓存/下载。
2. 缓存目录中已下载的二进制（Windows 为 `%LOCALAPPDATA%\xresconv-cli\bin`，其他平台为 `${XDG_CACHE_HOME:-~/.cache}/xresconv-cli/bin`）。
3. 以上都缺失时，从 [GitHub Releases](https://github.com/owent/xresconv-cli/releases) 下载最新版本的对应平台制品（校验 sha256 后写入缓存）。

Linux x64/arm64 优先使用 musl 静态包，以兼容不同 libc 发行版。下载具有超时与大小限制，校验失败不会覆盖现有缓存；
安装使用同目录临时文件与原子替换，并发启动不会读到下载了一半的二进制。已缓存版本不会每次联网检查升级。
Python 3.14 已实测；兼容层保留旧入口的调用方式。自动测试使用 Python 3 和本地编译的 Rust 二进制。

与 Python 版的行为差异
----------------------

- `-v/--version` 现在可以独立调用（原实现因 argparse 位置参数必填而无法单独使用）。
- 不带参数时打印帮助并以退出码 `-1` 退出（Unix 上表现为 255）；历史 argparse 实际会在必填参数检查时以 2 退出。
- 支持 `<output_type output_dir="...">` 属性（原实现解析了 rename/tag/class 但遗漏了 output_dir）。
- java 子进程启动失败会明确报错并计入失败数（原实现线程崩溃但失败数不增加）。
- 日志着色兼容 `TERM=dumb`（原实现判断的是 `dump`）。
- 颜色模式仍由 `CPRINTF_MODE` 环境变量控制（`term`/`none`/`win32_console`；`win32_console` 映射为 ANSI 虚拟终端）。
- 拒绝非正数并发；空计划不启动 Java；失败累计不会因 Unix 退出码截断而变成成功。
- 保留 include/全局/局部选项的覆盖顺序、scheme 重复值原文、空 file/scheme 的内联规则及输出矩阵筛选。
  循环 include 和超过 128 层的 include 会明确报错，不再递归至崩溃。
- `-J` 显式指定无效路径时直接报错；相对 Java 路径在切换后端工作目录前解析。
  Ctrl+C/终止信号会停止并回收本次后端进程树，以 130 退出。
- CLI 尾部参数保留空格边界；stdin 参数按 xresloader 实际分词协议选择单双引号。
  后端无法表示的组合（例如同时包含两种引号与空格，或换行/NUL）会报错。详细合同见 [迁移合同](doc/migration-contract.md)。

开发与测试
----------

```bash
cargo fmt --all --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

完整本地测试需要 Python 3 和 PowerShell 7（兼容入口/发布脚本测试），正式二进制不需要它们。
测试包含 [官方 sample 契约](tests/fixtures/)、测试专用 fake-java、进程取消/大量输出/并发、离线下载与缓存、制品校验。
Python 入口通过 `XRESCONV_CLI_BIN` 指向本地 Cargo 二进制；不会下载发布版本。
fake-java 在测试独立目录内从当前源码构建，过滤测试和覆盖率运行不会复用陈旧替身。
覆盖率与测试证据见 [验证记录](doc/ai/validation.md)，不将单个平台的覆盖率视为所有平台分支的完整证明。

CI 只测试本仓库的 Rust/Python 入口、仓内 fixture 和 fake-java，不下载其他仓库的 Release/JAR/sample。
Python 使用最新稳定的 3.x（Python 没有单独的 LTS 发行系列）。
常规 runner 使用 `*-latest`；Linux/Windows ARM 原生 runner 使用 GitHub 提供的专用标签，
macOS x64 包在 `macos-latest` 上构建和冒烟。

发布流程
--------

推送与 `Cargo.toml` 版本一致的 tag（如 `v2.0.1` 或 `2.0.1`）会触发 [release.yml](.github/workflows/release.yml)。
流程复用完整构建/测试门禁，检查全部 8 个核心制品及 SHA256，上传完毕后自动公开 Release。
预发布版本标记 prerelease，不替换 latest；上传失败保留草稿，已公开版本拒绝覆盖。
常规 push/PR 同样构建并保存跨平台包为 Actions artifacts，扩展平台失败不阻塞核心发布。
本地可用 `pwsh -NoProfile -File scripts/release.ps1 -Mode Tag -Tag v2.0.1` 检查 tag。

示例截图
------

应用图标、多尺寸 PNG、Windows ICO 和项目横幅见 [静态资源说明](doc/branding.md)，包含设计、导出与 LFS 维护方法。

![示例截图-1](doc/snapshoot-1.png)

![示例截图-2](doc/snapshoot-2.png)
