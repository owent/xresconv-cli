//! 基于 xresconv-conf 官方 sample.xml / sample_include.xml 的契约测试
//! fixture 来源：https://github.com/xresloader/xresconv-conf （main 分支，2026-09-23）

use std::path::PathBuf;

use xresconv_cli::options::ConvOptions;
use xresconv_cli::plan::build_commands;
use xresconv_cli::xml_conf::XmlConf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load_opts(fixture_name: &str) -> ConvOptions {
    let mut conf = XmlConf::new();
    conf.load(&fixture(fixture_name)).unwrap();
    let mut opts = ConvOptions::new();
    conf.apply_global_entries(&mut opts).unwrap();
    conf.apply_item_entries(&mut opts, &[]);
    opts
}

#[test]
fn sample_xml_global_parsing() {
    let opts = load_opts("sample.xml");

    assert_eq!(opts.work_dir, "../xresloader/sample");
    assert_eq!(opts.xresloader_path, "../target/xresloader-2.14.0-rc3.jar");
    // proto 唯一值
    assert_eq!(
        opts.args.entries()[0],
        ("-p".to_string(), "protobuf".to_string())
    );
    // 空的 <output_dir></output_dir> 与 <rename/> 被跳过
    assert!(!opts.args.entries().iter().any(|(k, _)| k == "-o"));
    assert!(!opts.args.entries().iter().any(|(k, _)| k == "-n"));
    // data_version
    assert_eq!(opts.data_version.as_deref(), Some("1.0.0.0"));
    // java_option 顺序保持
    assert_eq!(
        opts.java_options,
        vec!["-Xmx2048m".to_string(), "-client".to_string()]
    );
    // proto_file
    assert_eq!(
        opts.protocol_files.inputs,
        vec!["-f".to_string(), "\"proto_v3/kind.pb\"".to_string()]
    );
    // default_scheme
    let ds = opts.default_scheme.entries();
    assert_eq!(ds.len(), 2);
    assert_eq!(ds[0].0, "KeyRow");
    assert_eq!(ds[0].1, vec!["2".to_string()]);
    assert_eq!(ds[1].0, "MacroSource");
    assert_eq!(ds[1].1, vec!["资源转换示例.xlsx|macro|2,1".to_string()]);
    // option -> ext_args_l1
    assert_eq!(
        opts.ext_args_l1,
        vec!["--validator-rules custom_validator.yaml".to_string()]
    );
}

#[test]
fn sample_xml_output_matrix() {
    let opts = load_opts("sample.xml");
    let outputs = &opts.output_matrix.outputs;
    assert_eq!(outputs.len(), 3);

    assert_eq!(outputs[0].output_type.as_deref(), Some("bin"));
    assert!(outputs[0].rename.is_none());
    assert!(outputs[0].tags.is_empty() && outputs[0].classes.is_empty());

    assert_eq!(outputs[1].output_type.as_deref(), Some("json"));
    assert_eq!(outputs[1].rename.as_deref(), Some(r"/(?i)\.bin$/\.json/"));
    // Python 版未解析 output_dir 属性（缺陷）；Rust 版支持
    assert_eq!(outputs[1].output_dir.as_deref(), Some("json_output"));

    assert_eq!(outputs[2].output_type.as_deref(), Some("ue-csv"));
    assert_eq!(outputs[2].rename.as_deref(), Some(r"/(?i)\.bin$/\.csv/"));
    assert!(outputs[2].classes.contains("client"));
}

#[test]
fn sample_xml_items() {
    let opts = load_opts("sample.xml");
    assert_eq!(opts.items.len(), 3);
    assert!(opts.items.iter().all(|i| i.enable));

    let item0 = &opts.items[0];
    assert_eq!(item0.file.as_deref(), Some("资源转换示例.xlsx"));
    assert_eq!(item0.scheme.as_deref(), Some("scheme_kind"));
    assert!(item0.classes.contains("server"));

    let item1 = &opts.items[1];
    assert_eq!(item1.options, vec!["--disable-empty-list".to_string()]);

    // 无 file/scheme 属性的 item：使用 scheme_data + 合并 default_scheme
    let item2 = &opts.items[2];
    assert!(item2.file.is_none() && item2.scheme.is_none());
    let keys: Vec<&str> = item2
        .scheme_data
        .entries()
        .iter()
        .map(|e| e.0.as_str())
        .collect();
    assert_eq!(
        keys,
        vec![
            "DataSource",
            "ProtoName",
            "OutputFile",
            "KeyRow",
            "MacroSource"
        ]
    );
    assert_eq!(
        item2.scheme_data.entries()[0].1,
        vec!["资源转换示例.xlsx|arr_in_arr|3,1".to_string()]
    );
    assert_eq!(
        item2.scheme_data.entries()[1].1,
        vec!["arr_in_arr_cfg".to_string()]
    );
    assert_eq!(
        item2.scheme_data.entries()[2].1,
        vec!["arr_in_arr_cfg.bin".to_string()]
    );
}

#[test]
fn sample_xml_command_generation() {
    let opts = load_opts("sample.xml");
    let cmds = build_commands(&opts).unwrap();
    let lines: Vec<String> = cmds.iter().map(|c| c.join(" ")).collect();

    // bin 输出无限制：3 个 item 各一条
    let bin_cmds: Vec<&String> = lines.iter().filter(|l| l.contains("-t bin")).collect();
    assert_eq!(bin_cmds.len(), 3);
    // ue-csv 限定 class=client：只有嵌套数组 item（class="client server"）
    let csv_cmds: Vec<&String> = lines.iter().filter(|l| l.contains("-t ue-csv")).collect();
    assert_eq!(csv_cmds.len(), 1);
    assert!(csv_cmds[0].contains(r#"-m "ProtoName=arr_in_arr_cfg""#));
    assert!(csv_cmds[0].contains(r#"-n "/(?i)\.bin$/\.csv/""#));
    // json 输出带 output_dir 覆盖
    let json_cmds: Vec<&String> = lines.iter().filter(|l| l.contains("-t json")).collect();
    assert_eq!(json_cmds.len(), 3);
    assert!(json_cmds[0].contains(r#"-o "json_output""#));

    // 命令结构：--validator-rules 前缀、-f 协议文件、-p、-a 数据版本
    let first = &lines[0];
    assert!(first.starts_with("--validator-rules custom_validator.yaml -f \"proto_v3/kind.pb\""));
    assert!(first.contains("-p protobuf"));
    assert!(first.contains("-a \"1.0.0.0\""));

    // 3 items x 3 outputs - ue-csv 被过滤 2 = 7 条命令
    assert_eq!(lines.len(), 7);
}

#[test]
fn sample_include_overrides_and_merges() {
    let opts = load_opts("sample_include.xml");

    // 不同文件的 output_type 触发矩阵重置：只剩 lua
    let outputs = &opts.output_matrix.outputs;
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].output_type.as_deref(), Some("lua"));

    // output_dir / rename 覆盖
    let args = opts.args.entries();
    assert!(args.contains(&("-o".to_string(), "\"../..\"".to_string())));
    assert!(args.contains(&("-n".to_string(), "\"/(?i)\\.bin$/\\.lua/\"".to_string())));
    // -p protobuf 从 include 继承
    assert!(args.contains(&("-p".to_string(), "protobuf".to_string())));

    // ext_args_l1 合并：include 在前，本文件在后
    assert_eq!(
        opts.ext_args_l1,
        vec![
            "--validator-rules custom_validator.yaml".to_string(),
            "--pretty 2".to_string()
        ]
    );

    // include 的 item 全部继承
    assert_eq!(opts.items.len(), 3);

    // 命令：3 items x 1 output(lua)
    let cmds = build_commands(&opts).unwrap();
    assert_eq!(cmds.len(), 3);
    let first = cmds[0].join(" ");
    assert!(first.contains("-t lua"));
    assert!(first.contains(r#"-n "/(?i)\.bin$/\.lua/""#));
    assert!(first.contains("-o \"../..\""));
    assert!(first.starts_with("--validator-rules custom_validator.yaml --pretty 2 "));
}

#[test]
fn sample_scheme_name_filter() {
    let mut conf = XmlConf::new();
    conf.load(&fixture("sample.xml")).unwrap();
    let mut opts = ConvOptions::new();
    conf.apply_global_entries(&mut opts).unwrap();
    conf.apply_item_entries(&mut opts, &["scheme_upgrade".to_string()]);
    // 只有 scheme_upgrade 启用；无 scheme 属性的 item 被禁用（Python 中 scheme=False 永不匹配）
    let enabled: Vec<_> = opts.items.iter().filter(|i| i.enable).collect();
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].scheme.as_deref(), Some("scheme_upgrade"));
}
