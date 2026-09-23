use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::options::{ConvItem, ConvOptions, OutputRule, split_by_spaces};
use crate::plan::quote_argument;

#[derive(Debug)]
pub enum LoadError {
    Io(PathBuf, std::io::Error),
    Parse(PathBuf, roxmltree::Error),
    Invalid(PathBuf, String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(path, err) => write!(f, "{}: {}", path.display(), err),
            LoadError::Parse(path, err) => write!(f, "{}: {}", path.display(), err),
            LoadError::Invalid(path, err) => write!(f, "{}: {}", path.display(), err),
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug)]
pub struct GlobalEntry {
    pub file_path: PathBuf,
    pub tag: String,
    pub text: Option<String>,
    pub attr_rename: Option<String>,
    pub attr_output_dir: Option<String>,
    pub attr_tag: Option<String>,
    pub attr_class: Option<String>,
    pub attr_name: Option<String>,
}

#[derive(Debug)]
pub struct ItemEntry {
    pub file_path: PathBuf,
    pub attr_file: Option<String>,
    pub attr_scheme: Option<String>,
    pub attr_tag: Option<String>,
    pub attr_class: Option<String>,
    pub options: Vec<Option<String>>,
    pub schemes: Vec<(Option<String>, Option<String>)>,
}

#[derive(Debug, Default)]
pub struct XmlConf {
    pub globals: Vec<GlobalEntry>,
    pub items: Vec<ItemEntry>,
}

impl XmlConf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, file_path: &Path) -> Result<(), LoadError> {
        let globals_len = self.globals.len();
        let items_len = self.items.len();
        let result = self.load_inner(file_path, &mut Vec::new());
        if result.is_err() {
            self.globals.truncate(globals_len);
            self.items.truncate(items_len);
        }
        result
    }

    fn load_inner(&mut self, file_path: &Path, active: &mut Vec<PathBuf>) -> Result<(), LoadError> {
        let canonical = std::fs::canonicalize(file_path)
            .map_err(|e| LoadError::Io(file_path.to_path_buf(), e))?;
        if active.contains(&canonical) {
            return Err(LoadError::Invalid(
                file_path.to_path_buf(),
                format!(
                    "include cycle: {}",
                    active
                        .iter()
                        .chain(std::iter::once(&canonical))
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ),
            ));
        }
        if active.len() >= 128 {
            return Err(LoadError::Invalid(
                file_path.to_path_buf(),
                "include depth exceeds 128".into(),
            ));
        }
        active.push(canonical);
        let result = self.load_document(file_path, active);
        active.pop();
        result
    }

    fn load_document(
        &mut self,
        file_path: &Path,
        active: &mut Vec<PathBuf>,
    ) -> Result<(), LoadError> {
        let bytes =
            std::fs::read(file_path).map_err(|e| LoadError::Io(file_path.to_path_buf(), e))?;
        let content =
            decode_xml(&bytes).map_err(|e| LoadError::Invalid(file_path.to_path_buf(), e))?;
        let doc = roxmltree::Document::parse_with_options(
            &content,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .map_err(|e| LoadError::Parse(file_path.to_path_buf(), e))?;
        let root = doc.root_element();

        // 枚举 include 文件（先于本文件的 global/list 处理，与 Python 版一致）
        for include_node in root
            .children()
            .filter(|n| n.has_tag_name("include") && n.tag_name().namespace().is_none())
        {
            let include_file_path = include_node.text().unwrap_or("");
            // Python: if include_file_path and len(include_file_path) > 1
            let mut chars = include_file_path.chars();
            let c0 = chars.next();
            let c1 = chars.next();
            if c0.is_none() || c1.is_none() {
                continue;
            }
            let mut real_path = include_file_path.to_string();
            if c0 != Some('/') && c1 != Some(':') {
                let dir_prefix = file_path.parent().unwrap_or_else(|| Path::new(""));
                real_path = dir_prefix
                    .join(include_file_path)
                    .to_string_lossy()
                    .into_owned();
            }
            self.load_inner(Path::new(&real_path), active)?;
        }

        for node in root
            .children()
            .filter(|n| n.has_tag_name("global") && n.tag_name().namespace().is_none())
        {
            for global_option in node
                .children()
                .filter(|n| n.is_element() && n.tag_name().namespace().is_none())
            {
                self.globals.push(GlobalEntry {
                    file_path: file_path.to_path_buf(),
                    tag: global_option.tag_name().name().to_lowercase(),
                    text: global_option.text().map(|s| s.to_string()),
                    attr_rename: global_option.attribute("rename").map(|s| s.to_string()),
                    attr_output_dir: global_option.attribute("output_dir").map(|s| s.to_string()),
                    attr_tag: global_option.attribute("tag").map(|s| s.to_string()),
                    attr_class: global_option.attribute("class").map(|s| s.to_string()),
                    attr_name: global_option.attribute("name").map(|s| s.to_string()),
                });
            }
        }

        for list_node in root
            .children()
            .filter(|n| n.has_tag_name("list") && n.tag_name().namespace().is_none())
        {
            for item_node in list_node
                .children()
                .filter(|n| n.has_tag_name("item") && n.tag_name().namespace().is_none())
            {
                let options = item_node
                    .children()
                    .filter(|n| n.has_tag_name("option") && n.tag_name().namespace().is_none())
                    .map(|n| n.text().map(|s| s.to_string()))
                    .collect();
                let schemes = item_node
                    .children()
                    .filter(|n| n.has_tag_name("scheme") && n.tag_name().namespace().is_none())
                    .map(|n| {
                        (
                            n.attribute("name").map(|s| s.to_string()),
                            n.text().map(|s| s.to_string()),
                        )
                    })
                    .collect();
                self.items.push(ItemEntry {
                    file_path: file_path.to_path_buf(),
                    attr_file: item_node.attribute("file").map(|s| s.to_string()),
                    attr_scheme: item_node.attribute("scheme").map(|s| s.to_string()),
                    attr_tag: item_node.attribute("tag").map(|s| s.to_string()),
                    attr_class: item_node.attribute("class").map(|s| s.to_string()),
                    options,
                    schemes,
                });
            }
        }

        Ok(())
    }

    /// global 配置解析/合并
    pub fn apply_global_entries(&self, opts: &mut ConvOptions) -> Result<(), String> {
        for global_entry in &self.globals {
            let text_value = global_entry.text.as_deref();
            let trip_value = text_value.map(|s| s.trim()).filter(|s| !s.is_empty());
            let Some(trip_value) = trip_value else {
                continue;
            };
            let text_value = text_value.unwrap_or("");
            let file_path = global_entry.file_path.to_string_lossy().into_owned();

            match global_entry.tag.as_str() {
                "work_dir" => {
                    opts.work_dir = text_value.to_string();
                }
                "xresloader_path" => {
                    opts.xresloader_path = text_value.to_string();
                }
                "proto" => {
                    opts.args.set("-p", trip_value.to_string());
                }
                "output_type" => {
                    if opts.output_matrix.file_path.as_deref() != Some(file_path.as_str()) {
                        opts.output_matrix.outputs.clear();
                        opts.output_matrix.file_path = Some(file_path.clone());
                    }
                    let mut output_rule = OutputRule {
                        output_type: Some(trip_value.to_string()),
                        ..Default::default()
                    };
                    if let Some(rename_rule) = &global_entry.attr_rename
                        && !rename_rule.trim().is_empty()
                    {
                        output_rule.rename = Some(rename_rule.clone());
                    }
                    if let Some(output_dir_rule) = &global_entry.attr_output_dir
                        && !output_dir_rule.trim().is_empty()
                    {
                        output_rule.output_dir = Some(output_dir_rule.clone());
                    }
                    if let Some(tag_rule) = &global_entry.attr_tag
                        && !tag_rule.trim().is_empty()
                    {
                        output_rule.tags = split_by_spaces(tag_rule.trim());
                    }
                    if let Some(class_rule) = &global_entry.attr_class
                        && !class_rule.trim().is_empty()
                    {
                        output_rule.classes = split_by_spaces(class_rule.trim());
                    }
                    opts.output_matrix.outputs.push(output_rule);
                }
                "proto_file" => {
                    if opts.protocol_files.file_path.as_deref() != Some(file_path.as_str()) {
                        opts.protocol_files.inputs.clear();
                        opts.protocol_files.file_path = Some(file_path.clone());
                    }
                    opts.protocol_files.inputs.push("-f".to_string());
                    opts.protocol_files.inputs.push(quote_argument(text_value)?);
                }
                "output_dir" => {
                    opts.args.set("-o", quote_argument(text_value)?);
                }
                "data_src_dir" | "data_source_dir" => {
                    if opts.data_source_dir.file_path.as_deref() != Some(file_path.as_str()) {
                        opts.data_source_dir.inputs.clear();
                        opts.data_source_dir.file_path = Some(file_path.clone());
                    }
                    opts.data_source_dir.inputs.push("-d".to_string());
                    opts.data_source_dir
                        .inputs
                        .push(quote_argument(text_value)?);
                }
                "data_version" => {
                    if opts.data_version.is_none() {
                        opts.data_version = Some(text_value.to_string());
                    }
                }
                "rename" => {
                    opts.args.set("-n", quote_argument(trip_value)?);
                }
                "option" => {
                    opts.ext_args_l1.push(trip_value.to_string());
                }
                "java_option" => {
                    opts.java_options.push(trip_value.to_string());
                }
                "default_scheme" => {
                    if let Some(scheme_key) = &global_entry.attr_name {
                        if opts.default_scheme.contains_key(scheme_key) {
                            opts.default_scheme.push(scheme_key, trip_value.to_string());
                        } else {
                            opts.default_scheme.push(scheme_key, text_value.to_string());
                        }
                    }
                }
                _ => {
                    println!("[ERROR] unknown global configure {}", global_entry.tag);
                }
            }
        }
        Ok(())
    }

    /// 转换项配置解析/合并
    pub fn apply_item_entries(&self, opts: &mut ConvOptions, rule_schemes: &[String]) {
        for item_entry in &self.items {
            let mut item = ConvItem {
                file: item_entry.attr_file.clone(),
                scheme: item_entry.attr_scheme.clone(),
                options: Vec::new(),
                enable: false,
                scheme_data: Default::default(),
                tags: HashSet::new(),
                classes: HashSet::new(),
            };

            if let Some(tag_attr) = &item_entry.attr_tag {
                item.tags = split_by_spaces(tag_attr);
            }
            if let Some(class_attr) = &item_entry.attr_class {
                item.classes = split_by_spaces(class_attr);
            }

            // 局部选项
            for local_option in &item_entry.options {
                let trip_value = local_option
                    .as_deref()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty());
                if let Some(trip_value) = trip_value {
                    item.options.push(trip_value.to_string());
                }
            }

            // 局部 scheme
            for (name_attr, text_value) in &item_entry.schemes {
                let trip_value = text_value
                    .as_deref()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty());
                if trip_value.is_none() {
                    continue;
                }
                if let Some(scheme_key) = name_attr {
                    let raw_text = text_value.clone().unwrap_or_default();
                    // ElementTree preserves local scheme text, including repeated values.
                    // The legacy empty name replaces its previous value.
                    if scheme_key.is_empty() {
                        item.scheme_data.replace(scheme_key, raw_text);
                    } else {
                        item.scheme_data.push(scheme_key, raw_text);
                    }
                }
            }
            for (key, values) in opts.default_scheme.entries() {
                if !item.scheme_data.contains_key(key) {
                    for value in values {
                        item.scheme_data.push(key, value.clone());
                    }
                }
            }

            // 转换规则
            if rule_schemes.is_empty()
                || item
                    .scheme
                    .as_ref()
                    .is_some_and(|s| rule_schemes.iter().any(|r| r == s))
            {
                item.enable = true;
            }

            opts.items.push(item);
        }
    }
}

fn decode_xml(bytes: &[u8]) -> Result<String, String> {
    use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE};
    let declared = if bytes.starts_with(b"<?xml") {
        let end = bytes
            .windows(2)
            .position(|s| s == b"?>")
            .unwrap_or(bytes.len());
        let header = String::from_utf8_lossy(&bytes[..end]);
        let re = regex::Regex::new(r#"encoding\s*=\s*["']([^"']+)["']"#).unwrap();
        re.captures(&header).map(|c| c[1].to_ascii_lowercase())
    } else {
        None
    };
    if matches!(
        declared.as_deref(),
        Some("iso-8859-1" | "latin-1" | "latin1")
    ) {
        return Ok(bytes.iter().map(|&b| char::from(b)).collect());
    }
    if matches!(declared.as_deref(), Some("ascii" | "us-ascii")) && !bytes.is_ascii() {
        return Err("invalid ASCII XML".into());
    }
    let encoding = if let Some((encoding, _)) = Encoding::for_bom(bytes) {
        encoding
    } else if bytes.starts_with(b"\x00<\x00?") {
        UTF_16BE
    } else if bytes.starts_with(b"<\x00?\x00") {
        UTF_16LE
    } else if let Some(label) = declared {
        Encoding::for_label(label.as_bytes())
            .ok_or_else(|| format!("unsupported XML encoding: {label}"))?
    } else {
        UTF_8
    };
    let (text, _, errors) = encoding.decode(bytes);
    if errors {
        Err("invalid XML character encoding".into())
    } else {
        Ok(text.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn load_include_recursive_and_order() {
        let dir = tempfile::tempdir().unwrap();
        write_file(
            dir.path(),
            "sub/included.xml",
            r#"<?xml version="1.0" encoding="utf-8"?>
<root>
  <global><proto>pb</proto></global>
  <list><item file="a.xlsx" scheme="sa"/></list>
</root>"#,
        );
        let main = write_file(
            dir.path(),
            "main.xml",
            r#"<?xml version="1.0" encoding="utf-8"?>
<root>
  <include>sub/included.xml</include>
  <global><output_dir>out</output_dir></global>
  <list><item file="b.xlsx" scheme="sb"/></list>
</root>"#,
        );

        let mut conf = XmlConf::new();
        conf.load(&main).unwrap();
        // include 文件的节点先于本文件节点
        assert_eq!(conf.globals.len(), 2);
        assert_eq!(conf.globals[0].tag, "proto");
        assert_eq!(conf.globals[1].tag, "output_dir");
        assert_eq!(conf.items.len(), 2);
        assert_eq!(conf.items[0].attr_scheme.as_deref(), Some("sa"));
        assert_eq!(conf.items[1].attr_scheme.as_deref(), Some("sb"));
    }

    #[test]
    fn load_missing_file_is_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut conf = XmlConf::new();
        let err = conf.load(&dir.path().join("not-exists.xml")).unwrap_err();
        assert!(matches!(err, LoadError::Io(_, _)));
    }

    #[test]
    fn load_broken_xml_is_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(dir.path(), "bad.xml", "<root><global></root>");
        let mut conf = XmlConf::new();
        let err = conf.load(&path).unwrap_err();
        assert!(matches!(err, LoadError::Parse(_, _)));
    }

    #[test]
    fn short_include_path_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let main = write_file(
            dir.path(),
            "main.xml",
            "<root><include>x</include><list><item file=\"f\" scheme=\"s\"/></list></root>",
        );
        let mut conf = XmlConf::new();
        conf.load(&main).unwrap();
        assert_eq!(conf.items.len(), 1);
    }

    #[test]
    fn global_merge_semantics() {
        let dir = tempfile::tempdir().unwrap();
        let main = write_file(
            dir.path(),
            "main.xml",
            r#"<root>
  <global>
    <work_dir>sub_dir</work_dir>
    <xresloader_path>tools/xresloader.jar</xresloader_path>
    <proto>protobuf</proto>
    <proto>capnp</proto>
    <output_dir>output</output_dir>
    <proto_file>a.proto</proto_file>
    <proto_file>b.proto</proto_file>
    <data_src_dir>csv</data_src_dir>
    <data_version>1.2.3</data_version>
    <rename>rule_$(name)</rename>
    <option>--foo</option>
    <java_option>-Xmx512m</java_option>
    <default_scheme name="ks">v1</default_scheme>
    <default_scheme name="ks">v2</default_scheme>
    <output_type rename="r1" tag="t1 t2" class="c1">bin</output_type>
    <output_type>lua</output_type>
    <unknown_node>x</unknown_node>
  </global>
</root>"#,
        );
        let mut conf = XmlConf::new();
        conf.load(&main).unwrap();
        let mut opts = ConvOptions::new();
        conf.apply_global_entries(&mut opts).unwrap();

        assert_eq!(opts.work_dir, "sub_dir");
        assert_eq!(opts.xresloader_path, "tools/xresloader.jar");
        // 后值覆盖前值且保持首次插入位置
        assert_eq!(
            opts.args.entries()[0],
            ("-p".to_string(), "capnp".to_string())
        );
        assert_eq!(
            opts.args.entries()[1],
            ("-o".to_string(), "\"output\"".to_string())
        );
        assert_eq!(
            opts.protocol_files.inputs,
            vec!["-f", "\"a.proto\"", "-f", "\"b.proto\""]
        );
        assert_eq!(opts.data_source_dir.inputs, vec!["-d", "\"csv\""]);
        assert_eq!(opts.data_version.as_deref(), Some("1.2.3"));
        assert_eq!(
            opts.args.entries()[2],
            ("-n".to_string(), "\"rule_$(name)\"".to_string())
        );
        assert_eq!(opts.ext_args_l1, vec!["--foo"]);
        assert_eq!(opts.java_options, vec!["-Xmx512m"]);
        let ds = opts.default_scheme.entries();
        assert_eq!(ds.len(), 1);
        assert_eq!(ds[0].0, "ks");
        assert_eq!(ds[0].1, vec!["v1".to_string(), "v2".to_string()]);
        assert_eq!(opts.output_matrix.outputs.len(), 2);
        assert_eq!(
            opts.output_matrix.outputs[0].output_type.as_deref(),
            Some("bin")
        );
        assert_eq!(opts.output_matrix.outputs[0].rename.as_deref(), Some("r1"));
        assert_eq!(opts.output_matrix.outputs[0].tags.len(), 2);
        assert_eq!(
            opts.output_matrix.outputs[1].output_type.as_deref(),
            Some("lua")
        );
    }

    #[test]
    fn cli_data_version_overrides_xml() {
        let dir = tempfile::tempdir().unwrap();
        let main = write_file(
            dir.path(),
            "main.xml",
            "<root><global><data_version>from_xml</data_version></global></root>",
        );
        let mut conf = XmlConf::new();
        conf.load(&main).unwrap();
        let mut opts = ConvOptions::new();
        opts.data_version = Some("from_cli".to_string());
        conf.apply_global_entries(&mut opts).unwrap();
        assert_eq!(opts.data_version.as_deref(), Some("from_cli"));
    }

    #[test]
    fn output_matrix_grouped_by_file() {
        let dir = tempfile::tempdir().unwrap();
        write_file(
            dir.path(),
            "inc.xml",
            "<root><global><output_type>bin</output_type></global></root>",
        );
        let main = write_file(
            dir.path(),
            "main.xml",
            "<root><include>inc.xml</include><global><output_type>lua</output_type></global></root>",
        );
        let mut conf = XmlConf::new();
        conf.load(&main).unwrap();
        let mut opts = ConvOptions::new();
        conf.apply_global_entries(&mut opts).unwrap();
        // 不同文件重置分组：只保留本文件的 output_type
        assert_eq!(opts.output_matrix.outputs.len(), 1);
        assert_eq!(
            opts.output_matrix.outputs[0].output_type.as_deref(),
            Some("lua")
        );
    }

    #[test]
    fn item_scheme_filter_and_default_scheme_merge() {
        let dir = tempfile::tempdir().unwrap();
        let main = write_file(
            dir.path(),
            "main.xml",
            r#"<root>
  <global><default_scheme name="ds">dv</default_scheme></global>
  <list>
    <item file="a.xlsx" scheme="sa" tag="x y" class="c">
      <option>--opt1</option>
      <scheme name="k1">v1</scheme>
      <scheme name="k1">v2</scheme>
    </item>
    <item scheme="sb"/>
    <item file="c.xlsx" scheme="sc"/>
  </list>
</root>"#,
        );
        let mut conf = XmlConf::new();
        conf.load(&main).unwrap();
        let mut opts = ConvOptions::new();
        conf.apply_global_entries(&mut opts).unwrap();
        conf.apply_item_entries(&mut opts, &["sa".to_string()]);

        assert_eq!(opts.items.len(), 3);
        assert!(opts.items[0].enable);
        assert!(!opts.items[1].enable);
        assert!(!opts.items[2].enable);
        let item = &opts.items[0];
        assert_eq!(item.options, vec!["--opt1"]);
        assert!(item.tags.contains("x") && item.tags.contains("y"));
        assert!(item.classes.contains("c"));
        let keys: Vec<&str> = item
            .scheme_data
            .entries()
            .iter()
            .map(|e| e.0.as_str())
            .collect();
        assert_eq!(keys, vec!["k1", "ds"]);
        assert_eq!(
            item.scheme_data.entries()[0].1,
            vec!["v1".to_string(), "v2".to_string()]
        );

        // 无过滤时全部启用
        let mut opts2 = ConvOptions::new();
        conf.apply_global_entries(&mut opts2).unwrap();
        conf.apply_item_entries(&mut opts2, &[]);
        assert!(opts2.items.iter().all(|i| i.enable));
    }

    #[test]
    fn unknown_global_tag_is_reported_not_fatal() {
        let dir = tempfile::tempdir().unwrap();
        let main = write_file(
            dir.path(),
            "main.xml",
            "<root><global><what_is_this>1</what_is_this></global></root>",
        );
        let mut conf = XmlConf::new();
        conf.load(&main).unwrap();
        let mut opts = ConvOptions::new();
        conf.apply_global_entries(&mut opts).unwrap();
        // 不 panic、不改变其它状态即符合 Python 行为（仅打印错误）
    }
}
