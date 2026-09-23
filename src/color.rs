use std::io::Write;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Magenta,
    Cyan,
}

impl Color {
    fn ansi_code(self) -> &'static str {
        match self {
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Magenta => "35",
            Color::Cyan => "36",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorMode {
    Term,
    None,
}

static COLOR_MODE: OnceLock<ColorMode> = OnceLock::new();
static PRINT_LOCK: Mutex<()> = Mutex::new(());

pub fn color_enabled() -> bool {
    *COLOR_MODE.get_or_init(resolve_mode) == ColorMode::Term
}

fn resolve_mode() -> ColorMode {
    if let Ok(mode) = std::env::var("CPRINTF_MODE")
        && !mode.is_empty()
    {
        return match mode.to_lowercase().as_str() {
            "term" | "win32_console" => {
                enable_windows_vt();
                ColorMode::Term
            }
            _ => ColorMode::None,
        };
    }
    if resolve_auto_mode() {
        enable_windows_vt();
        ColorMode::Term
    } else {
        ColorMode::None
    }
}

/// 对应 print_color.cprintf_resolve_auto_mode；
/// 差异：同时兼容 TERM=dumb（原实现判断的是 "dump"，视为缺陷修正）。
fn resolve_auto_mode() -> bool {
    resolve_auto_mode_with(
        |key| std::env::var_os(key).map(|v| v.to_string_lossy().into_owned()),
        cfg!(windows),
    )
}

fn resolve_auto_mode_with(get: impl Fn(&str) -> Option<String>, windows: bool) -> bool {
    // Python os.getenv checks truthiness, so empty environment values are absent.
    let get = |key: &str| get(key).filter(|value| !value.is_empty());
    let term_name = get("TERM").unwrap_or_default();
    let term_lower = term_name.to_lowercase();
    if term_lower == "dump" || term_lower == "dumb" {
        return false;
    }
    if term_lower.ends_with("-256color") || term_lower.ends_with("-256") {
        return true;
    }
    let term_prefixes = ["screen", "xterm", "vt100", "vt220", "rxvt"];
    let term_keywords = ["color", "ansi", "cygwin", "linux"];
    if term_prefixes.iter().any(|p| term_lower.starts_with(p))
        || term_keywords.iter().any(|k| term_lower.contains(k))
    {
        return true;
    }
    if get("COLORTERM").is_some() {
        return true;
    }
    if get("TF_BUILD").is_some() && get("AGENT_NAME").is_some() {
        return true;
    }
    if get("CI").is_some() {
        for known_ci in [
            "TRAVIS",
            "CIRCLECI",
            "APPVEYOR",
            "GITLAB_CI",
            "GITHUB_ACTIONS",
            "BUILDKITE",
            "DRONE",
        ] {
            if get(known_ci).is_some() {
                return true;
            }
        }
        if get("CI_NAME").is_some_and(|v| v.to_lowercase() == "codeship") {
            return true;
        }
        return false;
    }
    if let Some(teamcity_version) = get("TEAMCITY_VERSION") {
        return is_new_teamcity(&teamcity_version);
    }
    if let Some(term_program) = get("TERM_PROGRAM").map(|v| v.to_lowercase())
        && (term_program.contains("iterm.app") || term_program.contains("apple_terminal"))
    {
        return true;
    }
    if windows {
        let ostype = get("OSTYPE").unwrap_or_default().to_lowercase();
        if ostype == "msys" || ostype == "cygwin" {
            return false;
        }
        return true;
    }
    false
}

/// TEAMCITY_VERSION >= 9.1 时支持 ANSI：^(9\.(0*[1-9]\d*)\.|\d{2,}\.)
fn is_new_teamcity(version: &str) -> bool {
    let re = regex::Regex::new(r"^(9\.(0*[1-9]\d*)\.|\d{2,}\.)").unwrap();
    re.is_match(version)
}

#[cfg(windows)]
fn enable_windows_vt() {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::{
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE,
        STD_OUTPUT_HANDLE, SetConsoleMode,
    };
    unsafe {
        for handle_id in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            let handle = GetStdHandle(handle_id);
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                continue;
            }
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

#[cfg(not(windows))]
fn enable_windows_vt() {}

fn wrap_color(color: Option<Color>, text: &str) -> String {
    match color {
        Some(c) if color_enabled() => format!("\x1b[{}m{text}\x1b[0m", c.ansi_code()),
        _ => text.to_string(),
    }
}

pub fn cprintln_stdout(color: Option<Color>, text: &str) {
    let _guard = PRINT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out = wrap_color(color, text);
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(out.as_bytes());
    let _ = handle.write_all(b"\n");
    let _ = handle.flush();
}

pub fn cprintln_stderr(color: Option<Color>, text: &str) {
    let _guard = PRINT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out = wrap_color(color, text);
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    let _ = handle.write_all(out.as_bytes());
    let _ = handle.write_all(b"\n");
    let _ = handle.flush();
}

/// 多行文本一次输出（用于 --test 模式的命令块），避免与其它 worker 的输出交错
pub fn cprint_stdout_block(color: Option<Color>, text: &str) {
    let _guard = PRINT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out = wrap_color(color, text);
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(out.as_bytes());
    let _ = handle.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_color_detection_respects_environment_precedence() {
        let resolve = |vars: &[(&str, &str)], windows| {
            resolve_auto_mode_with(
                |key| {
                    vars.iter()
                        .find(|(k, _)| *k == key)
                        .map(|(_, value)| value.to_string())
                },
                windows,
            )
        };
        assert!(!resolve(&[], false));
        assert!(resolve(&[], true));
        assert!(!resolve(&[("OSTYPE", "msys")], true));
        assert!(!resolve(&[("OSTYPE", "cygwin")], true));
        for term in [
            "screen",
            "xterm",
            "vt220",
            "rxvt",
            "ansi",
            "linux",
            "foo-256",
            "foo-256color",
        ] {
            assert!(resolve(&[("TERM", term)], false));
        }
        assert!(!resolve(&[("COLORTERM", "")], false));
        assert!(resolve(&[("COLORTERM", "truecolor")], false));
        assert!(!resolve(&[("CI", ""), ("GITHUB_ACTIONS", "1")], false));
        assert!(resolve(
            &[("TF_BUILD", "1"), ("AGENT_NAME", "agent")],
            false
        ));
        for ci in [
            "TRAVIS",
            "CIRCLECI",
            "APPVEYOR",
            "GITLAB_CI",
            "GITHUB_ACTIONS",
            "BUILDKITE",
            "DRONE",
        ] {
            assert!(resolve(&[("CI", "1"), (ci, "1")], false));
        }
        assert!(resolve(&[("CI", "1"), ("CI_NAME", "CodeShip")], false));
        assert!(!resolve(&[("CI", "1")], true));
        assert!(resolve(&[("TEAMCITY_VERSION", "10.0")], false));
        assert!(!resolve(&[("TEAMCITY_VERSION", "8.0")], true));
        for program in ["iTerm.app", "Apple_Terminal", "apple_terminal"] {
            assert!(resolve(&[("TERM_PROGRAM", program)], false));
        }
        assert!(!resolve(&[("TERM_PROGRAM", "unknown")], false));
        assert!(!resolve(
            &[("TERM", "dumb"), ("COLORTERM", "truecolor")],
            true
        ));
    }

    #[test]
    fn teamcity_version_rule() {
        assert!(is_new_teamcity("9.1.0"));
        assert!(is_new_teamcity("10.0"));
        assert!(is_new_teamcity("2023.11"));
        assert!(!is_new_teamcity("9.0.5"));
        assert!(!is_new_teamcity("8.1.0"));
    }

    #[test]
    fn wrap_color_plain_when_disabled_or_none() {
        assert_eq!(wrap_color(None, "hello"), "hello");
    }
}
