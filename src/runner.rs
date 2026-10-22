use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use regex::Regex;

use crate::color::{Color, cprint_stdout_block, cprintln_stderr};
use crate::options::ConvOptions;

fn log_rules() -> &'static (Regex, Regex, Regex) {
    static RULES: OnceLock<(Regex, Regex, Regex)> = OnceLock::new();
    RULES.get_or_init(|| {
        let prefix = r"^(?:\x1b\[[0-?]*[ -/]*[@-~])*[\x00-\x1f\x7f]*\s*\[?\s*";
        let build = |body: &str| {
            regex::RegexBuilder::new(&format!("{prefix}{body}"))
                .case_insensitive(true)
                .build()
                .unwrap()
        };
        (build(r"error\b"), build(r"warn(ing)?\b"), build(r"info\b"))
    })
}

fn classify_line(line: &str, is_stderr: bool, previous: Option<Color>) -> Option<Color> {
    let (error_rule, warn_rule, info_rule) = log_rules();
    if error_rule.is_match(line) {
        Some(Color::Red)
    } else if warn_rule.is_match(line) {
        Some(Color::Yellow)
    } else if info_rule.is_match(line) {
        Some(Color::Green)
    } else if is_stderr && previous.is_none() {
        Some(Color::Cyan)
    } else {
        previous
    }
}

fn pump_lines<R: Read>(reader: R, is_stderr: bool) -> std::io::Result<()> {
    let mut reader = BufReader::new(reader);
    let mut previous: Option<Color> = None;
    let mut buf: Vec<u8> = Vec::new();
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => break,
            Ok(_) => {
                let content = String::from_utf8_lossy(&buf);
                let line = content.trim_end_matches(['\r', '\n']);
                let color = classify_line(line, is_stderr, previous);
                previous = color;
                if is_stderr {
                    cprintln_stderr(color, line);
                } else {
                    crate::color::cprintln_stdout(color, line);
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn java_launch_prefix(opts: &ConvOptions, cli_java_options: &[String]) -> Vec<String> {
    let mut java_options: Vec<String> = vec![opts.java_path.clone()];
    for java_option in cli_java_options {
        java_options.push(format!("-{java_option}"));
    }
    for java_option in &opts.java_options {
        java_options.push(java_option.clone());
    }
    java_options.push("-Dfile.encoding=utf-8".to_string());
    java_options.push("-jar".to_string());
    java_options.push(opts.xresloader_path.clone());
    java_options
}

/// JVM arguments, then backend-wide stdin/concurrency options (never per-task options).
pub fn java_command_prefix(opts: &ConvOptions, cli_java_options: &[String]) -> Vec<String> {
    let mut java_options = java_launch_prefix(opts, cli_java_options);
    java_options.push("--stdin".to_string());
    if let Some(parallelism) = opts.builtin_parallelism {
        java_options.extend(["--parallelism".to_string(), parallelism.to_string()]);
    }
    java_options
}

fn supports_builtin_parallelism(output: &[u8]) -> Option<bool> {
    static VERSION: OnceLock<Regex> = OnceLock::new();
    let pattern = VERSION
        .get_or_init(|| Regex::new(r"^v?(\d+)\.(\d+)(?:\.\d+)?(?:[-+][0-9A-Za-z.+-]+)?$").unwrap());
    String::from_utf8_lossy(output).lines().find_map(|line| {
        let version = pattern.captures(line.trim())?;
        let major = version[1].parse::<u64>().ok()?;
        let minor = version[2].parse::<u64>().ok()?;
        Some((major, minor) >= (2, 24))
    })
}

fn scan_probe_version(mut reader: impl Read) -> std::io::Result<Option<bool>> {
    // Scan the entire stream without retaining startup logs. Never parse a partial
    // oversized line: its prefix could look like a valid version by accident.
    const LINE_LIMIT: usize = 1024;
    let mut line = Vec::new();
    let mut oversized = false;
    let mut detected = None;
    let mut buffer = [0; 4096];
    loop {
        let count = match reader.read(&mut buffer) {
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 {
            if !oversized {
                detected = supports_builtin_parallelism(&line).or(detected);
            }
            return Ok(detected);
        }
        for &byte in &buffer[..count] {
            if byte == b'\n' {
                if !oversized {
                    detected = supports_builtin_parallelism(&line).or(detected);
                }
                line.clear();
                oversized = false;
            } else if line.len() < LINE_LIMIT {
                line.push(byte);
            } else {
                oversized = true;
            }
        }
    }
}

/// Probe the configured backend, with the same JVM options, cwd and process-tree ownership.
pub fn detect_builtin_parallelism(
    opts: &ConvOptions,
    cli_java_options: &[String],
    work_dir: &Path,
    cancelled: &AtomicBool,
    timeout: Duration,
) -> Result<bool, String> {
    if cancelled.load(Ordering::SeqCst) {
        return Err("version probe cancelled".to_string());
    }
    let prefix = java_launch_prefix(opts, cli_java_options);
    let mut tree = crate::process::ChildTree::spawn(
        Command::new(&prefix[0])
            .args(&prefix[1..])
            .arg("--version")
            .current_dir(work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .map_err(|err| format!("version probe failed to start java: {err}"))?;
    std::thread::scope(|scope| {
        let stdout = tree.child.stdout.take().expect("piped stdout");
        let mut stderr = tree.child.stderr.take().expect("piped stderr");
        let stdout_reader = scope.spawn(move || scan_probe_version(stdout));
        let stderr_reader = scope.spawn(move || std::io::copy(&mut stderr, &mut std::io::sink()));
        let started = Instant::now();
        let status = loop {
            if cancelled.load(Ordering::SeqCst) {
                break Err("version probe cancelled".to_string());
            }
            if started.elapsed() >= timeout {
                break Err("version probe timed out".to_string());
            }
            match tree.child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                Err(err) => break Err(format!("version probe wait failed: {err}")),
            }
        };
        tree.kill();
        let _ = tree.child.wait();
        // Always join both readers, including failure/cancellation paths.
        let stdout = stdout_reader.join();
        let stderr = stderr_reader.join();
        let status = status?;
        if !status.success() {
            return Err(format!("version probe exited with {status}"));
        }
        let stdout = stdout
            .map_err(|_| "version probe stdout reader panicked".to_string())?
            .map_err(|err| format!("version probe stdout failed: {err}"))?;
        stderr
            .map_err(|_| "version probe stderr reader panicked".to_string())?
            .map_err(|err| format!("version probe stderr failed: {err}"))?;
        stdout
            .ok_or_else(|| "version probe returned an unrecognized xresloader version".to_string())
    })
}

fn feed_commands(
    mut stdin: impl Write,
    cmd_stack: &Mutex<Vec<Vec<String>>>,
    once_pick_count: usize,
    cancelled: &AtomicBool,
) -> std::io::Result<()> {
    loop {
        if cancelled.load(Ordering::SeqCst) {
            break;
        }
        let batch: Vec<Vec<String>> = {
            let mut stack = cmd_stack.lock().unwrap_or_else(|e| e.into_inner());
            if stack.is_empty() {
                drop(stack);
                break;
            }
            let mut picked = Vec::new();
            for _ in 0..once_pick_count.max(1) {
                match stack.pop() {
                    Some(cmd) => picked.push(cmd),
                    None => break,
                }
            }
            picked
        };
        if batch.is_empty() {
            break;
        }
        for cmd in batch {
            let line = cmd.join(" ");
            stdin.write_all(line.as_bytes())?;
            stdin.write_all(b"\n")?;
        }
        stdin.flush()?;
    }
    Ok(())
}

fn worker_run_real(
    opts: &ConvOptions,
    cli_java_options: &[String],
    work_dir: &Path,
    cmd_stack: &Mutex<Vec<Vec<String>>>,
    once_pick_count: usize,
    worker_idx: usize,
    cancelled: &AtomicBool,
) -> i32 {
    if cancelled.load(Ordering::SeqCst) {
        return 0;
    }
    let java_options = java_command_prefix(opts, cli_java_options);
    let spawn_result = crate::process::ChildTree::spawn(
        Command::new(&java_options[0])
            .args(&java_options[1..])
            .current_dir(work_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    );

    let mut tree = match spawn_result {
        Ok(c) => c,
        Err(err) => {
            // 与 Python 版差异：Python 线程直接崩溃且不累计失败数；这里累计失败并输出诊断
            cprintln_stderr(
                Some(Color::Red),
                &format!("[ERROR] worker thread {worker_idx} failed to start java: {err}"),
            );
            return 1;
        }
    };

    let write_failed = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let stdin = tree.child.stdin.take().expect("piped stdin");
        let stdout = tree.child.stdout.take().expect("piped stdout");
        let stderr = tree.child.stderr.take().expect("piped stderr");
        let write_failed_ref = &write_failed;
        let writer = scope.spawn(move || {
            let result = feed_commands(stdin, cmd_stack, once_pick_count, cancelled);
            if result.is_err() {
                write_failed_ref.store(true, Ordering::Release);
            }
            result
        });
        let stdout_reader = scope.spawn(move || pump_lines(stdout, false));
        let stderr_reader = scope.spawn(move || pump_lines(stderr, true));
        let mut code = loop {
            if cancelled.load(Ordering::SeqCst) {
                break 1;
            }
            // A backend may close stdin and keep running. Stop it immediately;
            // the writer's error below supplies the nonzero failure status.
            if write_failed.load(Ordering::Acquire) {
                break 0;
            }
            match tree.child.try_wait() {
                Ok(Some(status)) => {
                    break if status.success() {
                        0
                    } else {
                        status.code().unwrap_or(1).max(1)
                    };
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                Err(err) => {
                    cprintln_stderr(
                        Some(Color::Red),
                        &format!("[ERROR] worker {worker_idx} wait failed: {err}"),
                    );
                    break 1;
                }
            }
        };
        // Also close pipes inherited by descendants before joining reader/writer threads.
        tree.kill();
        let _ = tree.child.wait();
        for result in [writer.join(), stdout_reader.join(), stderr_reader.join()] {
            if !matches!(result, Ok(Ok(()))) {
                cprintln_stderr(
                    Some(Color::Red),
                    &format!("[ERROR] worker {worker_idx} backend pipe failed"),
                );
                code = code.saturating_add(1);
            }
        }
        code
    })
}

fn worker_run_test(
    opts: &ConvOptions,
    cli_java_options: &[String],
    cmd_stack: &Mutex<Vec<Vec<String>>>,
    once_pick_count: usize,
) {
    let java_options = java_command_prefix(opts, cli_java_options);
    let mut this_thd_cmds: Vec<String> = Vec::new();
    loop {
        let mut stack = cmd_stack.lock().unwrap_or_else(|e| e.into_inner());
        if stack.is_empty() {
            break;
        }
        for _ in 0..once_pick_count.max(1) {
            match stack.pop() {
                Some(cmd) => this_thd_cmds.push(cmd.join(" ")),
                None => break,
            }
        }
    }
    let header = java_options
        .iter()
        .map(|a| format!("\"{a}\""))
        .collect::<Vec<_>>()
        .join(" ");
    let body = this_thd_cmds
        .iter()
        .map(|c| format!("\t> {c}"))
        .collect::<Vec<_>>()
        .join("\n");
    cprint_stdout_block(Some(Color::Green), &format!("{header}\n{body}\n"));
}

pub fn run_workers(
    opts: &ConvOptions,
    cli_java_options: &[String],
    test_mode: bool,
    work_dir: &Path,
    mut cmd_list: Vec<Vec<String>>,
    parallelism: i64,
    cancelled: &AtomicBool,
) -> i32 {
    if parallelism <= 0 {
        return 1;
    }
    if cmd_list.is_empty() {
        return 0;
    }
    // build_commands 返回正序命令；反转后作为栈弹出即恢复正序（等价 Python 的 reverse + pop）
    cmd_list.reverse();
    let once_pick_count = opts.output_matrix.outputs.len().max(1);
    let cmd_stack = Mutex::new(cmd_list);
    let failed_count = Mutex::new(0i32);

    let process_limit = if opts.builtin_parallelism.is_some() {
        1
    } else {
        parallelism
    };
    let worker_count = usize::try_from(process_limit)
        .unwrap_or(usize::MAX)
        .min(cmd_stack.lock().unwrap().len().div_ceil(once_pick_count));
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for idx in 0..worker_count {
            let cmd_stack_ref = &cmd_stack;
            let handle = std::thread::Builder::new()
                .name(format!("converter-{idx}"))
                .spawn_scoped(scope, move || {
                    if test_mode {
                        worker_run_test(opts, cli_java_options, cmd_stack_ref, once_pick_count);
                        0
                    } else {
                        worker_run_real(
                            opts,
                            cli_java_options,
                            work_dir,
                            cmd_stack_ref,
                            once_pick_count,
                            idx,
                            cancelled,
                        )
                    }
                });
            match handle {
                Ok(handle) => handles.push(handle),
                Err(err) => {
                    cprintln_stderr(
                        Some(Color::Red),
                        &format!("[ERROR] cannot start worker: {err}"),
                    );
                    *failed_count.lock().unwrap() += 1;
                    break;
                }
            }
        }
        for handle in handles {
            match handle.join() {
                Ok(code) => {
                    let mut failed = failed_count.lock().unwrap_or_else(|e| e.into_inner());
                    *failed = failed.saturating_add(code);
                }
                Err(_) => {
                    let mut failed = failed_count.lock().unwrap_or_else(|e| e.into_inner());
                    *failed += 1;
                }
            }
        }
    });

    let remaining = cmd_stack.lock().unwrap_or_else(|e| e.into_inner()).len();
    if remaining > 0 && !cancelled.load(Ordering::SeqCst) {
        cprintln_stderr(
            Some(Color::Red),
            &format!("[ERROR] {remaining} command(s) were not sent to the backend"),
        );
        let mut failed = failed_count.lock().unwrap();
        *failed = failed.saturating_add(1);
    }
    *failed_count.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_version_boundary_includes_prereleases_but_not_unrelated_jvm_logs() {
        for (version, builtin) in [
            ("2.23.99", false),
            ("2.24", true),
            ("2.24.0-beta", true),
            ("v2.24.0-beta.1+build", true),
            ("2.100.0", true),
            ("3.0.0", true),
            ("1.99.99", false),
        ] {
            assert_eq!(
                supports_builtin_parallelism(version.as_bytes()),
                Some(builtin)
            );
        }
        for version in ["Unknown", "java version 21.0.1", "", "2", "2.24.invalid"] {
            assert_eq!(supports_builtin_parallelism(version.as_bytes()), None);
        }
        assert_eq!(
            supports_builtin_parallelism(b"[INFO] startup\r\n2.24.0-beta\r\n"),
            Some(true)
        );
    }

    #[test]
    fn probe_ignores_oversized_lines_and_still_fully_drains() {
        let bytes = vec![b'x'; 256 * 1024];
        let mut input = std::io::Cursor::new(&bytes);
        assert_eq!(scan_probe_version(&mut input).unwrap(), None);
        assert_eq!(input.position(), bytes.len() as u64);
    }

    #[test]
    fn probe_recognizes_version_after_large_startup_logs() {
        let mut bytes = vec![b'x'; 256 * 1024];
        bytes.extend_from_slice(b"\n2.24.0-beta\n");
        assert_eq!(scan_probe_version(&bytes[..]).unwrap(), Some(true));
    }

    #[test]
    fn probe_does_not_parse_a_truncated_line_as_a_version() {
        let mut bytes = vec![b'x'; 64 * 1024 - 5];
        bytes.extend_from_slice(b"\n2.24.invalid\n");
        assert_eq!(scan_probe_version(&bytes[..]).unwrap(), None);
    }

    #[test]
    fn probe_handles_short_reads_interruption_and_eof_without_a_newline() {
        struct ShortReader {
            bytes: std::io::Cursor<Vec<u8>>,
            interrupted: bool,
        }
        impl Read for ShortReader {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                if !self.interrupted {
                    self.interrupted = true;
                    return Err(std::io::ErrorKind::Interrupted.into());
                }
                self.bytes.read(&mut buffer[..1])
            }
        }
        let mut bytes = b"2.24.0-".to_vec();
        bytes.extend(std::iter::repeat_n(b'a', 2048));
        assert_eq!(scan_probe_version(&bytes[..]).unwrap(), None);
        bytes.extend_from_slice(b"\ninvalid UTF-8: \xff\xfe\r\n 2.23.7 ");
        assert_eq!(
            scan_probe_version(ShortReader {
                bytes: std::io::Cursor::new(bytes),
                interrupted: false,
            })
            .unwrap(),
            Some(false)
        );
    }

    #[test]
    fn io_errors_and_cancellation_are_returned() {
        struct BrokenPipe;
        impl Write for BrokenPipe {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        impl Read for BrokenPipe {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::Other.into())
            }
        }
        assert!(pump_lines(BrokenPipe, false).is_err());
        assert!(scan_probe_version(BrokenPipe).is_err());
        let stack = Mutex::new(vec![vec!["command".to_string()]]);
        let cancel = AtomicBool::new(false);
        assert!(feed_commands(BrokenPipe, &stack, 1, &cancel).is_err());
        let stack = Mutex::new(vec![vec!["not sent".to_string()]]);
        cancel.store(true, Ordering::SeqCst);
        let mut bytes = Vec::new();
        feed_commands(&mut bytes, &stack, 1, &cancel).unwrap();
        assert!(bytes.is_empty());
        assert_eq!(stack.lock().unwrap().len(), 1);
        let opts = ConvOptions::default();
        assert_eq!(
            run_workers(&opts, &[], true, Path::new("."), vec![], 0, &cancel),
            1
        );
    }

    #[test]
    fn classify_log_lines() {
        assert_eq!(classify_line("[ERROR] boom", false, None), Some(Color::Red));
        assert_eq!(
            classify_line("[warn] slow", false, None),
            Some(Color::Yellow)
        );
        assert_eq!(
            classify_line("warning: x", false, None),
            Some(Color::Yellow)
        );
        assert_eq!(classify_line("[INFO] ok", false, None), Some(Color::Green));
        assert_eq!(classify_line("plain stdout", false, None), None);
        assert_eq!(classify_line("plain stderr", true, None), Some(Color::Cyan));
        // 已着色行延续之前颜色
        assert_eq!(
            classify_line("plain stdout", false, Some(Color::Red)),
            Some(Color::Red)
        );
        // 带 ANSI 前缀的日志
        assert_eq!(
            classify_line("\x1b[0m[ERROR] x", false, None),
            Some(Color::Red)
        );
    }

    #[test]
    fn java_prefix_order() {
        let mut opts = ConvOptions::new();
        opts.java_path = "/usr/bin/java".to_string();
        opts.xresloader_path = "xresloader.jar".to_string();
        opts.java_options = vec!["-Xmx512m".to_string()];
        let prefix = java_command_prefix(&opts, &["Xmx2048m".to_string()]);
        assert_eq!(
            prefix,
            vec![
                "/usr/bin/java",
                "-Xmx2048m",
                "-Xmx512m",
                "-Dfile.encoding=utf-8",
                "-jar",
                "xresloader.jar",
                "--stdin"
            ]
        );
    }

    #[test]
    fn test_mode_collects_all_commands_without_spawning() {
        let mut opts = ConvOptions::new();
        opts.output_matrix.outputs.push(Default::default());
        opts.output_matrix.outputs.push(Default::default());
        let cmds = vec![
            vec!["a".to_string()],
            vec!["b".to_string()],
            vec!["c".to_string()],
        ];
        // build_commands 会 reverse；模拟其语义
        let mut reversed = cmds.clone();
        reversed.reverse();
        let stack = Mutex::new(reversed);
        worker_run_test(&opts, &[], &stack, 2);
        assert!(stack.lock().unwrap().is_empty());
    }
}
