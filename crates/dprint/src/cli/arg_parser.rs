use clap::{App, Arg};
use crate::types::ErrBox;

pub struct CliArgs {
    pub version: bool,
    pub init: bool,
    pub clear_cache: bool,
    pub output_file_paths: bool,
    pub output_resolved_config: bool,
    pub allow_node_modules: bool,
    pub verbose: bool,
    pub write: bool,
    pub config: Option<String>,
    pub file_patterns: Vec<String>,
}

pub fn parse_args(args: Vec<String>) -> Result<CliArgs, ErrBox> {
    let cli_parser = create_cli_parser();
    let matches = match cli_parser.get_matches_from_safe(args) {
        Ok(result) => result,
        Err(err) => return err!("{}", err.to_string()),
    };

    Ok(CliArgs {
        version: matches.is_present("version"),
        init: matches.is_present("init"),
        clear_cache: matches.is_present("clear-cache"),
        output_file_paths: matches.is_present("output-file-paths"),
        output_resolved_config: matches.is_present("output-resolved-config"),
        write: matches.is_present("write"),
        verbose: matches.is_present("verbose"),
        allow_node_modules: matches.is_present("allow-node-modules"),
        config: matches.value_of("config").map(String::from),
        file_patterns: matches.values_of("file patterns").map(|x| x.map(std::string::ToString::to_string).collect()).unwrap_or(Vec::new()),
    })
}

fn create_cli_parser<'a, 'b>() -> clap::App<'a, 'b> {
    App::new("dprint")
        .about("Auto-format source code")
        .long_about(
            r#"Auto-format source code.

    Create a dprint.config.json file:

      dprint --init

    Check formatting:

      dprint

    Write formatted files to file system:

      dprint --write

    Specify path to config file other than the default ./dprint.config.json:

      dprint --config configs/dprint.config.json

    Write using the specified config and file paths:

      dprint --write --config formatting.config.json "**/*.{ts,tsx,js,jsx,json}""#,
        )
        .arg(
            Arg::with_name("write")
                .long("write")
                .help("Writes the formatted files to the file system.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .help("Path to JSON configuration file. Defaults to ./dprint.config.json when not provided.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("file patterns")
                .help("List of file patterns used to find files to format.")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("allow-node-modules")
                .long("allow-node-modules")
                .help("Allows traversing node module directories (unstable - This flag will be renamed to be non-node specific in the future).")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("init")
                .long("init")
                .help("Initializes a configuration file in the current directory.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("clear-cache")
                .long("clear-cache")
                .help("Deletes the plugin cache directory.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Prints the version.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output-resolved-config")
                .long("output-resolved-config")
                .help("Prints the resolved configuration.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output-file-paths")
                .long("output-file-paths")
                .help("Prints the resolved file paths.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .help("Prints additional diagnostic information.")
                .takes_value(false),
        )
}
