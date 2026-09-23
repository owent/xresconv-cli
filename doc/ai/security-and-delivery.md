# 本地调试、安全边界与交付

按任务选择相关章节。规则说明将来涉及这些场景时应如何执行，不表示当前已部署服务、配置凭据或接入 MCP。
核验日期：2026-09-23；责任：仓库维护者与变更作者。协议和产品升级、授权变化、安全公告或行为异常时复核。

## 本地产物与秘密

- 一次性脚本、日志和产物使用已确认的安全临时目录；需要留在仓库时可采用 `build/<task-name>/`，先核对忽略规则。
  正常 Cargo 输出遵循实际工程配置，不强制迁移。记录任务拥有的路径，避免无主后台进程和残留文件。
- `development/` 可承载开发资源，但目录名或 `secret/` 子目录不构成保护。
  私有配置先核验 Git 忽略、访问权限及秘密扫描；需要模板时只生成无真实值的 `.env.example`。
- 凭据由秘密管理器、系统凭据存储或受控进程环境提供，不让用户粘贴到聊天。
  仅经授权通道交给预期接收方；不写入提示词、argv、日志、普通文件或其他服务。
- 不转储全量环境、认证配置、cookie 或会话文件；避免 shell tracing、含秘密的错误回显和 HTTP 调试输出。
  `jq`/`yq` 解析数据不等于保护传输。发现泄露时保留脱敏证据并按已有授权处理撤销/轮换。
- 清理只针对本任务产物；递归删除/移动前核验解析后的绝对路径和链接目标，保留用户文件及必要证据。

依据：[OWASP Secrets Management][secrets]（rolling，正文已核验）；本项目尚无凭据接入运行验收。

## MCP 与外部工具接入

仅在项目决定采用且任务已有授权时实施。先记录客户端、服务端/SDK、协议版本、传输方式、服务器来源、数据流、读写路径和出站目标。
本轮核对 MCP `2026-07-28` 文档；不能把这版核心/传输行为直接套到旧 SDK，也不因文档较新自动迁移。

- 选择最小工具集、最小权限和最小路径；提示词不能替代 sandbox、认证、权限配置、hooks 或 CI 的实际限制。
- HTTP 按匹配版本的授权流程检查资源/受众、发行方、授权服务发现和重定向，不透传本不属于该服务的 token。
  STDIO 通过受控进程环境供凭据，不机械套用 HTTP OAuth。[授权规范][auth]
- HTTP 校验 Origin、限制监听地址并配置认证；本地默认只监听 loopback，不能因为“本地”就暴露无认证端点。
  STDIO stdout 只放协议，诊断写 stderr。[HTTP][http]、[STDIO][stdio]
- 无状态协议不意味着无业务状态；显式状态句柄不可预测、绑定主体并检查权限，不能拿句柄替代身份认证。
- 校验工具输入、目标路径、符号链接与出站目标，防止路径穿越、越界、SSRF、工具元数据投毒及返回内容中的指令注入。
- 设有限调用超时、输出上限、速率限制和脱敏审计。工具 annotations 的只读/破坏性标记只是提示，不独立构成授权。
- 调用取消遵循该版本的传输机制；服务退出清理自己拥有的进程树，不因一个请求取消杀掉共享服务。
- 先用无副作用样例验证发现、认证、权限拒绝、输入边界、取消/退出及清理，分别记录未通过项。
  配置模板可入库，token、密码、cookie 和会话产物不得进入仓库或模型上下文。

依据：[版本][version]、[安全实践][mcp-security]、[工具规范][mcp-tools]；本仓库未安装/启动 MCP 服务，只有文档核验。

## CI、依赖与交付准备

- 先核对实际目标平台、Rust 工具链/锁文件、JDK 与 xresloader 版本，区分构建所需与运行所需依赖。
- 新依赖/Action/外部 Skill 审计来源、版本、许可、脚本和网络行为。锁版本并保留更新原因，不用“最新版”替代兼容验证。
- GitHub Actions 第三方 Action 固定完整 commit SHA，确认提交属于权威仓库。
  采用最小 token 权限，支持时使用短期凭据，隔离不可信 PR 与持有秘密的发布任务。[GitHub 安全使用][actions]
- 发布准备记录目标环境、制品名/版本/校验信息、配置与迁移顺序、依赖获取、健康/冒烟检查、回滚条件和上一版制品。
  CLI 的发布门禁至少覆盖启动、帮助/版本、预览和仓内 fake-java 的固定转换流程，而非仅能解压；
  不把其他仓库的 Release/JAR/sample 作为本仓库 CI 前提。
- 数据或配置迁移先明确向前/向后兼容、备份与恢复、失败中断点和重复执行行为；先在受控环境预演。
- 发布前先生成可审阅制品及执行/回滚步骤；按已有授权决定实际上传、合并和发布，不反复询问已授权动作。
- 本地/mock、固定版本后端集成、预发布和生产验收分别保存结果。测试环境通过不代表生产成功。
- 超时可能发生在远程动作成功之后；查询实际状态，必要时使用幂等键或去重，不能盲目重放创建/部署/发送操作。[幂等重试][retry]

本仓库的发行结果见 [验证记录](validation.md)；当前没有生产部署验收。
网络可恢复故障遵守退避、抖动与 Retry-After；参数、解析或权限错误先定位，不无限重试。

[secrets]: https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html
[auth]: https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization
[http]: https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http
[stdio]: https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio
[version]: https://modelcontextprotocol.io/docs/2026-07-28/learn/versioning
[mcp-security]: https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices
[mcp-tools]: https://modelcontextprotocol.io/specification/2026-07-28/server/tools
[actions]: https://docs.github.com/en/actions/reference/security/secure-use
[retry]: https://aws.amazon.com/builders-library/making-retries-safe-with-idempotent-APIs/
