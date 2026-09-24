#![allow(dead_code)] // 各集成测试 crate 只使用 support 中的部分 helper

/// 查找可用的 Python 3 解释器：优先 python3，回退 python、py。
/// Windows 默认安装可能只有 py 启动器在 PATH，或商店别名 stub 无法运行。
pub fn find_python() -> &'static str {
    for candidate in ["python3", "python", "py"] {
        let ok = std::process::Command::new(candidate)
            .arg("-c")
            .arg("import sys;sys.exit(0 if sys.version_info[0] >= 3 else 1)")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return candidate;
        }
    }
    panic!("Python 3 (python3/python/py) is required for these tests")
}

pub fn fake_java() -> std::path::PathBuf {
    // Filtered cargo test / coverage commands do not necessarily build examples.
    // Keep compiler output under Cargo's ignored target directory; process-lifetime
    // statics do not run TempDir destructors on exit.
    static HELPER: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    HELPER
        .get_or_init(|| {
            let dir = std::path::Path::new(env!("CARGO_BIN_EXE_xresconv-cli"))
                .parent()
                .unwrap()
                .join("test-support")
                .join(std::process::id().to_string());
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join(format!("fake-java{}", std::env::consts::EXE_SUFFIX));
            let output = std::process::Command::new("rustc")
                .args(["--edition=2024", "--crate-name", "fake_java"])
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("tests/support/fake_java.rs"),
                )
                .arg("-o")
                .arg(&path)
                .output()
                .expect("rustc is required for the backend fixture");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            path
        })
        .clone()
}
