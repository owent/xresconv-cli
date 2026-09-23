# Rust 重构计划

## 当前状态

2026-09-23：完成当前暂存 Rust 迁移的源码审查与修复，保留原暂存区。
行为基线为 `656c7e3d44efee978334e0364eb7de0fe3c8778c`；合同与测试映射见 [迁移合同](doc/migration-contract.md)。
单 Cargo package、edition 2024、MSRV 1.88；依赖按官方注册表核验，Cargo.toml 使用显式 `^` 兼容范围，Cargo.lock 锁定构建版本。
CLI 不绑定 JDK；历史本机真实验收使用 OpenJDK 25.0.4.1 + xresloader 2.23.7，见验证记录。

本地常规测试、真实后端对照和最低工具链检查已执行；最终数量与覆盖率见 [验证记录](doc/ai/validation.md)。
发布配置包含 8 个核心包、3 个扩展包，tag 通过门禁后自动公开 Release。
[首次 CI run](https://github.com/owent/xresconv-cli/actions/runs/35853777700) 暴露了跨平台打包清理、LFS 样本和 musl 冒烟问题；
已按实际日志修复。[第二次 CI run](https://github.com/owent/xresconv-cli/actions/runs/35861863204)
确认全部 8 个核心包、常规测试和覆盖率通过。
当前 CI 的 LoongArch 交叉构建调用了 x64 链接器；缺少已验证的构建环境，该非必需目标已从矩阵移除。
[第三次 CI run](https://github.com/owent/xresconv-cli/actions/runs/35866699467) 的唯一失败 job 仍来自外部后端样本；
按当前范围删除跨仓库集成门禁及 Windows/Linux 32 位目标，后续 CI 只验证本仓库可控资源。
首次 tag 发布下载与 Python 2.7 运行尚未验收。
本轮未提交、推送、打 tag 或发布。

## 阶段与验收

### P0：工程指引

- [x] 建立共享 `AGENTS.md`、两个按需 Skills、来源索引和文档验证约定。
- [x] 7 份 Markdown lint、两个 Skill frontmatter 与本地引用检查通过。
- [x] Kilo CLI 在仓库根和 `doc/ai` 的原生发现诊断均列出两个项目 Skill（`--pure` 会话）。
- [x] 补齐工程原则、PowerShell/现代工具、客户端和维护流程，以 [逐项覆盖表](doc/ai/requirements-coverage.md) 保存范围。
- [x] 补全后的 15 份文档、三个 Skill 格式/引用及 PowerShell 示例通过；Kilo 根/子目录均发现三个 Skill。
- [ ] 完成规则注入、新增 Skill 的 Codex 独立启动发现、目标客户端真实 Skill 路由及效果对照。
  静态检查与运行验证分别记录在 [验证记录](doc/ai/validation.md)。

### P1：兼容合同

- [x] 对照历史 Python 全部 CLI/XML/规划/调度实现，检查当前所有 Rust 模块、调用方与测试。
- [x] 固定 xresconv-conf fixture 来源与 xresloader 2.23.7 sample 提交，见 [来源索引](doc/ai/source-index.md)。
- [x] 将兼容行为、历史缺陷修正、stdin 表达限制、失败状态与测试对应记录到迁移合同。
- [x] 先执行 4 个失败回归，再修复空属性、重复 scheme 空白、无效并发与重复 JVM 参数。
- [x] 临时运行历史 Python 与 Rust，6 组预览命令流逐项一致，保留原配置合并与筛选语义。

### P2：Rust 实现

- [x] CLI 重复参数/长选项前缀、Java 路径解析和无副作用预览。
- [x] XML 编码、内部实体、命名空间、循环/深度保护以及失败加载回滚。
- [x] 按固定后端实际分词器处理参数；不以 shell 转义替代 stdin 协议。
- [x] 取消时回收自有进程树；stdout/stderr 并行排空；写管道失败与剩余任务返回失败。
- [x] 后端测试替身独立于正式二进制；过滤/覆盖率测试从当前源码构建替身。
- [x] 核验最新稳定依赖；MSRV 1.88 已实际执行本机 `--all-targets` 检查，常规 Cargo 门禁见验证记录。
- [x] 最新稳定工具链完成其他 7 个核心 target 的交叉 `cargo check`；本机 Windows x64 完成实际构建与运行。

### P3：测试与真实后端

- [x] 覆盖 CLI、include/合并/默认 scheme、输出矩阵、引用/Unicode、路径与颜色环境。
- [x] 覆盖并发 1/2/4/100 的无重复遗漏、启动失败、提前退出、大量输出、取消及后代回收。
- [x] Python 三脚本/目录入口均用环境变量转交本地二进制；下载/校验/缓存/异常离线测试。
- [x] 历史固定 JAR + sample 的对照证据保留在验证记录；现行自动测试不依赖外部仓库资源。
- [x] 建立 LLVM 覆盖率与 CI 90% 行覆盖门禁；平台分支和无法稳定注入的系统失败路径单列限制。
- [x] 第二次 CI 已实际运行 Linux/macOS/ARM 常规测试及 8 个核心目标的构建和冒烟。

### P4：发布与替换

- [x] 常规 push/PR 保存预编译 artifacts；tag 复用完整门禁。
- [x] 版本与 tag 一致性、8 个核心制品完整性、SHA256 与 ZIP/TAR 打包消费测试。
- [x] 发布任务独占写权限，Actions 使用官方当前大版本标签；上传成功后公开，预发布不替换 latest。
- [x] CI 只使用仓内 fixture/fake-java；不查询 xresloader Release/JAR/sample。Python 使用最新稳定 3.x。
- [x] 发布目标移除 Windows/Linux i686 与 Linux ARM32；Python 自动下载映射同步，不承诺未构建的架构。
- [x] 修复跨平台 PowerShell 隐藏暂存目录清理；Unix 冒烟改用 Bash。LoongArch 因当前交叉链接配置不可用而移除。
- [x] 环境路径缺失继续缓存/下载；原子安装、有限网络等待、失败不损坏旧缓存。
- [x] README、历史记录、工程指引、Rust Skill、来源与验证记录同步。
- [ ] 解耦后的 GitHub CI 复跑、首次 tag 流程与全部目标解压/运行验收，扩展目标为尽力构建。
- [ ] Python 2.7 运行验收：兼容写法保留，当前机器只有 Python 3。

## 交接

继续从上述外部验收项开始；不要把历史“尚未实现/下一步 P1”描述重新当作现状。
下载或发布失败先检查当前 Release 状态，已有公开版本不覆盖；回滚说明见迁移合同。
以前的 44/46 测试计数及“草稿即发布”状态已由本轮实测纠正，历史审计保留于验证记录与 Git。
