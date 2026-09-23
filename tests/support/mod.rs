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
