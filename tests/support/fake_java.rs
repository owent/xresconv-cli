use std::io::{BufRead, Write};

fn main() {
    if std::env::args().any(|a| a == "--descendant") {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    let args: Vec<String> = std::env::args().collect();
    let option = |key: &str| {
        args.iter()
            .find_map(|a| a.strip_prefix(key).map(str::to_owned))
    };
    let version_probe = args.iter().any(|arg| arg == "--version");
    let probe_mode = option("--fake-probe-mode=").unwrap_or_default();
    if version_probe && probe_mode.is_empty() {
        if let Some(record) = option("--fake-probe-record=") {
            std::fs::write(
                record,
                format!("{:?}\n{:?}", &args[1..], std::env::current_dir().unwrap()),
            )
            .unwrap();
        }
        let verbose = std::env::var_os("FAKE_JAVA_PROBE_VERBOSE").is_some();
        if verbose {
            std::io::stdout()
                .write_all(&vec![b'x'; 256 * 1024])
                .unwrap();
            println!();
            std::io::stderr()
                .write_all(&vec![b'y'; 256 * 1024])
                .unwrap();
            eprintln!();
        }
        println!(
            "{}",
            std::env::var("FAKE_JAVA_VERSION").unwrap_or_else(|_| "2.23.7".into())
        );
        if verbose {
            std::io::stdout()
                .write_all(&vec![b'z'; 256 * 1024])
                .unwrap();
            println!();
        }
        std::process::exit(
            std::env::var("FAKE_JAVA_VERSION_EXIT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
        );
    }
    let mode = if version_probe {
        probe_mode
    } else {
        option("--fake-mode=")
            .unwrap_or_else(|| std::env::var("FAKE_JAVA_MODE").unwrap_or_default())
    };
    if mode == "exit-early" {
        return;
    }
    if mode == "close-stdin" {
        #[cfg(windows)]
        unsafe {
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetStdHandle(kind: u32) -> *mut std::ffi::c_void;
                fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
            }
            CloseHandle(GetStdHandle(-10i32 as u32));
        }
        #[cfg(unix)]
        unsafe {
            unsafe extern "C" {
                fn close(fd: i32) -> i32;
            }
            close(0);
        }
        std::thread::sleep(std::time::Duration::from_secs(30));
        return;
    }
    if mode == "hang" {
        let mut descendant = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--descendant")
            .spawn()
            .unwrap();
        let ready = std::path::PathBuf::from(
            option("--fake-ready=").unwrap_or_else(|| std::env::var("FAKE_JAVA_READY").unwrap()),
        );
        let pending = ready.with_extension("pending");
        std::fs::write(
            &pending,
            format!("{} {}", std::process::id(), descendant.id()),
        )
        .unwrap();
        std::fs::rename(pending, ready).unwrap();
        let _ = descendant.wait();
        return;
    }
    println!("[ARGS] {:?}", std::env::args().skip(1).collect::<Vec<_>>());
    if mode == "large-output" {
        let bytes = vec![b'x'; 256 * 1024];
        std::io::stdout().write_all(&bytes).unwrap();
        println!("\nstdout-large-end 中文");
        std::io::stderr().write_all(&bytes).unwrap();
        eprintln!("\nstderr-large-end 中文");
        std::io::stdout().write_all(b"\xff\xfe\n").unwrap();
    }
    let mut failed = 0;
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        println!("[INFO] fake java got: {line}");
        if line.contains("fake_java_fail") {
            eprintln!("[ERROR] fake java failed: {line}");
            failed += 1;
        }
    }
    if let Ok(code) = std::env::var("FAKE_JAVA_EXIT") {
        failed = code.parse().unwrap();
    }
    std::process::exit(failed.min(255));
}
