use std::{env, fs, io};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=RC_PATH");
    // Build scripts run on the host; resource compilation follows the target OS.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }

    let icon = "assets/icons/xresconv-cli.ico";
    println!("cargo:rerun-if-changed={icon}");
    let bytes = fs::read(icon).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Cannot read {icon}: {error}. Fetch project assets with `git lfs pull`."),
        )
    })?;
    if !bytes.starts_with(&[0, 0, 1, 0]) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{icon} is not an ICO file. Fetch project assets with `git lfs pull`."),
        ));
    }

    winresource::WindowsResource::new()
        .set_icon(icon)
        .set("OriginalFilename", "xresconv-cli.exe")
        .compile()
}
