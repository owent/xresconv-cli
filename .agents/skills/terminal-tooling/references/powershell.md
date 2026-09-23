# PowerShell 与 Windows 执行约束

适用于命令构造、脚本、子进程、文件操作与排障。核验日期：2026-09-23；本机 PowerShell 7.6.6。
官方解析和编码依据见文末；7.5 文档不等于所有版本行为一致，关键分支仍用当前解释器核验。

## 解释器与能力

- 优先 PowerShell 7+ `pwsh`，先核验 `$PSVersionTable.PSVersion` 和 harness 实际启动器。
  单独启动自动化 shell 使用 `-NoLogo -NoProfile -NonInteractive`，避免 profile、横幅和交互污染结果。
- 仅有其他 shell 时使用已验证的兼容路径并报告差异；不在同一删除、移动或改写操作中混用 PowerShell、cmd 和其他 shell。
- 用 `Get-Command` 或明确可执行路径确认程序；区分 `curl.exe` 与可能存在的同名别名，使用 `Where-Object` 而非含糊的 `where`。
- 环境只探测必要变量，不打印全部环境或配置。任务变量采用 `$taskRoot`、`$toolArgs` 等名称，不覆盖系统变量或 `$CODEX_HOME`。

## 字面量、路径与参数

- 无插值文本用单引号，需要插值时才用双引号。PowerShell 的转义字符不是反斜杠；避免多层 shell 与反引号续行。
- 多行文本使用 here-string；保留真实换行。PR/Issue 内容优先结构化参数，CLI 则用临时正文文件与 `--body-file`。
- 文件 cmdlet 优先 `-LiteralPath`，关键操作加 `-ErrorAction Stop`。通配符用于搜索范围，不能误用于写入或清理目标。
- 使用调用运算符和参数数组，不用拼接可执行命令，不将不可信内容交给 `Invoke-Expression`。
  `JSON.stringify()` / `ConvertTo-Json` 是序列化，不是 shell 转义；其中的 `$()`、反引号或引号仍可能被下一层解释。
- PowerShell 7.3+ 的原生参数行为受 `$PSNativeCommandArgumentPassing` 控制。
  Windows 模式对部分程序、`.bat`/`.cmd` 等回退 Legacy；参数数组不是所有程序的自动引号修复。
- `--%` 仅适用于特定 Windows 原生命令场景，仍可能展开 `%ENV%`，不能跨换行/管道当通用修复。
  `--` 是否终止选项解析取决于被调用命令；不要混淆两者。
- 对空参数、嵌入引号、尾随反斜杠、非 ASCII 和空格路径做无副作用的参数往返验证。
  调试不得输出含凭据的完整 argv；必要时用临时输入文件或程序的结构化 stdin。

仓库根的只读搜索示例；`rg` 返回 1 不属于执行错误：

```powershell
$toolArgs = @('--files', '--hidden', '-g', '!.git', '-g', '!.kilo/worktrees/**')
& rg @toolArgs
$toolExit = $LASTEXITCODE
if ($toolExit -notin @(0, 1)) { throw "rg failed: $toolExit" }
```

语句块输出接管道时加调用运算符，不把 `foreach (...) { ... }` 直接接到管道：

```powershell
& {
    foreach ($toolName in @('rg', 'jq', 'yq')) {
        $toolCommand = Get-Command $toolName -ErrorAction SilentlyContinue | Select-Object -First 1
        [pscustomobject]@{ name = $toolName; available = [bool]$toolCommand }
    }
} | ConvertTo-Json
```

## 错误与退出码

- cmdlet 的错误处理与原生命令退出码是两个合同。关键 cmdlet 用 `-ErrorAction Stop`；脚本需要时显式设置错误偏好并说明范围。
- 执行原生命令后立即保存 `$LASTEXITCODE`，再运行下一条原生命令；不能只看 `$?`、stderr 或“没有抛异常”。
  检查 `$PSNativeCommandUseErrorActionPreference` 的实际影响，不假定 `try/catch` 自动接住所有非零状态。
- 用包装脚本执行子命令时显式传递最终失败退出码；诊断写 stderr，机器结果放 stdout。
  流合并 `2>&1` 可能破坏 JSON 解析，按通道捕获并分别处理。
- `rg`：0 为匹配，1 为无匹配，2 为错误。`jq -e`：有效真值结果为 0，末项 false/null 为 1，没有有效结果为 4。
  解析错误和参数错误不能当作无匹配；其他工具查自身帮助和版本合同。

## 编码与文件写入

- 新共享文本显式 UTF-8。现有文件尽量保留 BOM、换行和末尾换行；本仓库 Markdown 遵循 `.gitattributes` 的 CRLF。
- PowerShell 7 默认 UTF-8 行为不应被泛化到 Windows PowerShell 5.1；5.1 的各文件 cmdlet 默认编码并不一致。
  不能声称其所有写入都是 UTF-16LE；也不要用未指定编码的重定向/追加改写项目文件。
- 精确改动优先补丁。确需写完整文本时核验编码、目标和内容，避免附加多余换行；比较字节差异以发现无关重编码。
- 原生输出、管道文本、文件编码与控制台显示是不同层次；二进制输出使用保留字节的接口，不能经文本处理再称字节不变。

## 后台进程、超时与清理

- 独立进程用 `Start-Process` 时默认 `-WindowStyle Hidden`，保存 `-PassThru` 返回的进程对象、PID 和日志路径。
  参数引用须验证，不能假定 `-ArgumentList` 自动保留所有数组元素边界；必要时使用支持独立参数列表的进程 API。
- 区分工具等待窗口、子进程截止时间与任务总预算。工具返回 session/cell ID 表示进程仍在运行，继续用对应等待入口。
- 根据实际 CI/历史耗时和风险设有限预算；禁止无依据统一超时或无限日志跟随。
  超时先保留脱敏诊断、检查下载/死锁/资源/残留进程，再决定是否终止或重试。
- 只对可恢复错误有限重试；网络退避、抖动并遵守 `Retry-After`。
  解析/参数/权限错误先定位；有副作用的操作先核查是否已经成功，使用幂等或去重机制。
- 清理限本任务拥有的进程树和路径。递归删除/移动前解析绝对目标、检查链接/重解析点并确认仍在授权范围内。
  不删除整个临时根、工作区或独立 worktree；不把 PowerShell 枚举的路径拼给 `cmd /c` 处理。
- 登录必须走受控交互界面，不让模型收集秘密；无人值守流程不得等待输入。
- `CreateProcessAsUserW` 等启动器故障不等于脚本失败：先确认进程能否启动，按平台批准机制重试，不降低沙箱或安全设置。

## 核验依据

- [PowerShell 解析规则][parsing]
- [PowerShell 编码规则][encoding]
- [jq 退出状态](https://jqlang.org/manual/#invoking-jq)

`source_version`：PowerShell 7.5 文档、jq rolling；`status`：正文已核验，Windows 本机例证见项目验证记录。
不同 shell/版本的运行行为未统一验收；升级、引用丢失或编码差异时复查。维护责任为本仓库维护者与变更作者。

[parsing]: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_parsing?view=powershell-7.5
[encoding]: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_character_encoding?view=powershell-7.5
