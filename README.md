xresconv-cli
==========

这是一个符合 [xresconv-conf](https://github.com/xresloader/xresconv-conf) 规范的CLI转表工具，并且使用 [xresloader](https://github.com/xresloader/xresloader) 作为数据导出工具后端。

依赖python（python2和python3都支持）

使用说明
------

```bash
python xresconv-cli.py [本脚本选项] <转换列表文件> [附加xresloader选项]
本脚本选项:
-h                          帮助信息
-s <要转换的scheme名称>     按scheme名称指定要转换的表

```

示例截图
------
![示例截图-1](doc/snapshoot-1.png)

![示例截图-2](doc/snapshoot-2.png)