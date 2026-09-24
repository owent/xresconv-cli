//! Python 兼容入口测试：通过环境变量 XRESCONV_CLI_BIN 指向本地编译的二进制，
//! 验证升级提示、参数转发与退出码兼容行为。
//! Python 3 是这些兼容入口测试的必需依赖；缺失时明确失败。

use std::path::Path;
use std::process::Command;

mod support;
use support::find_python;

fn wrapper(python: &str) -> Command {
    let mut cmd = Command::new(python);
    cmd.arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("xresconv_cli.py"));
    cmd.env("XRESCONV_CLI_BIN", env!("CARGO_BIN_EXE_xresconv-cli"));
    cmd
}

fn exit_code_matches(code: Option<i32>, expected: i32) -> bool {
    match code {
        Some(c) => c == expected || c == (expected as u8 as i32),
        None => false,
    }
}

#[test]
fn python_entry_delegates_version_to_local_binary() {
    let python = find_python();
    let output = wrapper(python).arg("--version").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout.trim(), env!("CARGO_PKG_VERSION"));
    assert!(stderr.contains("[DEPRECATED]"));
}

#[test]
fn python_upgrade_notice_supports_ascii_console() {
    let python = find_python();
    let output = wrapper(python)
        .env("PYTHONIOENCODING", "ascii")
        .arg("--version")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("[DEPRECATED]"));
}

#[test]
fn python_entry_passthrough_args_and_exit_code() {
    let python = find_python();
    // 无参数 -> 打印帮助并以 -1 退出（Unix 上为 255），与直接调用二进制一致
    let output = wrapper(python).output().unwrap();
    assert!(exit_code_matches(output.status.code(), -1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("convert list file"));
}

#[test]
fn python_launcher_offline_unit_tests() {
    let python = find_python();
    let mut cmd = Command::new(python);
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "-m",
            "unittest",
            "discover",
            "-s",
            "tests",
            "-p",
            "test_python_shim.py",
            "-v",
        ])
        .env("XRESCONV_CLI_BIN", env!("CARGO_BIN_EXE_xresconv-cli"));
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("OK"));
}

#[test]
fn all_legacy_entrypoints_forward_to_local_build() {
    let python = find_python();
    for entry in ["xresconv-cli.py", "__main__.py", "xresconv_cli.py", "."] {
        let output = Command::new(python)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args([entry, "--version"])
            .env("XRESCONV_CLI_BIN", env!("CARGO_BIN_EXE_xresconv-cli"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{entry}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            env!("CARGO_PKG_VERSION")
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("[DEPRECATED]"));
    }
}
