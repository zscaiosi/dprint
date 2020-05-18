use clap::{App, Arg};
use crate::types::ErrBox;

pub struct CliArgs {
    pub version: bool,
    pub init: bool,
    pub output_file_paths: bool,
    pub output_resolved_config: bool,
    pub allow_node_modules: bool,
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
        output_file_paths: matches.is_present("output-file-paths"),
        output_resolved_config: matches.is_present("output-resolved-config"),
        write: matches.is_present("write"),
        allow_node_modules: matches.is_present("allow-node-modules"),
        config: matches.value_of("config").map(String::from),
        file_patterns: matches.values_of("file patterns").map(|x| x.map(std::string::ToString::to_string).collect()).unwrap_or(Vec::new()),
    })
}

fn create_cli_parser<'a, 'b>() -> clap::App<'a, 'b> {
    App::new("dprint")
        .about("Format source files")
        .long_about(
            r#"Auto-format JavaScript, TypeScript, and JSON source code.

  # create a dprint.config.json file
  dprint --init

  # check formatting
  dprint

  # write formatted files to file system
  dprint --write

  # specify path to config file other than the default ./dprint.config.json
  dprint --config configs/dprint.config.json

  # write using the specified config and file paths
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
            Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Outputs the version.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output-resolved-config")
                .long("output-resolved-config")
                .help("Outputs the resolved configuration.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output-file-paths")
                .long("output-file-paths")
                .help("Outputs the resolved file paths.")
                .takes_value(false),
        )
}
