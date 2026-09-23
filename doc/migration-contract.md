# Rust 2.0 迁移合同与测试映射

核验日期：2026-09-23。Python 行为基线为 `656c7e3d44efee978334e0364eb7de0fe3c8778c`；
当前单 Cargo package 的实现按 CLI、XML、选项、规划、执行、进程树和颜色输出分工。
完成状态在 [Plan](../Plan.md)，运行证据在 [验证记录](ai/validation.md)。

## 保留的行为

| 边界 | 合同 | 自动验证 |
| --- | --- | --- |
| CLI | 保留全部短/长选项；scheme/JVM 选项追加；单值重复时最后一个生效；长选项唯一前缀；`--` 终止选项解析 | `src/cli.rs`、`tests/cli_integration.rs` |
| XML include | 相对路径以所在 XML 为准；深度优先、包含文件先合并；重复但不循环的 include 仍重复应用 | `src/xml_conf.rs`、`tests/regressions.rs` |
| XML 文本 | UTF-8/UTF-16、声明编码、内部实体；未知/空节点不生成转换参数；命名空间节点不误识别为无命名空间配置 | `tests/regressions.rs` |
| 全局合并 | 单值按原顺序覆盖；首个非空 data_version 生效，CLI 优先；JVM/option 追加 | `src/xml_conf.rs`、`tests/sample_conf.rs` |
| 多值分组 | output_type、proto_file、data_src_dir/data_source_dir 按来源文件重新分组 | `tests/sample_conf.rs`、`tests/regressions.rs` |
| 局部 scheme | 局部键覆盖默认键；保留重复局部值原始空白；file/scheme 任一为空时使用内联 scheme | `tests/regressions.rs` |
| 输出矩阵 | tag/class 各自任一匹配，两类同时受限时需同时匹配；全局、局部、尾部选项顺序不变 | `src/plan.rs`、`tests/sample_conf.rs` |
| 路径与 Java | 主配置目录加 work_dir；CLI `-J`、JAVA_HOME、PATH 顺序；JVM 参数与 stdin 分开传输 | `tests/cli_integration.rs` |
| 日志 | stdout/stderr 分别排空、保留 Unicode；沿用日志级别着色与 CPRINTF_MODE | `src/runner.rs`、`src/color.rs`、集成测试 |
| Python 入口 | 三个脚本和目录调用打印升级提示；转交参数、工作目录、标准流及退出状态 | `tests/python_entry.rs` |
| 下载 | 环境指定路径、缓存、最新稳定 Release；校验 SHA256；失败不损坏缓存；并发原子安装 | `tests/test_python_shim.py`，由 Cargo 调用 |
| 真实转换 | proto2/proto3，bin/lua/json/xml/msgpack/js，文件 scheme/内联 scheme、输出目录覆盖、中文空格路径 | `tests/real_backend.rs` |

Java 进程数不超过请求的正整数并发上限，也不超过实际命令批次数。每条命令从队列取出一次；
写入失败不盲目重试，避免重复执行已发送的转换。进程/管道/剩余任务失败均使最终状态非零。
正常退出码聚合饱和为 255；配置读取保留 -2、缺失 JAR 保留 -4（Unix 分别为 254、252）。

## 明确修正的历史缺陷

- `--version` 可独立调用；无参数打印帮助并返回 -1（旧 argparse 实际返回 2）。
- 支持 `output_type` 的 `output_dir` 属性；拒绝 `-p 0` 和负数；空任务列表不启动 Java。
- CLI JVM 参数只加一次前导 `-`，并且只追加一次；显式 Java 路径失败不再静默回退。
- include 循环/超过 128 层深度报错；一次失败的 load 不留下已部分加载的配置。
- 后端写管道失败即使进程退出 0 也计为失败；大量双向输出不能造成管道死锁。
- Ctrl+C/终止信号触发取消后关闭本次拥有的进程树并等待回收。Windows 使用 Job Object，Unix 使用独立进程组。
  正常结束也清理仍持有管道的后代；没有新增自动转换超时选项。
- 保留 Windows verbatim/UNC 路径；尾部 CLI 值有空格时保持一个参数；`TERM=dumb` 关闭颜色。

## stdin 协议边界

固定后端 [Main.java](https://github.com/xresloader/xresloader/blob/7263367d99f04af8fca8b1fd80cac29b05f2f6b0/src/org/xresloader/core/Main.java)
使用单双引号或非空白串分词，没有 shell 反斜杠转义。路径中的反斜杠保持原样；包含双引号时可用单引号包裹。
同时含两种引号和空白的单个值无法无损表示，CLI 在启动后端前报错。NUL/CR/LF 不允许变成额外 stdin 命令。
XML 的 `option` 仍是已有后端命令片段，其内部引号由配置作者提供；`--` 后的值遵循真实 argv 边界。
空字符串受后端丢弃空 token 的既有限制，不承诺可以作为后端必填参数值。

## 发布与回滚

- `build.yml` 在普通 push/PR 和 tag 复用调用中运行：六种原生 OS/架构测试、MSRV 检查、三个系统真实后端测试、15 个制品目标。
- 核心 8 包为 Linux GNU/musl x64/arm64、macOS x64/arm64、Windows MSVC x64/arm64。
  7 个扩展目标失败不会阻塞核心发布；其 JDK 可用性和制品运行仍需各平台验收。
- tag 必须匹配 Cargo 版本；只有门禁和必需制品/校验和完整后才发布。发布任务独占内容写权限，构建只读。
- Release 先保持草稿，全部上传成功才公开；预发布不标记 latest；已公开版本拒绝覆盖。
- 用户回滚时下载上一版对应平台包，校验 SHA256 后替换本地安装，或设置 `XRESCONV_CLI_BIN` 指向已验证旧二进制。
  不修改 XML/输入数据，不自动删除旧版。已缓存二进制不会在每次启动时联网升级。
- Python 下载只读取官方仓库 API/制品，限时限量；从压缩包只复制唯一的普通二进制文件，拒绝链接、空文件和路径穿越。

本轮只完成本地修复与发布流程配置，没有提交、推送 tag、上传或公开 Release。
Linux/macOS/ARM 的实际构建和运行验收由首次 CI 结果补齐；Python 2.7 运行未验收。
Windows 本地覆盖率只覆盖本平台编译分支，不能作为跨平台或 100% 分支覆盖证明。
