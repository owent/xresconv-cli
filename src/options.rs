use std::collections::HashSet;

/// 保持插入顺序的键值表，用于模拟 Python dict 的插入序迭代行为
#[derive(Debug, Clone, Default)]
pub struct OrderedArgs {
    entries: Vec<(String, String)>,
}

impl OrderedArgs {
    pub fn set(&mut self, key: &str, value: String) {
        for entry in &mut self.entries {
            if entry.0 == key {
                entry.1 = value;
                return;
            }
        }
        self.entries.push((key.to_string(), value));
    }

    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }
}

/// 保持插入顺序的多值表（default_scheme / item scheme_data）
#[derive(Debug, Clone, Default)]
pub struct OrderedMultiMap {
    entries: Vec<(String, Vec<String>)>,
}

impl OrderedMultiMap {
    pub fn replace(&mut self, key: &str, value: String) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.0 == key) {
            entry.1 = vec![value];
        } else {
            self.push(key, value);
        }
    }
    pub fn push(&mut self, key: &str, value: String) {
        for entry in &mut self.entries {
            if entry.0 == key {
                entry.1.push(value);
                return;
            }
        }
        self.entries.push((key.to_string(), vec![value]));
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.iter().any(|entry| entry.0 == key)
    }

    pub fn entries(&self) -> &[(String, Vec<String>)] {
        &self.entries
    }
}

#[derive(Debug, Clone, Default)]
pub struct OutputRule {
    pub output_type: Option<String>,
    pub rename: Option<String>,
    /// 对应 <output_type output_dir="...">：Python 版未解析该属性（缺陷），Rust 版支持
    pub output_dir: Option<String>,
    pub tags: HashSet<String>,
    pub classes: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct OutputMatrix {
    pub file_path: Option<String>,
    pub outputs: Vec<OutputRule>,
}

#[derive(Debug, Default)]
pub struct GroupedInputs {
    pub file_path: Option<String>,
    pub inputs: Vec<String>,
}

#[derive(Debug)]
pub struct ConvItem {
    pub file: Option<String>,
    pub scheme: Option<String>,
    pub options: Vec<String>,
    pub enable: bool,
    pub scheme_data: OrderedMultiMap,
    pub tags: HashSet<String>,
    pub classes: HashSet<String>,
}

#[derive(Debug)]
pub struct ConvOptions {
    pub conv_list: String,
    pub args: OrderedArgs,
    pub ext_args_l1: Vec<String>,
    pub ext_args_l2: Vec<String>,
    pub work_dir: String,
    pub xresloader_path: String,
    pub items: Vec<ConvItem>,
    pub parallelism: i64,
    /// 来自 XML `<java_option>` 的 JVM 参数（原样追加）
    pub java_options: Vec<String>,
    pub java_path: String,
    pub default_scheme: OrderedMultiMap,
    pub data_version: Option<String>,
    pub output_matrix: OutputMatrix,
    pub protocol_files: GroupedInputs,
    pub data_source_dir: GroupedInputs,
}

impl Default for ConvOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvOptions {
    pub fn new() -> Self {
        ConvOptions {
            conv_list: String::new(),
            args: OrderedArgs::default(),
            ext_args_l1: Vec::new(),
            ext_args_l2: Vec::new(),
            work_dir: ".".to_string(),
            xresloader_path: "xresloader.jar".to_string(),
            items: Vec::new(),
            parallelism: 2,
            java_options: Vec::new(),
            java_path: "java".to_string(),
            default_scheme: OrderedMultiMap::default(),
            data_version: None,
            output_matrix: OutputMatrix::default(),
            protocol_files: GroupedInputs::default(),
            data_source_dir: GroupedInputs::default(),
        }
    }
}

/// 对应 Python 的 re.split("\\s+") + 过滤空串，等价于 split_whitespace
pub fn split_by_spaces(value: &str) -> HashSet<String> {
    value.split_whitespace().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_args_keep_insertion_order_and_replace() {
        let mut args = OrderedArgs::default();
        args.set("-p", "pb".to_string());
        args.set("-o", "\"out\"".to_string());
        args.set("-p", "pb2".to_string());
        args.set("-a", "\"1.0\"".to_string());
        let keys: Vec<&str> = args.entries().iter().map(|e| e.0.as_str()).collect();
        assert_eq!(keys, vec!["-p", "-o", "-a"]);
        assert_eq!(args.entries()[0].1, "pb2");
    }

    #[test]
    fn ordered_multi_map_groups_values() {
        let mut map = OrderedMultiMap::default();
        map.push("a", "1".to_string());
        map.push("b", "2".to_string());
        map.push("a", "3".to_string());
        assert!(map.contains_key("a"));
        assert!(!map.contains_key("c"));
        assert_eq!(map.entries()[0].1, vec!["1".to_string(), "3".to_string()]);
    }

    #[test]
    fn split_by_spaces_filters_empty() {
        let set = split_by_spaces("  a  b\tc ");
        assert_eq!(set.len(), 3);
        assert!(set.contains("a") && set.contains("b") && set.contains("c"));
        assert!(split_by_spaces("   ").is_empty());
    }
}
