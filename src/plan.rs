use crate::options::{ConvOptions, OrderedArgs};

/// 生成所有转换命令（每个命令是一组参数，执行时以空格拼接为一行发给 java 进程 stdin）。
/// 按配置正序返回；执行端入栈时反转，弹出顺序即为正序（与 Python 版 cmd_list.reverse() + pop 等价）。
pub fn build_commands(opts: &ConvOptions) -> Result<Vec<Vec<String>>, String> {
    // data_version 设置后覆盖转换列表里的 data_version 选项
    let mut global_cmd_args_map_storage = opts.args.clone();
    if let Some(data_version) = &opts.data_version {
        global_cmd_args_map_storage.set("-a", quote_argument(data_version)?);
    }
    let global_cmd_args_map = &global_cmd_args_map_storage;
    let mut global_cmd_args_prefix_array: Vec<String> = Vec::new();
    let mut global_cmd_args_suffix_array: Vec<String> = Vec::new();

    global_cmd_args_prefix_array.extend(opts.ext_args_l1.iter().cloned());
    for argument in &opts.ext_args_l2 {
        global_cmd_args_suffix_array.push(
            if argument.chars().any(char::is_whitespace) || argument.contains(['\'', '"']) {
                quote_argument(argument)?
            } else {
                argument.clone()
            },
        );
    }

    let mut cmd_list: Vec<Vec<String>> = Vec::new();
    for conv_item in &opts.items {
        if !conv_item.enable {
            continue;
        }

        let empty_rule = [crate::options::OutputRule::default()];
        let item_output_matrix: &[crate::options::OutputRule] =
            if opts.output_matrix.outputs.is_empty() {
                &empty_rule
            } else {
                &opts.output_matrix.outputs
            };

        for item_output in item_output_matrix {
            let mut item_cmd_args_array: Vec<String> = Vec::new();
            item_cmd_args_array.extend(global_cmd_args_prefix_array.iter().cloned());
            item_cmd_args_array.extend(opts.protocol_files.inputs.iter().cloned());
            item_cmd_args_array.extend(opts.data_source_dir.inputs.iter().cloned());

            // tag/class 过滤
            if !item_output.tags.is_empty()
                && !item_output.tags.iter().any(|t| conv_item.tags.contains(t))
            {
                continue;
            }
            if !item_output.classes.is_empty()
                && !item_output
                    .classes
                    .iter()
                    .any(|t| conv_item.classes.contains(t))
            {
                continue;
            }

            let mut item_cmd_args_map: OrderedArgs = global_cmd_args_map.clone();
            if let Some(output_type) = &item_output.output_type {
                item_cmd_args_map.set("-t", output_type.clone());
            }
            if let Some(rename) = &item_output.rename {
                item_cmd_args_map.set("-n", quote_argument(rename)?);
            }
            if let Some(output_dir) = &item_output.output_dir {
                item_cmd_args_map.set("-o", quote_argument(output_dir)?);
            }

            for (key, value) in item_cmd_args_map.entries() {
                item_cmd_args_array.push(key.clone());
                item_cmd_args_array.push(value.clone());
            }

            // add item options
            item_cmd_args_array.extend(conv_item.options.iter().cloned());

            // add item scheme
            if conv_item.file.as_ref().is_some_and(|s| !s.is_empty())
                && conv_item.scheme.as_ref().is_some_and(|s| !s.is_empty())
            {
                item_cmd_args_array.push("-s".to_string());
                item_cmd_args_array.push(quote_argument(conv_item.file.as_deref().unwrap_or(""))?);
                item_cmd_args_array.push("-m".to_string());
                item_cmd_args_array
                    .push(quote_argument(conv_item.scheme.as_deref().unwrap_or(""))?);
            } else {
                for (key, values) in conv_item.scheme_data.entries() {
                    for opt_val in values {
                        item_cmd_args_array.push("-m".to_string());
                        item_cmd_args_array.push(quote_argument(&format!("{key}={opt_val}"))?);
                    }
                }
            }

            item_cmd_args_array.extend(global_cmd_args_suffix_array.iter().cloned());
            if item_cmd_args_array
                .iter()
                .any(|s| s.contains(['\r', '\n', '\0']))
            {
                return Err("xresloader stdin commands cannot contain NUL or line breaks".into());
            }
            cmd_list.push(item_cmd_args_array);
        }
    }

    Ok(cmd_list)
}

/// xresloader 2.23.7 Main.readArgsFromStdin has no backslash escaping.
pub fn quote_argument(value: &str) -> Result<String, String> {
    if value.contains(['\r', '\n', '\0']) {
        return Err("xresloader stdin arguments cannot contain NUL or line breaks".into());
    }
    if !value.contains('"') {
        return Ok(format!("\"{value}\""));
    }
    if !value.contains('\'') {
        return Ok(format!("'{value}'"));
    }
    if !value.chars().any(char::is_whitespace) && !value.starts_with(['\'', '"']) {
        return Ok(value.into());
    }
    Err(
        "xresloader stdin cannot represent an argument containing both quote types and whitespace"
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_conf::XmlConf;
    use std::fs;
    use std::path::PathBuf;

    fn setup(xml: &str) -> ConvOptions {
        let dir = tempfile::tempdir().unwrap();
        let path: PathBuf = dir.path().join("list.xml");
        fs::write(&path, xml).unwrap();
        let mut conf = XmlConf::new();
        conf.load(&path).unwrap();
        let mut opts = ConvOptions::new();
        conf.apply_global_entries(&mut opts).unwrap();
        conf.apply_item_entries(&mut opts, &[]);
        opts
    }

    #[test]
    fn single_item_with_file_and_scheme() {
        let opts = setup(
            r#"<root>
  <global><proto>pb</proto><output_dir>out</output_dir></global>
  <list><item file="a.xlsx" scheme="sa"/></list>
</root>"#,
        );
        let cmds = build_commands(&opts).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            vec![
                "-p",
                "pb",
                "-o",
                "\"out\"",
                "-s",
                "\"a.xlsx\"",
                "-m",
                "\"sa\""
            ]
        );
    }

    #[test]
    fn item_without_file_uses_scheme_data() {
        let opts = setup(
            r#"<root>
  <list><item><scheme name="k">v</scheme></item></list>
</root>"#,
        );
        let cmds = build_commands(&opts).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], vec!["-m", "\"k=v\""]);
    }

    #[test]
    fn output_matrix_expands_and_filters_by_tag() {
        let opts = setup(
            r#"<root>
  <global>
    <output_type tag="client">lua</output_type>
    <output_type>bin</output_type>
  </global>
  <list>
    <item file="a.xlsx" scheme="sa" tag="client"/>
    <item file="b.xlsx" scheme="sb" tag="server"/>
  </list>
</root>"#,
        );
        let cmds = build_commands(&opts).unwrap();
        // 原始顺序：a(lua), a(bin), b(bin；b 的 tag=server 不匹配 lua 的 client 限制)
        assert_eq!(cmds.len(), 3);
        let joined: Vec<String> = cmds.iter().map(|c| c.join(" ")).collect();
        assert_eq!(joined[0], "-t lua -s \"a.xlsx\" -m \"sa\"");
        assert_eq!(joined[1], "-t bin -s \"a.xlsx\" -m \"sa\"");
        assert_eq!(joined[2], "-t bin -s \"b.xlsx\" -m \"sb\"");
    }

    #[test]
    fn output_matrix_class_filter() {
        let opts = setup(
            r#"<root>
  <global><output_type class="c1 c2">bin</output_type></global>
  <list>
    <item file="a.xlsx" scheme="sa" class="c2"/>
    <item file="b.xlsx" scheme="sb" class="c3"/>
  </list>
</root>"#,
        );
        let cmds = build_commands(&opts).unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].join(" ").contains("\"a.xlsx\""));
    }

    #[test]
    fn ext_args_l1_prefix_and_l2_suffix() {
        let mut opts = setup(
            r#"<root>
  <global><option>--ext1</option></global>
  <list><item file="a.xlsx" scheme="sa"/></list>
</root>"#,
        );
        opts.ext_args_l2 = vec!["--tail".to_string(), "tailv".to_string()];
        let cmds = build_commands(&opts).unwrap();
        let line = cmds[0].join(" ");
        assert!(line.starts_with("--ext1 "));
        assert!(line.ends_with(" --tail tailv"));
    }

    #[test]
    fn data_version_adds_a_arg() {
        let mut opts = setup(
            r#"<root>
  <list><item file="a.xlsx" scheme="sa"/></list>
</root>"#,
        );
        opts.args.set("-a", "\"9.9\"".to_string());
        let cmds = build_commands(&opts).unwrap();
        let line = cmds[0].join(" ");
        assert!(line.contains("-a \"9.9\""));
    }

    #[test]
    fn proto_files_and_data_source_before_args() {
        let opts = setup(
            r#"<root>
  <global>
    <proto>pb</proto>
    <proto_file>x.proto</proto_file>
    <data_source_dir>csv</data_source_dir>
  </global>
  <list><item file="a.xlsx" scheme="sa"/></list>
</root>"#,
        );
        let cmds = build_commands(&opts).unwrap();
        assert_eq!(
            cmds[0],
            vec![
                "-f",
                "\"x.proto\"",
                "-d",
                "\"csv\"",
                "-p",
                "pb",
                "-s",
                "\"a.xlsx\"",
                "-m",
                "\"sa\""
            ]
        );
    }

    #[test]
    fn disabled_items_produce_no_commands() {
        let mut opts = setup(r#"<root><list><item file="a.xlsx" scheme="sa"/></list></root>"#);
        for item in &mut opts.items {
            item.enable = false;
        }
        assert!(build_commands(&opts).unwrap().is_empty());
    }

    #[test]
    fn no_output_matrix_runs_once() {
        let opts = setup(r#"<root><list><item file="a.xlsx" scheme="sa"/></list></root>"#);
        assert_eq!(build_commands(&opts).unwrap().len(), 1);
    }
}
