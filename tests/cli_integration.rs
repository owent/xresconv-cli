mod support;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xresconv-cli"))
}

/// 兼容退出码：Unix 下负数退出码被截断为 8 位（-2 -> 254），Windows 保留原值
fn exit_code_matches(code: Option<i32>, expected: i32) -> bool {
    match code {
        Some(c) => c == expected || c == (expected as u8 as i32),
        None => false,
    }
}

fn write_fixture(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

const BASIC_XML: &str = r#"<root>
  <global>
    <proto>pb</proto>
    <output_dir>out</output_dir>
  </global>
  <list>
    <item file="a.xlsx" scheme="sa"/>
    <item file="b.xlsx" scheme="sb"/>
  </list>
</root>"#;

fn setup_workdir(xml: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let list = write_fixture(dir.path(), "list.xml", xml);
    // xresloader.jar 存在性检查：放一个空文件
    write_fixture(dir.path(), "xresloader.jar", "");
    (dir, list)
}

#[test]
fn version_flag_prints_version() {
    let output = bin().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), env!("CARGO_PKG_VERSION"));

    let output = bin().arg("-v").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn no_arguments_shows_help_and_fails() {
    // 与原 Python 版 print_help_msg(-1) 分支一致：打印帮助并退出 -1
    let output = bin().output().unwrap();
    assert!(exit_code_matches(output.status.code(), -1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("convert list file"));
}

#[test]
fn missing_convert_list_file_exits_minus_2() {
    let dir = tempfile::tempdir().unwrap();
    let output = bin().arg(dir.path().join("missing.xml")).output().unwrap();
    assert!(exit_code_matches(output.status.code(), -2));
}

#[test]
fn missing_xresloader_exits_minus_4() {
    let dir = tempfile::tempdir().unwrap();
    let list = write_fixture(dir.path(), "list.xml", BASIC_XML);
    // 不放 xresloader.jar
    let output = bin().arg(&list).output().unwrap();
    assert!(exit_code_matches(output.status.code(), -4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("xresloader not found"));
}

#[test]
fn test_mode_prints_commands_without_running_java() {
    let (dir, list) = setup_workdir(BASIC_XML);
    // java_path 指向不存在的路径；test 模式不应启动任何进程
    let output = bin()
        .arg("--test")
        .arg("-J")
        .arg(dir.path().join("no-such-java"))
        .arg(&list)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 打印 java 命令头与每条转换命令
    assert!(stdout.contains("-jar"));
    assert!(stdout.contains("--stdin"));
    assert!(stdout.contains("-p pb"));
    assert!(stdout.contains("-s \"a.xlsx\" -m \"sa\""));
    assert!(stdout.contains("-s \"b.xlsx\" -m \"sb\""));
    assert!(stdout.contains("all jobs done. 0 job(s) failed."));
}

#[test]
fn scheme_name_filter_limits_items() {
    let (_dir, list) = setup_workdir(BASIC_XML);
    let output = bin()
        .arg("--test")
        .arg("-s")
        .arg("sa")
        .arg(&list)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"a.xlsx\""));
    assert!(!stdout.contains("\"b.xlsx\""));
}

#[test]
fn real_run_with_fake_java_backend() {
    let (_dir, list) = setup_workdir(BASIC_XML);
    let java = support::fake_java();
    let output = bin().arg("-J").arg(java).arg(&list).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fake java got:"));
    assert!(stdout.contains("\"a.xlsx\""));
    assert!(stdout.contains("\"b.xlsx\""));
    assert!(stdout.contains("all jobs done. 0 job(s) failed."));
}

#[test]
fn real_run_aggregates_child_exit_codes() {
    let xml = r#"<root>
  <list>
    <item file="a.xlsx" scheme="fake_java_fail"/>
    <item file="b.xlsx" scheme="sb"/>
  </list>
</root>"#;
    let (_dir, list) = setup_workdir(xml);
    let java = support::fake_java();
    let output = bin()
        .arg("-J")
        .arg(java)
        .arg("-p")
        .arg("1")
        .arg(&list)
        .output()
        .unwrap();
    // fake java 对包含 fake_java_fail 的行计 1 次失败
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("all jobs done. 1 job(s) failed."));
}

#[test]
fn trailing_args_after_double_dash_pass_through() {
    let (_dir, list) = setup_workdir(BASIC_XML);
    let output = bin()
        .arg("--test")
        .arg(&list)
        .arg("--")
        .arg("--custom-xresloader-arg")
        .arg("value")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--custom-xresloader-arg value"));
}

#[test]
fn data_version_overrides_and_quotes() {
    let xml = r#"<root>
  <global><data_version>from_xml</data_version></global>
  <list><item file="a.xlsx" scheme="sa"/></list>
</root>"#;
    let (_dir, list) = setup_workdir(xml);
    let output = bin()
        .arg("--test")
        .arg("-a")
        .arg("from_cli")
        .arg(&list)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-a \"from_cli\""));
    assert!(!stdout.contains("from_xml"));
}

#[test]
fn work_dir_and_xresloader_path_from_xml() {
    let dir = tempfile::tempdir().unwrap();
    let xml = r#"<root>
  <global>
    <work_dir>sub</work_dir>
    <xresloader_path>backend/xresloader.jar</xresloader_path>
  </global>
  <list><item file="a.xlsx" scheme="sa"/></list>
</root>"#;
    let list = write_fixture(dir.path(), "list.xml", xml);
    write_fixture(dir.path(), "sub/backend/xresloader.jar", "");
    let output = bin().arg("--test").arg(&list).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("backend/xresloader.jar") || stdout.contains("backend\\xresloader.jar")
    );
}

#[test]
fn broken_xml_exits_minus_2() {
    let dir = tempfile::tempdir().unwrap();
    let list = write_fixture(dir.path(), "bad.xml", "<root><global></root>");
    let output = bin().arg(&list).output().unwrap();
    assert!(exit_code_matches(output.status.code(), -2));
}

#[test]
fn explicit_java_path_failure_is_not_replaced_by_java_home() {
    let (dir, list) = setup_workdir(BASIC_XML);
    let output = bin()
        .arg("-J")
        .arg(dir.path().join("absent.exe"))
        .arg(list)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to start java"));
}

#[test]
fn relative_java_path_is_resolved_before_work_dir_changes() {
    let dir = tempfile::tempdir().unwrap();
    let java_name = format!("java helper{}", std::env::consts::EXE_SUFFIX);
    fs::copy(support::fake_java(), dir.path().join(&java_name)).unwrap();
    let list = write_fixture(dir.path(), "config/list.xml", BASIC_XML);
    write_fixture(dir.path(), "config/xresloader.jar", "");
    let output = bin()
        .current_dir(dir.path())
        .arg("-J")
        .arg(&java_name)
        .arg(list)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("fake java got:"));
}

#[test]
fn java_home_and_cli_jvm_arguments_are_usable() {
    let (dir, list) = setup_workdir(
        "<root><global><java_option>-Xms16m</java_option></global><list><item file='a.xlsx' scheme='s'/></list></root>",
    );
    fs::create_dir(dir.path().join("bin")).unwrap();
    fs::copy(
        support::fake_java(),
        dir.path()
            .join("bin")
            .join(format!("java{}", std::env::consts::EXE_SUFFIX)),
    )
    .unwrap();
    let output = bin()
        .env("JAVA_HOME", dir.path())
        .args(["-j", "Xmx512m", "-p", "1"])
        .arg(list)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("-Xmx512m").count(), 1);
    assert_eq!(stdout.matches("Xmx512m").count(), 1);
    assert_eq!(stdout.matches("-Xms16m").count(), 1);
}

#[test]
fn invalid_work_dir_is_diagnostic() {
    let (_dir, list) = setup_workdir("<root><global><work_dir>missing</work_dir></global></root>");
    let output = bin().arg(list).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("work dir not found"));
}

#[test]
fn aggregated_failure_code_never_wraps_to_success() {
    let (_dir, list) = setup_workdir(BASIC_XML);
    let output = bin()
        .env("FAKE_JAVA_EXIT", "128")
        .arg("-J")
        .arg(support::fake_java())
        .arg(list)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(255));
}

#[test]
fn color_modes_are_resolved_in_isolated_processes() {
    let (_dir, list) = setup_workdir(BASIC_XML);
    for (mode, enabled) in [
        ("term", true),
        ("win32_console", true),
        ("none", false),
        ("invalid", false),
    ] {
        let output = bin()
            .env("CPRINTF_MODE", mode)
            .arg("--test")
            .arg(&list)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.contains(&0x1b), enabled);
    }
    for (term, enabled) in [
        ("dumb", false),
        ("dump", false),
        ("xterm-256color", true),
        ("vt100", true),
    ] {
        let output = bin()
            .env_remove("CPRINTF_MODE")
            .env("TERM", term)
            .arg("--test")
            .arg(&list)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.contains(&0x1b), enabled);
    }
}
