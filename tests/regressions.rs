use std::fs;
use std::process::Command;

use xresconv_cli::{options::ConvOptions, plan::build_commands, xml_conf::XmlConf};
mod support;

fn options(xml: &str) -> ConvOptions {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("list.xml");
    fs::write(&path, xml).unwrap();
    let mut conf = XmlConf::new();
    conf.load(&path).unwrap();
    let mut opts = ConvOptions::new();
    conf.apply_global_entries(&mut opts).unwrap();
    conf.apply_item_entries(&mut opts, &[]);
    opts
}

#[test]
fn include_cycle_is_reported_and_failed_load_is_transactional() {
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.xml");
    fs::write(
        dir.path().join("valid.xml"),
        "<root><global><proto>pb</proto></global></root>",
    )
    .unwrap();
    fs::write(
        &main,
        "<root><include>valid.xml</include><include>sub.xml</include></root>",
    )
    .unwrap();
    fs::write(
        dir.path().join("sub.xml"),
        "<root><include>./main.xml</include></root>",
    )
    .unwrap();
    let mut conf = XmlConf::new();
    let err = conf.load(&main).unwrap_err().to_string();
    assert!(
        err.contains("include cycle") && err.contains("main.xml") && err.contains("sub.xml"),
        "{err}"
    );
    assert!(conf.globals.is_empty());
    fs::write(
        &main,
        "<root><include>valid.xml</include><include>valid.xml</include></root>",
    )
    .unwrap();
    conf.load(&main).unwrap();
    assert_eq!(
        conf.globals.len(),
        2,
        "repeated non-cyclic includes remain supported"
    );
}

#[test]
fn include_depth_is_bounded() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..130 {
        fs::write(
            dir.path().join(format!("{i}.xml")),
            format!("<root><include>{}.xml</include></root>", i + 1),
        )
        .unwrap();
    }
    let err = XmlConf::new()
        .load(&dir.path().join("0.xml"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("depth exceeds"), "{err}");
}

#[test]
fn xml_encodings_and_internal_entities_are_compatible() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("encoded.xml");
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-16\"?><!DOCTYPE root [<!ENTITY name '中文'>]><root><list><item scheme=\"&name;\"/></list></root>";
    for little in [true, false] {
        let mut bytes = if little {
            vec![0xff, 0xfe]
        } else {
            vec![0xfe, 0xff]
        };
        for c in xml.encode_utf16() {
            bytes.extend(if little {
                c.to_le_bytes()
            } else {
                c.to_be_bytes()
            });
        }
        fs::write(&path, bytes).unwrap();
        let mut conf = XmlConf::new();
        conf.load(&path).unwrap();
        assert_eq!(conf.items[0].attr_scheme.as_deref(), Some("中文"));
    }
    fs::write(&path, b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><root><list><item scheme=\"caf\xe9\"/></list></root>").unwrap();
    let mut conf = XmlConf::new();
    conf.load(&path).unwrap();
    assert_eq!(conf.items[0].attr_scheme.as_deref(), Some("café"));
    for bytes in [
        b"<root>\xff</root>".as_slice(),
        b"<?xml version=\"1.0\" encoding=\"unknown\"?><root/>".as_slice(),
    ] {
        fs::write(&path, bytes).unwrap();
        assert!(XmlConf::new().load(&path).is_err());
    }
}

#[test]
fn namespaces_and_empty_nodes_do_not_create_options() {
    let opts = options(
        "<root xmlns:n='urn:test'><n:global><proto>bad</proto></n:global><global><option/><proto> </proto></global><n:list><item/></n:list><list><item><scheme/><option/></item></list></root>",
    );
    assert!(opts.args.entries().is_empty());
    assert_eq!(opts.items.len(), 1);
    assert!(opts.items[0].options.is_empty());
    assert!(opts.items[0].scheme_data.entries().is_empty());
}

#[test]
fn backend_quoting_preserves_quotes_backslashes_and_unicode() {
    use xresconv_cli::plan::quote_argument;
    assert_eq!(
        quote_argument(r"C:\中文 folder\").unwrap(),
        "\"C:\\中文 folder\\\""
    );
    assert_eq!(
        quote_argument("a \"quoted\" value").unwrap(),
        "'a \"quoted\" value'"
    );
    assert_eq!(quote_argument("a'b\"c").unwrap(), "a'b\"c");
    for invalid in ["both ' and \"", "new\nline", "nul\0value", "cr\rvalue"] {
        assert!(quote_argument(invalid).is_err(), "{invalid:?}");
    }
    let mut opts = options("<root><list><item file='a.xlsx' scheme='s'/></list></root>");
    opts.ext_args_l2 = vec!["--name".into(), "值 with spaces".into()];
    assert!(
        build_commands(&opts).unwrap()[0]
            .join(" ")
            .ends_with("--name \"值 with spaces\"")
    );
    opts.ext_args_l1.push("--foo\n--bar".into());
    assert!(build_commands(&opts).is_err());
}

#[test]
fn local_schemes_override_defaults_and_empty_key_replaces() {
    let opts = options(
        "<root><global><default_scheme name='k'>default</default_scheme></global><list><item><scheme name='k'>local</scheme><scheme name=''>old</scheme><scheme name=''>new</scheme></item></list></root>",
    );
    assert_eq!(
        opts.items[0].scheme_data.entries(),
        &[
            ("k".into(), vec!["local".into()]),
            ("".into(), vec!["new".into()])
        ]
    );
}

#[test]
fn grouped_protocols_and_data_sources_reset_on_including_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("inc.xml"), "<root><global><proto_file>old</proto_file><data_src_dir>old</data_src_dir><data_version>first</data_version></global></root>").unwrap();
    fs::write(dir.path().join("main.xml"), "<root><include>inc.xml</include><global><proto_file>new1</proto_file><proto_file>new2</proto_file><data_source_dir>new</data_source_dir><data_version>later</data_version></global></root>").unwrap();
    let mut conf = XmlConf::new();
    conf.load(&dir.path().join("main.xml")).unwrap();
    let mut opts = ConvOptions::new();
    conf.apply_global_entries(&mut opts).unwrap();
    assert_eq!(
        opts.protocol_files.inputs,
        ["-f", "\"new1\"", "-f", "\"new2\""]
    );
    assert_eq!(opts.data_source_dir.inputs, ["-d", "\"new\""]);
    assert_eq!(opts.data_version.as_deref(), Some("first"));
}

fn run_fake(mode: &str, jobs: usize, parallelism: usize) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let mut xml = String::from("<root><list>");
    for i in 0..jobs {
        xml.push_str(&format!("<item file='file_{i}' scheme='scheme_{i}'/>"));
    }
    xml.push_str("</list></root>");
    fs::write(dir.path().join("list.xml"), xml).unwrap();
    fs::write(dir.path().join("xresloader.jar"), "").unwrap();
    Command::new(env!("CARGO_BIN_EXE_xresconv-cli"))
        .current_dir(dir.path())
        .env("FAKE_JAVA_MODE", mode)
        .env("CPRINTF_MODE", "none")
        .arg("-J")
        .arg(support::fake_java())
        .args(["-p", &parallelism.to_string(), "list.xml"])
        .output()
        .unwrap()
}

#[test]
fn parallel_workers_execute_every_command_once() {
    for parallelism in [1, 2, 4, 100] {
        let output = run_fake("", 41, parallelism);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.matches("[ARGS]").count() <= parallelism.min(41));
        let commands: Vec<_> = stdout
            .lines()
            .filter(|s| s.starts_with("[INFO] fake java got:"))
            .collect();
        assert_eq!(commands.len(), 41);
        for i in 0..41 {
            assert_eq!(
                commands
                    .iter()
                    .filter(|c| c.contains(&format!("\"file_{i}\"")))
                    .count(),
                1
            );
        }
    }
}

#[test]
fn large_stdout_stderr_and_non_utf8_output_do_not_deadlock() {
    let output = run_fake("large-output", 2, 2);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("stdout-large-end 中文"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("stderr-large-end 中文"));
    assert!(output.stdout.len() > 256 * 1024);
    assert!(output.stderr.len() > 256 * 1024);
}

#[test]
fn broken_stdin_is_a_failure_even_when_backend_exits_zero() {
    let output = run_fake("exit-early", 4000, 1);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("backend pipe failed"));
}

#[test]
fn closed_stdin_does_not_wait_for_a_hanging_backend() {
    let start = std::time::Instant::now();
    let output = run_fake("close-stdin", 4000, 1);
    assert!(!output.status.success());
    assert!(start.elapsed() < std::time::Duration::from_secs(15));
}

#[test]
fn empty_plan_does_not_start_backend() {
    let output = run_fake("large-output", 0, 2);
    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("[ARGS]"));
}

#[test]
fn cancellation_reaps_backend_and_descendant() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};
    let dir = tempfile::tempdir().unwrap();
    let ready = dir.path().join("ready");
    let mut opts = ConvOptions::new();
    opts.java_path = support::fake_java().to_string_lossy().into_owned();
    opts.java_options = vec![
        "--fake-mode=hang".into(),
        format!("--fake-ready={}", ready.display()),
    ];
    let cancelled = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            xresconv_cli::runner::run_workers(
                &opts,
                &[],
                false,
                dir.path(),
                vec![vec!["a".into()]],
                1,
                &cancelled,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.is_file() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        cancelled.store(true, Ordering::SeqCst);
        assert!(worker.join().unwrap() > 0);
        assert!(ready.is_file(), "backend did not start");
    });
    for pid in fs::read_to_string(ready)
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<u32>().unwrap())
    {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::{
                Foundation::{CloseHandle, WAIT_TIMEOUT},
                System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
            };
            let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
            if !handle.is_null() {
                let wait = WaitForSingleObject(handle, 5000);
                CloseHandle(handle);
                assert_ne!(wait, WAIT_TIMEOUT, "process {pid} survived cancellation");
            }
        }
        #[cfg(unix)]
        {
            // A reparented zombie may briefly exist; it is no longer running.
            let output = Command::new("ps")
                .args(["-o", "stat=", "-p", &pid.to_string()])
                .output()
                .unwrap();
            let state = String::from_utf8_lossy(&output.stdout);
            assert!(
                state.trim().is_empty() || state.trim().starts_with('Z'),
                "process {pid}: {state}"
            );
        }
    }
}

#[test]
fn empty_file_or_scheme_uses_inline_scheme() {
    let opts = options(
        r#"<root><list><item file="" scheme="s"><scheme name="Key">value</scheme></item></list></root>"#,
    );
    assert_eq!(
        build_commands(&opts).unwrap(),
        vec![vec!["-m", "\"Key=value\""]]
    );
}

#[test]
fn repeated_local_scheme_preserves_whitespace() {
    let opts = options(
        r#"<root><list><item><scheme name="Key"> first </scheme><scheme name="Key"> second </scheme></item></list></root>"#,
    );
    assert_eq!(
        opts.items[0].scheme_data.entries()[0].1,
        [" first ", " second "]
    );
}

#[test]
fn non_positive_parallelism_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("list.xml"),
        "<root><list><item scheme=\"s\"/></list></root>",
    )
    .unwrap();
    fs::write(dir.path().join("xresloader.jar"), "").unwrap();
    for value in ["0", "-1"] {
        let output = Command::new(env!("CARGO_BIN_EXE_xresconv-cli"))
            .current_dir(dir.path())
            .args(["--test", &format!("--parallelism={value}"), "list.xml"])
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "parallelism {value} silently dropped work"
        );
    }
}

#[test]
fn java_cli_options_are_appended_once_with_a_dash() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("list.xml"),
        "<root><list><item scheme=\"s\"/></list></root>",
    )
    .unwrap();
    fs::write(dir.path().join("xresloader.jar"), "").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xresconv-cli"))
        .current_dir(dir.path())
        .args(["--test", "-p", "1", "-j", "Xmx512m", "list.xml"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("Xmx512m").count(), 1, "{stdout}");
    assert!(stdout.contains("\"-Xmx512m\""));
    assert!(
        !stdout.contains("\"\""),
        "invalid preview quoting: {stdout}"
    );
}
