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
    let mode = option("--fake-mode=")
        .unwrap_or_else(|| std::env::var("FAKE_JAVA_MODE").unwrap_or_default());
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
        std::fs::write(
            option("--fake-ready=").unwrap_or_else(|| std::env::var("FAKE_JAVA_READY").unwrap()),
            format!("{} {}", std::process::id(), descendant.id()),
        )
        .unwrap();
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
