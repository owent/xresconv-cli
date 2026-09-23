use std::path::Path;
use std::process::{Command, Output};

fn release(args: &[&str], directory: &Path) -> Output {
    Command::new("pwsh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-File",
            "scripts/release.ps1",
        ])
        .args(args)
        .arg("-OutputDirectory")
        .arg(directory)
        .output()
        .expect("PowerShell 7 is required to verify release packaging")
}

#[test]
fn release_tag_must_match_cargo_version() {
    let dir = tempfile::tempdir().unwrap();
    for tag in [
        env!("CARGO_PKG_VERSION").to_owned(),
        format!("v{}", env!("CARGO_PKG_VERSION")),
    ] {
        assert!(
            release(&["-Mode", "Tag", "-Tag", &tag], dir.path())
                .status
                .success()
        );
    }
    assert!(
        !release(&["-Mode", "Tag", "-Tag", "v0.0.0"], dir.path())
            .status
            .success()
    );
}

#[test]
fn package_requires_binary_and_verify_requires_all_core_platforms() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        !release(
            &[
                "-Mode",
                "Package",
                "-Target",
                "x86_64-pc-windows-msvc",
                "-Binary",
                "not-present"
            ],
            dir.path()
        )
        .status
        .success()
    );
    let output = release(&["-Mode", "Verify"], dir.path());
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Missing required asset"));
}

#[test]
fn zip_package_checksum_and_contents() {
    let dir = tempfile::tempdir().unwrap();
    let output = release(
        &[
            "-Mode",
            "Package",
            "-Target",
            "x86_64-pc-windows-msvc",
            "-Binary",
            env!("CARGO_BIN_EXE_xresconv-cli"),
        ],
        dir.path(),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let archive = dir.path().join(format!(
        "xresconv-cli-{}-x86_64-pc-windows-msvc.zip",
        env!("CARGO_PKG_VERSION")
    ));
    assert!(archive.is_file());
    assert!(archive.with_extension("zip.sha256").is_file());
    // Check the digest, then consume the package with the actual shim and run it.
    let script = r#"
import hashlib,sys,zipfile,subprocess
from pathlib import Path
import xresconv_cli as shim
p=Path(sys.argv[1])
assert hashlib.sha256(p.read_bytes()).hexdigest()==Path(str(p)+'.sha256').read_text().split()[0]
with zipfile.ZipFile(p) as z:
    assert sorted(z.namelist())==['LICENSE','README.md','xresconv-cli.exe']
shim.BIN_NAME='xresconv-cli.exe'
shim.cache_dir=lambda: str(p.parent/'isolated cache')
binary=shim.install_binary(p.read_bytes(), '.zip')
assert subprocess.check_output([binary, '--version']).decode().strip()==sys.argv[2]
"#;
    let python = if cfg!(windows) { "python" } else { "python3" };
    let check = Command::new(python)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["-c", script])
        .arg(archive)
        .arg(env!("CARGO_PKG_VERSION"))
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn complete_asset_set_verifies_and_corruption_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let python = if cfg!(windows) { "python" } else { "python3" };
    let script = r#"
import hashlib, sys
from pathlib import Path
root, version = Path(sys.argv[1]), sys.argv[2]
for target in ['x86_64-unknown-linux-gnu', 'x86_64-unknown-linux-musl', 'aarch64-unknown-linux-gnu', 'aarch64-unknown-linux-musl', 'x86_64-apple-darwin', 'aarch64-apple-darwin', 'x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc']:
    ext = '.zip' if '-windows-' in target else '.tar.gz'
    path = root / ('xresconv-cli-' + version + '-' + target + ext)
    path.write_bytes(target.encode())
    Path(str(path) + '.sha256').write_text(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.name + '\n')
"#;
    let output = Command::new(python)
        .args(["-c", script])
        .arg(dir.path())
        .arg(env!("CARGO_PKG_VERSION"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = release(&["-Mode", "Verify"], dir.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "zip"))
        .unwrap();
    std::fs::write(first, "corrupted").unwrap();
    let output = release(&["-Mode", "Verify"], dir.path());
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Checksum mismatch"));
}

#[test]
fn tar_package_is_consumable_by_python_launcher() {
    let dir = tempfile::tempdir().unwrap();
    let output = release(
        &[
            "-Mode",
            "Package",
            "-Target",
            "x86_64-unknown-linux-gnu",
            "-Binary",
            env!("CARGO_BIN_EXE_xresconv-cli"),
        ],
        dir.path(),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let archive = dir.path().join(format!(
        "xresconv-cli-{}-x86_64-unknown-linux-gnu.tar.gz",
        env!("CARGO_PKG_VERSION")
    ));
    let script = r#"
import hashlib, sys
from pathlib import Path
import xresconv_cli as shim
path, cache = Path(sys.argv[1]), sys.argv[2]
assert hashlib.sha256(path.read_bytes()).hexdigest() == Path(str(path)+'.sha256').read_text().split()[0]
shim.cache_dir = lambda: cache
shim.BIN_NAME = 'xresconv-cli'
binary = shim.install_binary(path.read_bytes(), '.tar.gz')
assert Path(binary).read_bytes() == Path(sys.argv[3]).read_bytes()
"#;
    let python = if cfg!(windows) { "python" } else { "python3" };
    let check = Command::new(python)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["-c", script])
        .arg(archive)
        .arg(dir.path().join("cache"))
        .arg(env!("CARGO_BIN_EXE_xresconv-cli"))
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}
