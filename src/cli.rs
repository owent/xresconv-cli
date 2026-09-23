use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "xresconv-cli",
    about = "batch convert tool for xresconv-conf convert lists (backend: xresloader)",
    override_usage = "xresconv-cli [options...] <convert list file> [-- [xresloader options...]]",
    disable_version_flag = true,
    infer_long_args = true,
    args_override_self = true
)]
pub struct CliOptions {
    #[arg(short = 'v', long = "version", help = "show version and exit")]
    pub version: bool,

    #[arg(
        short = 's',
        long = "scheme-name",
        value_name = "<scheme>",
        help = "only convert schemes with name <scheme name>"
    )]
    pub rule_schemes: Vec<String>,

    #[arg(short = 't', long = "test", help = "test run and show cmds")]
    pub test: bool,

    #[arg(
        short = 'p',
        long = "parallelism",
        value_name = "<number>",
        default_value_t = crate::default_parallelism(),
        value_parser = clap::value_parser!(i64).range(1..),
        allow_negative_numbers = true,
        help = "set parallelism task number"
    )]
    pub parallelism: i64,

    #[arg(
        short = 'j',
        long = "java-option",
        value_name = "<java option>",
        help = "add java options to command(example: Xmx=2048m)"
    )]
    pub java_options: Vec<String>,

    #[arg(
        short = 'J',
        long = "java-path",
        value_name = "<java path>",
        help = "set path to java"
    )]
    pub java_path: Option<String>,

    #[arg(
        short = 'a',
        long = "data-version",
        value_name = "<version>",
        help = "set data version, if set it's will ignore the data_version option in convert list file"
    )]
    pub data_version: Option<String>,

    #[arg(
        value_name = "<convert list file> [-- [xresloader options...]]",
        num_args = 1..,
        help = "convert list file(xml) and options will be passed to xresloader.jar"
    )]
    pub convert_list_file: Vec<String>,
}

impl CliOptions {
    pub fn parse_args() -> Self {
        <Self as Parser>::parse()
    }

    pub fn print_help() {
        use clap::CommandFactory;
        let mut cmd = Self::command();
        let _ = cmd.print_help();
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_repeat_and_forwarding_stops_at_double_dash() {
        let cli = CliOptions::try_parse_from([
            "x",
            "-s",
            "one",
            "-s",
            "two",
            "-j",
            "Xmx1g",
            "-j",
            "Dkey=value",
            "-a",
            "first",
            "--data-version",
            "last",
            "-p",
            "1",
            "-p",
            "3",
            "list.xml",
            "--",
            "-p",
            "protobuf",
            "a b",
        ])
        .unwrap();
        assert_eq!(cli.rule_schemes, ["one", "two"]);
        assert_eq!(cli.java_options, ["Xmx1g", "Dkey=value"]);
        assert_eq!(cli.parallelism, 3);
        assert_eq!(cli.data_version.as_deref(), Some("last"));
        assert_eq!(cli.convert_list_file, ["list.xml", "-p", "protobuf", "a b"]);
    }

    #[test]
    fn invalid_options_and_parallelism_fail() {
        for args in [
            vec!["x", "--unknown"],
            vec!["x", "-p", "0"],
            vec!["x", "-p", "-1"],
            vec!["x", "-p", "NaN"],
            vec!["x", "-J"],
        ] {
            assert!(CliOptions::try_parse_from(args).is_err());
        }
    }

    #[test]
    fn version_and_help_are_independent_of_config() {
        assert!(CliOptions::try_parse_from(["x", "-v"]).unwrap().version);
        let err = CliOptions::try_parse_from(["x", "--help"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }
}
