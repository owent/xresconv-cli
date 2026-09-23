pub mod cli;
pub mod color;
pub mod options;
pub mod plan;
mod process;
pub mod runner;
pub mod xml_conf;

use std::path::{Path, PathBuf};

use cli::CliOptions;
use color::{Color, cprintln_stderr, cprintln_stdout};
use options::ConvOptions;

pub const XRESLOADER_DOWNLOAD_URL: &str = "https://github.com/xresloader/xresloader/releases";

pub fn default_parallelism() -> i64 {
    let cpu = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1);
    // 与原 Python 实现一致：int((cpu_count() - 1) / 2) + 1，上限 2
    std::cmp::min(2, (cpu - 1) / 2 + 1)
}

fn resolve_java_path(cli_java_path: &Option<String>) -> String {
    if let Some(p) = cli_java_path {
        // Resolve an explicit relative path before the child changes its cwd.
        // Bare command names still use PATH; invalid explicit paths must fail.
        return if Path::new(p).is_file() {
            std::fs::canonicalize(p)
                .unwrap_or_else(|_| PathBuf::from(p))
                .to_string_lossy()
                .into_owned()
        } else if Path::new(p).components().count() > 1 {
            std::env::current_dir()
                .unwrap_or_default()
                .join(p)
                .to_string_lossy()
                .into_owned()
        } else {
            p.clone()
        };
    }
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_exec = if cfg!(windows) { "java.exe" } else { "java" };
        let candidate = Path::new(&java_home).join("bin").join(java_exec);
        if candidate.is_file() {
            return std::fs::canonicalize(&candidate)
                .unwrap_or(candidate)
                .to_string_lossy()
                .into_owned();
        }
    }
    "java".to_string()
}

fn normalize_dir(path: &Path) -> PathBuf {
    match std::fs::canonicalize(path) {
        Ok(p) => {
            let s = p.to_string_lossy().into_owned();
            // Keep verbatim Windows paths, including UNC and long paths.
            PathBuf::from(s)
        }
        Err(_) => path.to_path_buf(),
    }
}

pub fn run() -> i32 {
    run_cli(CliOptions::parse_args())
}

fn run_cli(cli: CliOptions) -> i32 {
    if cli.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return 0;
    }

    let mut positional = cli.convert_list_file.into_iter();
    let conv_list = match positional.next() {
        Some(v) => v,
        None => {
            CliOptions::print_help();
            return -1;
        }
    };
    let ext_args_l2: Vec<String> = positional.collect();

    let mut opts = ConvOptions::new();
    opts.conv_list = conv_list.clone();
    opts.ext_args_l2 = ext_args_l2;
    opts.data_version = cli.data_version.clone();
    opts.parallelism = cli.parallelism;
    opts.java_path = resolve_java_path(&cli.java_path);

    // ========================================= XML 解析 =========================================
    let mut xml_conf = xml_conf::XmlConf::new();
    if let Err(err) = xml_conf.load(Path::new(&conv_list)) {
        println!("{err}");
        cprintln_stderr(Some(Color::Red), &format!("[ERROR]: {err}"));
        return -2;
    }

    if let Err(err) = xml_conf.apply_global_entries(&mut opts) {
        cprintln_stderr(Some(Color::Red), &format!("[ERROR] {err}"));
        return -2;
    }

    // ========================================= 工作目录 =========================================
    let mut work_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(parent) = Path::new(&conv_list).parent()
        && !parent.as_os_str().is_empty()
    {
        work_dir = work_dir.join(parent);
    }
    work_dir = work_dir.join(&opts.work_dir);
    if !work_dir.is_dir() {
        cprintln_stderr(
            Some(Color::Red),
            &format!("[ERROR] work dir not found: {}", work_dir.display()),
        );
        return 1;
    }
    let work_dir = normalize_dir(&work_dir);

    cprintln_stdout(
        Some(Color::Yellow),
        &format!(
            "[NOTICE] start to run conv cmds on dir: {}",
            work_dir.display()
        ),
    );

    let xresloader_full_path = {
        let p = Path::new(&opts.xresloader_path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            work_dir.join(p)
        }
    };
    if !xresloader_full_path.is_file() {
        cprintln_stderr(
            Some(Color::Red),
            &format!(
                "[ERROR] xresloader not found.({}, you can download it from {})",
                opts.xresloader_path, XRESLOADER_DOWNLOAD_URL
            ),
        );
        return -4;
    }

    // ========================================= 转换项解析 =========================================
    xml_conf.apply_item_entries(&mut opts, &cli.rule_schemes);

    // ========================================= 生成转换命令 =========================================
    let cmd_list = match plan::build_commands(&opts) {
        Ok(commands) => commands,
        Err(err) => {
            cprintln_stderr(Some(Color::Red), &format!("[ERROR] {err}"));
            return 2;
        }
    };

    let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    if !cli.test && !cmd_list.is_empty() {
        let flag = cancelled.clone();
        if let Err(err) =
            ctrlc::set_handler(move || flag.store(true, std::sync::atomic::Ordering::SeqCst))
        {
            cprintln_stderr(
                Some(Color::Red),
                &format!("[ERROR] cannot install cancellation handler: {err}"),
            );
            return 1;
        }
    }

    // ========================================= 实际开始转换 =========================================
    let exit_code = runner::run_workers(
        &opts,
        &cli.java_options,
        cli.test,
        &work_dir,
        cmd_list,
        opts.parallelism,
        &cancelled,
    );

    cprintln_stdout(
        Some(Color::Magenta),
        &format!("[INFO] all jobs done. {exit_code} job(s) failed."),
    );
    if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
        130
    } else {
        exit_code.clamp(0, 255)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_error_statuses_and_preview_are_observable_without_exiting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("list.xml");
        let run = || {
            run_cli(CliOptions::try_parse_from(["x", "--test", path.to_str().unwrap()]).unwrap())
        };
        assert_eq!(run(), -2);
        std::fs::write(&path, "<root><global></root>").unwrap();
        assert_eq!(run(), -2);
        std::fs::write(
            &path,
            "<root><global><work_dir>absent</work_dir></global></root>",
        )
        .unwrap();
        assert_eq!(run(), 1);
        std::fs::write(&path, "<root/>").unwrap();
        assert_eq!(run(), -4);
        std::fs::write(dir.path().join("xresloader.jar"), "").unwrap();
        std::fs::write(
            &path,
            "<root><global><output_dir>new&#10;line</output_dir></global></root>",
        )
        .unwrap();
        assert_eq!(run(), -2);
        std::fs::write(
            &path,
            "<root><list><item file='a&#10;b' scheme='s'/></list></root>",
        )
        .unwrap();
        assert_eq!(run(), 2);
        std::fs::write(
            &path,
            "<root><list><item file='a.xlsx' scheme='s'/></list></root>",
        )
        .unwrap();
        assert_eq!(run(), 0);
        assert!(!dir.path().join("a.bin").exists());
    }
}
