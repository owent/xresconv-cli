# 应用图标与静态资源

## 设计与使用

图形以六个青色表格单元表达结构化数据，以橙色右箭头和浅色下划线表达命令行转换。
深蓝圆角底板适配深浅背景；小图标保留图形，不放产品名。横幅使用完整项目名及简短说明，
用于 README 和仓库社交预览，不包含易过期的版本号或性能承诺。

| 资源 | 尺寸 / 格式 | 用途 |
| --- | --- | --- |
| [图标原图](../assets/source/icon.png) | 1254 × 1254 RGBA PNG | 保留生成原图及透明通道，派生图标的唯一输入 |
| [横幅原图](../assets/source/banner.png) | 1774 × 887 RGB PNG | 保留生成原图，派生横幅的唯一输入 |
| [应用图标](../assets/icons/xresconv-cli-1024.png) | 1024 × 1024 PNG | 高分辨率展示 |
| [图标目录](../assets/icons/) | 16、24、32、48、64、128、256、512、1024 px PNG | 文档、启动器与各尺寸显示 |
| [Windows 图标](../assets/icons/xresconv-cli.ico) | 16、24、32、48、64、128、256 px，32 位 PNG 帧 | Windows 可执行文件及快捷方式 |
| [项目横幅](../assets/branding/repository-banner.png) | 1280 × 640 PNG | README；可手动上传至仓库社交预览设置 |

设计色板为深蓝 `#101D2F`、青色 `#42C9D8`、橙色 `#FF895B`、浅白 `#F2F5F7` 和灰色 `#A5B6C7`。
这是生成提示中的目标色值；实际位图包含抗锯齿和细微色差。所有资源沿用仓库的 [MIT 许可](../LICENSE)。

[build.rs](../build.rs) 按 Cargo **目标系统**判断是否嵌入 ICO。Windows 构建使用
`winresource 0.1.31`，关闭 TOML 默认 feature，仅增加 `version_check` 间接构建依赖；
MSVC 构建需要已有 Windows SDK 的资源编译器。图标是 LFS 指针或缺失时给出 `git lfs pull` 提示，
构建失败，不静默生成缺图标程序。版本信息读取 Cargo 包元数据。

Linux/macOS 仍交付命令行程序；PNG 可供桌面集成使用，当前没有应用包或桌面启动器。
横幅已在 README 引用；修改本地资源不会自动更改 GitHub 社交预览设置。

## 导出与维护

内置 imagegen 于 2026-09-24 生成两张原图。未使用外部商标、字体文件或第三方素材；
生成提示词保存在下文，图标是第二次生成横幅时的视觉参考。重新调用生成模型不保证字节一致；
日常改尺寸使用保存的原图，不重新生成。

从任意工作目录调用 [导出脚本](../scripts/export-assets.ps1)，示例在仓库根执行：

```powershell
pwsh -NoLogo -NoProfile -NonInteractive -File scripts/export-assets.ps1
```

依赖 Windows + PowerShell 7 的 System.Drawing，不联网或安装依赖；普通 Cargo 构建无需执行它。
脚本检查原图尺寸和图标透明角，保持宽高比、透明通道，以高质量缩放输出 PNG 并封装 ICO。
只覆盖 `assets/icons/` 和 `assets/branding/` 下的约定派生文件，保留 `assets/source/` 原图。
相同原图及渲染环境下可重复导出；跨系统渲染器不保证字节一致。

## Git LFS 约定

[.gitattributes](../.gitattributes) 是唯一跟踪规则来源：

- `assets/**` 下的原图和派生静态资源全部进入 LFS。
- 图像、矢量图、字体、媒体、设计文件、常见二进制及归档扩展名进入 LFS；根规则的扩展名按小写命名。
- 无扩展名的入库二进制放入 `binaries/`，该目录下的文件统一进入 LFS。
- 源码、脚本、Markdown、Cargo 锁文件及普通配置保留 Git 文本管理；说明文件放在 `doc/`。
- `target/` 保持忽略；LFS 跟踪规则不会自动把构建输出加入版本控制。

首次克隆后，在仓库根执行：

```text
git lfs install --local
git lfs pull
```

两个现有 `doc/snapshoot-*.png` 随本次资源变更转换为 LFS 指针，工作区图像字节保持不变。
迁移从当前版本开始，不重写历史提交；历史中的原始 Git blob 仍保留。
常规提交和推送时应一起包含规则、资源指针及代码；LFS pre-push hook 负责上传引用的对象。

规则变化后，仅对受影响的已跟踪文件执行规范化，避免暂存无关修改：

```text
git add .gitattributes
git add --renormalize -- doc/snapshoot-1.png doc/snapshoot-2.png
git add assets
git lfs ls-files
git lfs fsck --objects
```

`git lfs fsck --objects` 检查本地对象，不能证明远端已上传。构建与发布 workflow 的六处
`actions/checkout@v7` 均设置 `lfs: true`，在消费资源前拉取实际内容。
自动生成的源码归档是否包含 LFS 内容取决于仓库设置；需要完整资源时使用 Git + LFS 克隆。

## 来源与核验依据

核验日期：2026-09-24。生成方式为内置 imagegen；本机导出环境为 PowerShell 7.6.6 / System.Drawing，
Git LFS 为 3.7.1。运行验证结果以本次实际检查为准。

- [Git LFS 3.7.1 track](https://github.com/git-lfs/git-lfs/blob/v3.7.1/docs/man/git-lfs-track.adoc)
- [GitHub LFS 配置](https://docs.github.com/en/repositories/working-with-files/managing-large-files/configuring-git-large-file-storage)
- [actions/checkout LFS 选项](https://github.com/actions/checkout#usage)
- [Windows 图标设计](https://learn.microsoft.com/en-us/windows/win32/uxguide/vis-icons)
- [winresource 0.1.31 API](https://docs.rs/winresource/0.1.31/winresource/struct.WindowsResource.html)；
  依赖及编译行为同时核对本机缓存的该版本源码和 Cargo.toml。

## 本地验证记录

2026-09-24，在 Windows x64、Rust/Cargo 1.98.0、PowerShell 7.6.6、Git LFS 3.7.1 下执行。
完整 Cargo 门禁、release、MSRV 和目标检查对应包版本 2.0.1 的图标代码（提交 `93a502c`）；
随后包版本更新至 2.0.2，另行运行 Windows 图标测试通过，系统读取的 FileVersion、ProductVersion
均为 2.0.2，OriginalFilename 为 `xresconv-cli.exe`。

- 四项默认 Cargo 门禁全部通过；`cargo test --workspace --locked` 实际执行 84 个测试。
  新增 Windows 资源测试通过系统 API 读取生成的 EXE，将 7 个图标帧与 ICO 原始字节逐一比对。
- `cargo test --release --test windows_resources --locked --offline`：1 个测试通过，确认发布优化后仍包含图标。
- `cargo +1.88.0 check --workspace --all-targets --locked --offline` 通过。
- Linux x64 GNU、macOS ARM64 的 `cargo check --workspace --target <目标> --locked --offline` 通过；
  这是 Windows 主机上的目标编译检查，不代表对应系统上的链接或运行验收。
- Windows ARM64 release 的资源编译成功，但最终链接未通过：本机未找到 MSVC ARM64 交叉链接器，
  Rust 回退调用 PATH 中的 coreutils `link.exe`。未安装额外工具；ARM64 实际运行仍由原生 CI 验证。
- 9 种 PNG 尺寸及透明角检查通过；32px 图标和横幅已目视核验。
  从包含中文和空格的独立目录导出 11 个派生文件，SHA256 与交付资源一致；
  输入 LFS 指针时导出失败，已有产物保持不变。
- 15 个资源的 LFS 指针长度、SHA256 与实际文件一致；`git lfs fsck` 通过。
  两张已有截图与迁移前的 Git blob 内容一致。未在本次验证中执行远端下载或上传验收。
- markdownlint-cli2 0.23.2 检查 16 个文档通过；Git 差异空白、新增文本 UTF-8、PowerShell 语法检查通过；
  yq 解析两份 workflow，确认六处 checkout 均启用 LFS。本次未执行远程 CI 验收。

本次新增及更新的命令日志保留于本机忽略目录 `target/branding-validation/`。

## 原图生成提示词

### 图标

```text
Use case: logo-brand.
Asset type: production application icon for the open-source xresconv-cli command-line batch table conversion tool.
Primary request: Design one polished, distinctive square app icon, 1024 by 1024. A simple bold flat geometric
emblem showing tabular data becoming a terminal prompt: on the left, six solid cyan square cells in a precise
2-column by 3-row grid; on the right, a substantial warm coral-orange right chevron and a short pale ivory terminal
underscore. Treat these as one balanced integrated symbol, with generous negative space and optically consistent
thick strokes, readable at 32 pixels.
Scene/backdrop: a deep ink-navy rounded-square app tile, almost filling the image with an approximately 6 percent
transparent safety margin. Outside the tile is genuinely transparent alpha, not white and not a checkerboard.
Style/medium: premium minimal vector-like branding rendered as a crisp raster; mathematically tidy geometry, no
outlines around the grid cells; flat fills, slightly softened corners only.
Color palette: ink navy #101D2F tile, cyan #42C9D8 grid, coral #FF895B chevron, ivory #F2F5F7 underscore.
Composition/framing: one centered app icon only, frontal view, absolutely no perspective; symbol fills about two
thirds of the tile.
Constraints: no letters, no words, no watermark, no mockup, no surrounding objects, no border frame, no gradients,
no glow, no drop shadow. This is the finished application asset, not a presentation board. Preserve the actual
transparent alpha around the rounded tile.
```

### 横幅

输入参考：`assets/source/icon.png`。

```text
Use case: logo-brand.
Asset type: finished repository banner and social-preview image for xresconv-cli, landscape 1280 by 640 pixels,
exact 2:1 aspect ratio.
Input images: Image 1 is the authoritative new application icon, a brand reference. Preserve its six cyan table
cells and orange terminal chevron with ivory underscore. Use the same visual identity, same proportions, and same
simple flat geometric style.
Primary request: Create a refined, minimal open-source developer-tool banner. Ink navy #101D2F background. A large
faithful version of the icon emblem sits on the left; on the right a clean typography lockup with the title exactly
"xresconv-cli" in large pale ivory, bold modern monospaced typography. Beneath it, smaller muted cool gray text
exactly "Batch tables. One command." Use generous clear spacing. A small cyan label above the title reads exactly
"XRESLOADER TOOLCHAIN". All text must be crisp, correctly spelled and comfortably inset from the edges.
Composition/framing: emblem at left occupies about 30 percent width; typography at right about 58 percent;
vertically centered content, generous outer margins of at least 8 percent. A subtle, sparse grid of very fine
navy-blue lines in the background hints at structured tabular data; keep it low contrast and avoid visual clutter.
Color palette: ink navy #101D2F, cyan #42C9D8, coral #FF895B, ivory #F2F5F7, gray #A5B6C7.
Constraints: no other words, no version numbers, no performance claims, no mockup, no devices, no watermark, no
decorative badges, no drop shadows, no 3D. Entire image is the finished banner, opaque background, no surrounding
frame.
```
