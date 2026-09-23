use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

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

/// java 启动前缀：[java_path, -{CLI java option}..., {XML java option}..., -Dfile.encoding, -jar, xresloader, --stdin]
pub fn java_command_prefix(opts: &ConvOptions, cli_java_options: &[String]) -> Vec<String> {
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
    java_options.push("--stdin".to_string());
    java_options
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

    let worker_count = usize::try_from(parallelism)
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
