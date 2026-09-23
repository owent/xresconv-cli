//! 真实后端集成测试：使用真实 xresloader jar 与官方 sample 目录执行一次完整转表。
//! 需要环境变量：
//!   XRESCONV_E2E_JAR        xresloader jar 路径（如 xresloader-2.23.7.jar）
//!   XRESCONV_E2E_SAMPLE_DIR xresloader 仓库 sample 目录（含 proto_v3/kind.pb、资源转换示例.xlsx）
//! 默认明确标记 ignored；使用 -- --ignored 执行，缺少环境或样本时失败。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// The backend writes a log in its working directory. Copy only required inputs
// so neither conversions nor direct-Java references modify the external sample.
fn isolated_sample(source: &Path, temporary: &Path) -> PathBuf {
    let sample = temporary.join("sample");
    for name in [
        "资源转换示例.xlsx",
        "custom_validator.yaml",
        "intext-validator.txt",
        "proto_v2/kind.pb",
        "proto_v3/kind.pb",
        "proto_v3/role_cfg.lua",
        "proto_v3/arr_in_arr_cfg.lua",
    ] {
        let target = sample.join(name);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(source.join(name), target).unwrap();
    }
    sample
}

#[test]
#[ignore = "requires XRESCONV_E2E_JAR and XRESCONV_E2E_SAMPLE_DIR"]
fn real_backend_sample_conversion() {
    let (Ok(jar), Ok(sample_dir)) = (
        std::env::var("XRESCONV_E2E_JAR"),
        std::env::var("XRESCONV_E2E_SAMPLE_DIR"),
    ) else {
        panic!("XRESCONV_E2E_JAR / XRESCONV_E2E_SAMPLE_DIR must be set");
    };
    let jar_path = PathBuf::from(&jar);
    let sample_path = PathBuf::from(&sample_dir);
    assert!(jar_path.exists(), "xresloader jar not found: {jar}");
    assert!(
        sample_path.join("proto_v3").join("kind.pb").exists(),
        "sample proto_v3/kind.pb not found in {sample_dir}"
    );

    let dir = tempfile::tempdir().unwrap();
    let sample_path = isolated_sample(&sample_path, dir.path());
    let out_dir = dir.path().join("输出 with spaces");
    fs::create_dir_all(&out_dir).unwrap();

    let xml = format!(
        r#"<root>
  <global>
    <work_dir>{}</work_dir>
    <xresloader_path>{}</xresloader_path>
    <proto>protobuf</proto>
    <output_type rename="/(?i)\.bin$/\.lua/">lua</output_type>
    <output_type rename="/(?i)\.bin$/\.json/">json</output_type>
    <proto_file>proto_v3/kind.pb</proto_file>
    <output_dir>{}</output_dir>
    <data_version>1.0.0.0</data_version>
    <option>--pretty 2</option>
    <option>--validator-rules custom_validator.yaml</option>
    <default_scheme name="KeyRow">2</default_scheme>
    <default_scheme name="MacroSource">资源转换示例.xlsx|macro|2,1</default_scheme>
  </global>
  <list>
    <item file="资源转换示例.xlsx" scheme="scheme_kind"/>
    <item>
      <scheme name="DataSource">资源转换示例.xlsx|arr_in_arr|3,1</scheme>
      <scheme name="ProtoName">arr_in_arr_cfg</scheme>
      <scheme name="OutputFile">arr_in_arr_cfg.bin</scheme>
      <option>--data-source-mapping-mode sha256</option>
      <option>--data-source-mapping-file "{}"</option>
    </item>
  </list>
</root>
"#,
        xml_escape(&sample_path.to_string_lossy()),
        xml_escape(&jar_path.to_string_lossy()),
        xml_escape(&out_dir.to_string_lossy()),
        xml_escape(&out_dir.join("data_source_mapping.txt").to_string_lossy()),
    );
    let conv_list = dir.path().join("conv.xml");
    fs::write(&conv_list, xml).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_xresconv-cli"))
        // The checked-in reference was generated with Chinese locale and
        // Asia/Shanghai time zone. Both affect Excel values and content hashes.
        .args([
            "-p",
            "2",
            "-j",
            "Duser.language=zh",
            "-j",
            "Duser.country=CN",
            "-j",
            "Duser.timezone=Asia/Shanghai",
        ])
        .arg(&conv_list)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "conversion failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("all jobs done. 0 job(s) failed."));

    // 输出文件名与 gen_sample_output.ps1 的真实产物一致
    for name in [
        "role_cfg.lua",
        "role_cfg.json",
        "arr_in_arr_cfg.lua",
        "arr_in_arr_cfg.json",
    ] {
        let path = out_dir.join(name);
        assert!(path.exists(), "missing output: {}", path.display());
        let size = fs::metadata(&path).unwrap().len();
        assert!(size > 0, "empty output: {}", path.display());
    }

    // 与 xresloader sample 中已生成的参考产物对比内容（lua/json 应一致到数据内容层面）
    for (generated, reference) in [
        ("role_cfg.lua", "proto_v3/role_cfg.lua"),
        ("arr_in_arr_cfg.lua", "proto_v3/arr_in_arr_cfg.lua"),
    ] {
        let generated_text = fs::read_to_string(out_dir.join(generated)).unwrap();
        let reference_path = sample_path.join(reference);
        assert!(
            reference_path.exists(),
            "missing required reference: {}",
            reference_path.display()
        );
        let reference_text = fs::read_to_string(reference_path).unwrap();
        assert_eq!(
            normalize(&generated_text),
            normalize(&reference_text),
            "content mismatch vs {reference}"
        );
    }
}

/// 数据内容比较：忽略缩进/换行差异与 xres_ver（参考产物可能由不同的后端版本生成）
fn normalize(text: &str) -> String {
    let version = regex::Regex::new(r#"xres_ver = "[^"]*""#).unwrap();
    let canonical = text.replace("\r\n", "\n");
    let masked = version.replace_all(&canonical, "xres_ver = \"\"");
    // Preserve whitespace inside data strings. Only leading indentation varies.
    masked
        .lines()
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn reference_normalization_accepts_future_backend_versions_without_losing_data_whitespace() {
    assert_eq!(
        normalize("  xres_ver = \"2.99.1\"\r\n  value = \"a  b\"\r\n"),
        "xres_ver = \"\"\nvalue = \"a  b\""
    );
}

#[test]
#[ignore = "requires XRESCONV_E2E_JAR and XRESCONV_E2E_SAMPLE_DIR"]
fn real_backend_missing_jar_is_error() {
    let (Ok(_), Ok(sample_dir)) = (
        std::env::var("XRESCONV_E2E_JAR"),
        std::env::var("XRESCONV_E2E_SAMPLE_DIR"),
    ) else {
        panic!("XRESCONV_E2E_JAR / XRESCONV_E2E_SAMPLE_DIR must be set");
    };
    let dir = tempfile::tempdir().unwrap();
    let xml = format!(
        r#"<root>
  <global>
    <work_dir>{}</work_dir>
    <xresloader_path>no-such-xresloader.jar</xresloader_path>
  </global>
  <list><item file="资源转换示例.xlsx" scheme="scheme_kind"/></list>
</root>
"#,
        xml_escape(&Path::new(&sample_dir).to_string_lossy()),
    );
    let conv_list = dir.path().join("conv.xml");
    fs::write(&conv_list, xml).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_xresconv-cli"))
        .arg(&conv_list)
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    // 退出码 -4（Unix 上为 252）
    assert!(code == -4 || code == 252, "unexpected exit code: {code}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("xresloader not found"));
}

fn compare_to_direct_java(proto_dir: &str) {
    let jar = PathBuf::from(std::env::var("XRESCONV_E2E_JAR").expect("XRESCONV_E2E_JAR required"));
    let sample = PathBuf::from(
        std::env::var("XRESCONV_E2E_SAMPLE_DIR").expect("XRESCONV_E2E_SAMPLE_DIR required"),
    );
    assert!(jar.is_file());
    assert!(sample.join(proto_dir).join("kind.pb").is_file());
    let dir = tempfile::tempdir().unwrap();
    let sample = isolated_sample(&sample, dir.path());
    let formats = [
        ("bin", "bin"),
        ("lua", "lua"),
        ("json", "json"),
        ("xml", "xml"),
        ("msgpack", "msgpack.bin"),
        ("js", "js"),
    ];
    let mut xml = format!(
        "<root><global><work_dir>{}</work_dir><xresloader_path>{}</xresloader_path><proto>protobuf</proto><proto_file>{proto_dir}/kind.pb</proto_file><data_version>1.0.0.0</data_version><option>--pretty 2</option><option>--validator-rules custom_validator.yaml</option><default_scheme name='KeyRow'>2</default_scheme><default_scheme name='MacroSource'>资源转换示例.xlsx|macro|2,1</default_scheme>",
        xml_escape(&sample.to_string_lossy()),
        xml_escape(&jar.to_string_lossy())
    );
    for (format, extension) in formats {
        let out = dir.path().join(format!("Rust 输出 {format}"));
        fs::create_dir(&out).unwrap();
        xml.push_str(&format!(r#"<output_type output_dir="{}" rename="/(?i)\.bin$/\.{extension}/">{format}</output_type>"#, xml_escape(&out.to_string_lossy())));
    }
    xml.push_str("</global><list><item file='资源转换示例.xlsx' scheme='scheme_kind'/><item><scheme name='DataSource'>资源转换示例.xlsx|arr_in_arr|3,1</scheme><scheme name='ProtoName'>arr_in_arr_cfg</scheme><scheme name='OutputFile'>arr_in_arr_cfg.bin</scheme></item></list></root>");
    let list = dir.path().join("转换.xml");
    fs::write(&list, xml).unwrap();
    // This runs the legacy entrypoint with the local Cargo binary, never a download.
    let python = if cfg!(windows) { "python" } else { "python3" };
    let output = Command::new(python)
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("xresconv-cli.py"))
        .env("XRESCONV_CLI_BIN", env!("CARGO_BIN_EXE_xresconv-cli"))
        .args(["-p", "2", "-j", "Dxresloader.version=dev"])
        .arg(list)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let java = std::env::var_os("JAVA_HOME")
        .map(|home| {
            PathBuf::from(home)
                .join("bin")
                .join(if cfg!(windows) { "java.exe" } else { "java" })
        })
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from("java"));
    for (format, extension) in formats {
        let reference = dir.path().join(format!("Java reference {format}"));
        fs::create_dir(&reference).unwrap();
        for inline in [false, true] {
            // Use real argv, independently of the Rust stdin formatter.
            let mut command = Command::new(&java);
            command
                .current_dir(&sample)
                .args(["-Dfile.encoding=utf-8", "-Dxresloader.version=dev", "-jar"])
                .arg(&jar)
                .args([
                    "-p",
                    "protobuf",
                    "-f",
                    &format!("{proto_dir}/kind.pb"),
                    "--pretty",
                    "2",
                    "--validator-rules",
                    "custom_validator.yaml",
                    "-a",
                    "1.0.0.0",
                    "-t",
                    format,
                    "-n",
                    &format!(r"/(?i)\.bin$/\.{extension}/"),
                    "-o",
                ])
                .arg(&reference);
            if inline {
                command.args([
                    "-m",
                    "DataSource=资源转换示例.xlsx|arr_in_arr|3,1",
                    "-m",
                    "ProtoName=arr_in_arr_cfg",
                    "-m",
                    "OutputFile=arr_in_arr_cfg.bin",
                    "-m",
                    "KeyRow=2",
                    "-m",
                    "MacroSource=资源转换示例.xlsx|macro|2,1",
                ]);
            } else {
                command.args(["-s", "资源转换示例.xlsx", "-m", "scheme_kind"]);
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "direct Java failed: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let name = format!(
                "{}.{extension}",
                if inline { "arr_in_arr_cfg" } else { "role_cfg" }
            );
            let expected = fs::read(reference.join(&name)).unwrap();
            let actual =
                fs::read(dir.path().join(format!("Rust 输出 {format}")).join(&name)).unwrap();
            assert!(!actual.is_empty());
            assert_eq!(
                actual, expected,
                "{proto_dir}/{name} differs from direct Java"
            );
        }
    }
}

#[test]
#[ignore = "requires fixed xresloader jar and sample"]
fn proto2_six_formats_match_direct_java() {
    compare_to_direct_java("proto_v2");
}

#[test]
#[ignore = "requires fixed xresloader jar and sample"]
fn proto3_six_formats_match_direct_java() {
    compare_to_direct_java("proto_v3");
}

#[test]
#[ignore = "requires fixed xresloader jar and sample"]
fn real_java_backend_failure_propagates() {
    let jar = std::env::var("XRESCONV_E2E_JAR").expect("jar required");
    let sample = std::env::var("XRESCONV_E2E_SAMPLE_DIR").expect("sample required");
    let dir = tempfile::tempdir().unwrap();
    let sample = isolated_sample(Path::new(&sample), dir.path());
    let xml = format!(
        "<root><global><work_dir>{}</work_dir><xresloader_path>{}</xresloader_path><proto>protobuf</proto><proto_file>proto_v3/kind.pb</proto_file><output_dir>{}</output_dir></global><list><item><scheme name='ProtoName'>NonExistentMessage</scheme><scheme name='DataSource'>资源转换示例.xlsx|kind|3,1</scheme></item></list></root>",
        xml_escape(&sample.to_string_lossy()),
        xml_escape(&jar),
        xml_escape(&dir.path().to_string_lossy())
    );
    let list = dir.path().join("invalid.xml");
    fs::write(&list, xml).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xresconv-cli"))
        .arg(list)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("all jobs done. 0 job(s) failed."));
}
