use clap::{App, Arg, Values, ArgMatches};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use super::environment::Environment;
use super::configuration;
use super::configuration::{ConfigMap, ConfigMapValue};
use super::plugins::{initialize_plugins, PluginContainer, PluginLoader};
use super::types::ErrBox;

pub async fn run_cli(args: Vec<String>, environment: &impl Environment, plugin_loader: &impl PluginLoader) -> Result<(), ErrBox> {
    let cli_parser = create_cli_parser();
    let matches = match cli_parser.get_matches_from_safe(args) {
        Ok(result) => result,
        Err(err) => return err!("{}", err.to_string()),
    };

    if matches.is_present("version") {
        return output_version(&matches, environment, plugin_loader).await;
    }

    if matches.is_present("init") {
        init_config_file(environment).await?;
        environment.log("Created dprint.config.json");
        return Ok(());
    }

    let mut config_map = get_config_map_from_matches(&matches, environment)?;
    check_project_type_diagnostic(&mut config_map, environment);
    let file_paths = resolve_file_paths(&mut config_map, &matches, environment)?;

    if matches.is_present("output-file-paths") {
        output_file_paths(file_paths.iter(), environment);
        return Ok(());
    }

    let plugins = load_plugins(&mut config_map, plugin_loader).await?;
    initialize_plugins(config_map, &plugins, environment)?;

    if matches.is_present("output-resolved-config") {
        output_resolved_config(&plugins, environment);
        return Ok(());
    }

    if matches.is_present("check") {
        check_files(environment, plugins, file_paths)?
    } else {
        format_files(environment, plugins, file_paths);
    }

    Ok(())
}

async fn output_version<'a>(matches: &ArgMatches<'a>, environment: &impl Environment, plugin_loader: &impl PluginLoader) -> Result<(), ErrBox> {
    // log the cli's current version first
    environment.log(&format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));

    // now check for the plugins
    match get_config_map_from_matches(matches, environment) {
        Ok(config_map) => {
            let mut config_map = config_map;
            let plugins = load_plugins(&mut config_map, plugin_loader).await?;

            // output their names and versions
            for plugin in plugins.iter() {
                environment.log(&format!("{} v{}", plugin.name(), plugin.version()));
            }
        },
        Err(_) => {
            // ignore
        }
    }

    Ok(())
}

fn output_file_paths<'a>(file_paths: impl Iterator<Item=&'a PathBuf>, environment: &impl Environment) {
    for file_path in file_paths {
        environment.log(&file_path.to_string_lossy())
    }
}

fn output_resolved_config(plugins: &PluginContainer, environment: &impl Environment) {
    for plugin in plugins.iter() {
        environment.log(&format!("{}: {}", plugin.config_keys().join("/"), plugin.get_resolved_config()));
    }
}

async fn init_config_file(environment: &impl Environment) -> Result<(), ErrBox> {
    let config_file_path = PathBuf::from("./dprint.config.json");
    if !environment.path_exists(&config_file_path) {
        environment.write_file(&config_file_path, &configuration::get_init_config_file_text(environment).await?)
    } else {
        err!("Configuration file 'dprint.config.json' already exists in current working directory.")
    }
}

fn check_files(environment: &impl Environment, plugins: PluginContainer, file_paths: Vec<PathBuf>) -> Result<(), String> {
    let not_formatted_files_count = AtomicUsize::new(0);

    // todo: parallelize
    for file_path in file_paths {
        let file_contents = environment.read_file(&file_path);
        match file_contents {
            Ok(file_contents) => {
                match plugins.format_text(&file_path, &file_contents) {
                    Ok(Some(formatted_file_text)) => {
                        if formatted_file_text != file_contents {
                            not_formatted_files_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    Ok(None) => {}, // do nothing
                    Err(e) => {
                        output_error(environment, &file_path, "Error checking", &e);
                    },
                }
            },
            Err(e) => {
                output_error(environment, &file_path, "Error reading file", &e);
            },
        }
    }

    let not_formatted_files_count = not_formatted_files_count.load(Ordering::SeqCst);
    if not_formatted_files_count == 0 {
        Ok(())
    } else {
        let f = if not_formatted_files_count == 1 { "file" } else { "files" };
        Err(format!("Found {} not formatted {}.", not_formatted_files_count, f))
    }
}

fn format_files(environment: &impl Environment, plugins: PluginContainer, file_paths: Vec<PathBuf>) {
    let formatted_files_count = AtomicUsize::new(0);
    let files_count = file_paths.len();

    // todo: parallelize
    for file_path in file_paths {
        let file_contents = environment.read_file(&file_path);

        match file_contents {
            Ok(file_contents) => {
                match plugins.format_text(&file_path, &file_contents) {
                    Ok(Some(formatted_text)) => {
                        if formatted_text != file_contents {
                            match environment.write_file(&file_path, &formatted_text) {
                                Ok(_) => {
                                    formatted_files_count.fetch_add(1, Ordering::SeqCst);
                                },
                                Err(e) => output_error(environment, &file_path, "Error writing file", &e),
                            };
                        }
                    }
                    Ok(None) => {}, // do nothing
                    Err(e) => output_error(environment, &file_path, "Error formatting", &e),
                }
            },
            Err(e) => output_error(environment, &file_path, "Error reading file", &e),
        }
    }

    let formatted_files_count = formatted_files_count.load(Ordering::SeqCst);
    if formatted_files_count > 0 {
        let suffix = if files_count == 1 { "file" } else { "files" };
        environment.log(&format!("Formatted {} {}.", formatted_files_count, suffix));
    }
}

fn output_error(environment: &impl Environment, file_path: &PathBuf, text: &str, error: &impl std::fmt::Display) {
    environment.log_error(&format!("{}: {}\n    {}", text, &file_path.to_string_lossy(), error));
}

fn create_cli_parser<'a, 'b>() -> clap::App<'a, 'b> {
    App::new("dprint")
        .about("Format source files")
        .long_about(
            r#"Auto-format JavaScript, TypeScript, and JSON source code.

  dprint "**/*.{ts,tsx,js,jsx,json}"

  dprint --check myfile1.ts myfile2.ts

  dprint --config dprint.config.json"#,
        )
        .arg(
            Arg::with_name("check")
                .long("check")
                .help("Check if the source files are formatted.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .help("Path to JSON configuration file.")
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
                .help("Allows traversing node module directories.")
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

async fn load_plugins(config_map: &mut ConfigMap, plugin_loader: &impl PluginLoader) -> Result<PluginContainer, ErrBox> {
    let plugin_urls = take_array_from_config_map(config_map, "plugins")?;
    plugin_loader.load_plugins(&plugin_urls).await
}

fn check_project_type_diagnostic(config_map: &mut ConfigMap, environment: &impl Environment) {
    if !config_map.is_empty() {
        if let Some(diagnostic) = configuration::handle_project_type_diagnostic(config_map) {
            environment.log_error(&diagnostic.message);
        }
    }
}

fn deserialize_config_file(config_path: Option<&str>, environment: &impl Environment) -> Result<ConfigMap, ErrBox> {
    let config_path = config_path.unwrap_or("./dprint.config.json");
    let config_file_text = environment.read_file(&PathBuf::from(config_path))?;

    let result = match configuration::deserialize_config(&config_file_text) {
        Ok(map) => map,
        Err(e) => return err!("Error deserializing. {}", e.to_string()),
    };

    Ok(result)
}

fn resolve_file_paths(config_map: &mut ConfigMap, args: &ArgMatches, environment: &impl Environment) -> Result<Vec<PathBuf>, ErrBox> {
    let mut file_patterns = get_config_file_patterns(config_map)?;
    file_patterns.extend(resolve_file_patterns_from_cli(args.values_of("file patterns")));
    if !args.is_present("allow-node-modules") {
        file_patterns.push(String::from("!**/node_modules/**/*"));
    }
    return environment.glob(&file_patterns);

    fn resolve_file_patterns_from_cli(cli_file_patterns: Option<Values>) -> Vec<String> {
        if let Some(file_patterns) = cli_file_patterns {
            file_patterns.map(std::string::ToString::to_string).collect()
        } else {
            Vec::new()
        }
    }

    fn get_config_file_patterns(config_map: &mut ConfigMap) -> Result<Vec<String>, ErrBox> {
        let mut patterns = Vec::new();
        patterns.extend(take_array_from_config_map(config_map, "includes")?);
        patterns.extend(
            take_array_from_config_map(config_map, "excludes")?
                .into_iter()
                .map(|exclude| if exclude.starts_with("!") { exclude } else { format!("!{}", exclude) })
        );
        return Ok(patterns);
    }
}

fn get_config_map_from_matches(matches: &ArgMatches, environment: &impl Environment) -> Result<ConfigMap, ErrBox> {
    deserialize_config_file(matches.value_of("config"), environment)
}

// todo: move somewhere else (maybe make a wrapper around ConfigMap)
fn take_array_from_config_map(config_map: &mut ConfigMap, property_name: &str) -> Result<Vec<String>, ErrBox> {
    let mut result = Vec::new();
    if let Some(value) = config_map.remove(property_name) {
        match value {
            ConfigMapValue::Vec(elements) => {
                result.extend(elements);
            },
            _ => return err!("Expected array in '{}' property.", property_name),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::run_cli;
    use crate::environment::{Environment, TestEnvironment};
    use crate::configuration::*;
    use crate::plugins::wasm::WasmPluginLoader;
    use crate::types::ErrBox;

    async fn run_test_cli(args: Vec<&'static str>, environment: &impl Environment) -> Result<(), ErrBox> {
        let mut args: Vec<String> = args.into_iter().map(String::from).collect();
        args.insert(0, String::from(""));
        let plugin_loader = WasmPluginLoader::new(environment, &quick_compile);
        run_cli(args, environment, &plugin_loader).await
    }

    #[tokio::test]
    async fn it_should_output_version_with_no_plugins() {
        let environment = TestEnvironment::new();
        run_test_cli(vec!["--version"], &environment).await.unwrap();
        let logged_messages = environment.get_logged_messages();
        assert_eq!(logged_messages, vec![format!("dprint v{}", env!("CARGO_PKG_VERSION"))]);
    }

    #[tokio::test]
    async fn it_should_output_version_with_plugins() {
        let environment = get_test_environment_with_remote_plugin();
        environment.write_file(&PathBuf::from("./dprint.config.json"), r#"{
            "projectType": "openSource",
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();

        run_test_cli(vec!["--version"], &environment).await.unwrap();
        let logged_messages = environment.get_logged_messages();
        assert_eq!(logged_messages, vec![
            format!("dprint v{}", env!("CARGO_PKG_VERSION")),
            String::from("Compiling wasm module..."), // this should happen after getting dprint version
            String::from("test-plugin v0.1.0")
        ]);

        environment.clear_logs();
        run_test_cli(vec!["--version"], &environment).await.unwrap();
        let logged_messages = environment.get_logged_messages();
        assert_eq!(logged_messages, vec![
            format!("dprint v{}", env!("CARGO_PKG_VERSION")),
            String::from("test-plugin v0.1.0")
        ]);
    }

    #[tokio::test]
    async fn it_should_output_resolve_config() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        run_test_cli(vec!["--output-resolved-config"], &environment).await.unwrap();
        let logged_messages = environment.get_logged_messages();
        assert_eq!(logged_messages, vec!["test-plugin: {\n  \"ending\": \"formatted\"\n}"]);
    }

    #[tokio::test]
    async fn it_should_output_resolved_file_paths() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("/file.ts"), "const t=4;").unwrap();
        environment.write_file(&PathBuf::from("/file2.ts"), "const t=4;").unwrap();
        run_test_cli(vec!["--output-file-paths", "**/*.ts"], &environment).await.unwrap();
        let mut logged_messages = environment.get_logged_messages();
        logged_messages.sort();
        assert_eq!(logged_messages, vec!["/file.ts", "/file2.ts"]);
    }

    #[tokio::test]
    async fn it_should_format_files() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        let file_path = PathBuf::from("/file.txt");
        environment.write_file(&file_path, "text").unwrap();
        run_test_cli(vec!["/file.txt"], &environment).await.unwrap();
        assert_eq!(environment.get_logged_messages(), vec!["Formatted 1 file."]);
        assert_eq!(environment.get_logged_errors().len(), 0);
        assert_eq!(environment.read_file(&file_path).unwrap(), "text_formatted");
    }

    #[tokio::test]
    async fn it_should_ignore_files_in_node_modules_by_default() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("/node_modules/file.txt"), "").unwrap();
        environment.write_file(&PathBuf::from("/test/node_modules/file.txt"), "").unwrap();
        run_test_cli(vec!["**/*.txt"], &environment).await.unwrap();
        assert_eq!(environment.get_logged_messages().len(), 0);
        assert_eq!(environment.get_logged_errors().len(), 0);
    }

    #[tokio::test]
    async fn it_should_not_ignore_files_in_node_modules_when_allowed() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("/node_modules/file.txt"), "const t=4;").unwrap();
        environment.write_file(&PathBuf::from("/test/node_modules/file.txt"), "const t=4;").unwrap();
        run_test_cli(vec!["--allow-node-modules", "**/*.txt"], &environment).await.unwrap();
        assert_eq!(environment.get_logged_messages(), vec!["Formatted 2 files."]);
        assert_eq!(environment.get_logged_errors().len(), 0);
    }

    #[tokio::test]
    async fn it_should_format_files_with_config() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        let file_path1 = PathBuf::from("/file1.txt");
        let file_path2 = PathBuf::from("/file2.txt");
        environment.write_file(&PathBuf::from("/config.json"), r#"{
            "projectType": "openSource",
            "test-plugin": {
                "ending": "custom-formatted"
            },
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();
        environment.write_file(&file_path1, "text").unwrap();
        environment.write_file(&file_path2, "text2").unwrap();

        run_test_cli(vec!["--config", "/config.json", "/file1.txt", "/file2.txt"], &environment).await.unwrap();

        assert_eq!(environment.get_logged_messages(), vec!["Formatted 2 files."]);
        assert_eq!(environment.get_logged_errors().len(), 0);
        assert_eq!(environment.read_file(&file_path1).unwrap(), "text_custom-formatted");
        assert_eq!(environment.read_file(&file_path2).unwrap(), "text2_custom-formatted");
    }

    #[tokio::test]
    async fn it_should_format_files_with_config_using_c() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        let file_path1 = PathBuf::from("/file1.txt");
        environment.write_file(&file_path1, "text").unwrap();
        environment.write_file(&PathBuf::from("/config.json"), r#"{
            "projectType": "openSource",
            "test-plugin": { "ending": "custom-formatted" },
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();

        run_test_cli(vec!["-c", "/config.json", "/file1.txt"], &environment).await.unwrap();

        assert_eq!(environment.get_logged_messages(), vec!["Formatted 1 file."]);
        assert_eq!(environment.get_logged_errors().len(), 0);
        assert_eq!(environment.read_file(&file_path1).unwrap(), "text_custom-formatted");
    }


    #[tokio::test]
    async fn it_should_error_on_plugin_config_diagnostic() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("./dprint.config.json"), r#"{
            "projectType": "openSource",
            "test-plugin": { "non-existent": 25 },
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();

        let error_message = run_test_cli(vec!["**/*.txt"], &environment).await.err().unwrap();

        assert_eq!(error_message.to_string(), "Error initializing from configuration file. Had 1 diagnostic(s).");
        assert_eq!(environment.get_logged_messages().len(), 0);
        assert_eq!(environment.get_logged_errors(), vec!["[test-plugin]: Unknown property in configuration: non-existent"]);
    }

    #[tokio::test]
    async fn it_should_format_files_with_config_includes() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        let file_path1 = PathBuf::from("/file1.txt");
        let file_path2 = PathBuf::from("/file2.txt");
        environment.write_file(&file_path1, "text1").unwrap();
        environment.write_file(&file_path2, "text2").unwrap();
        environment.write_file(&PathBuf::from("./dprint.config.json"), r#"{
            "projectType": "openSource",
            "includes": ["**/*.txt"]
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();

        run_test_cli(vec![], &environment).await.unwrap();

        assert_eq!(environment.get_logged_messages(), vec!["Formatted 2 files."]);
        assert_eq!(environment.get_logged_errors().len(), 0);
        assert_eq!(environment.read_file(&file_path1).unwrap(), "text1_formatted");
        assert_eq!(environment.read_file(&file_path2).unwrap(), "text2_formatted");
    }

    #[tokio::test]
    async fn it_should_format_files_with_config_excludes() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        let file_path1 = PathBuf::from("/file1.txt");
        let file_path2 = PathBuf::from("/file2.txt");
        environment.write_file(&file_path1, "text1").unwrap();
        environment.write_file(&file_path2, "text2").unwrap();
        environment.write_file(&PathBuf::from("./dprint.config.json"), r#"{
            "projectType": "openSource",
            "includes": ["**/*.txt"],
            "excludes": ["/file2.txt"],
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();

        run_test_cli(vec![], &environment).await.unwrap();

        assert_eq!(environment.get_logged_messages(), vec!["Formatted 1 file."]);
        assert_eq!(environment.get_logged_errors().len(), 0);
        assert_eq!(environment.read_file(&file_path1).unwrap(), "text1_formatted");
        assert_eq!(environment.read_file(&file_path2).unwrap(), "text2");
    }

    #[tokio::test]
    async fn it_should_only_warn_when_missing_project_type() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("./dprint.config.json"), r#"{
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();
        environment.write_file(&PathBuf::from("/file1.txt"), "text1_formatted").unwrap();
        run_test_cli(vec!["/file1.txt"], &environment).await.unwrap();
        assert_eq!(environment.get_logged_messages().len(), 0);
        assert_eq!(environment.get_logged_errors().len(), 1);
        assert_eq!(environment.get_logged_errors()[0].find("The 'projectType' property").is_some(), true);
    }

    #[tokio::test]
    async fn it_should_not_output_when_no_files_need_formatting() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("/file.txt"), "text_formatted").unwrap();
        run_test_cli(vec!["/file.txt"], &environment).await.unwrap();
        assert_eq!(environment.get_logged_messages().len(), 0);
        assert_eq!(environment.get_logged_errors().len(), 0);
    }

    #[tokio::test]
    async fn it_should_not_output_when_no_files_need_formatting_for_check() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        let file_path = PathBuf::from("/file.txt");
        environment.write_file(&file_path, "text_formatted").unwrap();
        run_test_cli(vec!["--check", "/file.ts"], &environment).await.unwrap();
        assert_eq!(environment.get_logged_messages().len(), 0);
        assert_eq!(environment.get_logged_errors().len(), 0);
    }

    #[tokio::test]
    async fn it_should_output_when_a_file_need_formatting_for_check() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("/file.txt"), "const t=4;").unwrap();
        let error_message = run_test_cli(vec!["--check", "/file.txt"], &environment).await.err().unwrap();
        assert_eq!(error_message.to_string(), "Found 1 not formatted file.");
        assert_eq!(environment.get_logged_messages().len(), 0);
        assert_eq!(environment.get_logged_errors().len(), 0);
    }

    #[tokio::test]
    async fn it_should_output_when_files_need_formatting_for_check() {
        let environment = get_initialized_test_environment_with_remote_plugin().await.unwrap();
        environment.write_file(&PathBuf::from("/file1.txt"), "const t=4;").unwrap();
        environment.write_file(&PathBuf::from("/file2.txt"), "const t=4;").unwrap();

        let error_message = run_test_cli(vec!["--check", "/file1.txt", "/file2.txt"], &environment).await.err().unwrap();
        assert_eq!(error_message.to_string(), "Found 2 not formatted files.");
        assert_eq!(environment.get_logged_messages().len(), 0);
        assert_eq!(environment.get_logged_errors().len(), 0);
    }

    #[tokio::test]
    async fn it_should_initialize() {
        let environment = TestEnvironment::new();
        environment.add_remote_file(crate::plugins::REMOTE_INFO_URL, r#"{
            "schemaVersion": 1,
            "pluginSystemSchemaVersion": 1,
            "latest": [{
                "name": "dprint-plugin-typescript",
                "version": "0.17.2",
                "url": "https://plugins.dprint.dev/typescript-0.17.2.wasm",
                "configKey": "typescript"
            }, {
                "name": "dprint-plugin-jsonc",
                "version": "0.2.3",
                "url": "https://plugins.dprint.dev/json-0.2.3.wasm",
                "configKey": "json"
            }]
        }"#.as_bytes());
        let expected_text = get_init_config_file_text(&environment).await.unwrap();
        run_test_cli(vec!["--init"], &environment).await.unwrap();
        assert_eq!(environment.get_logged_messages(), vec!["Created dprint.config.json"]);
        assert_eq!(environment.read_file(&PathBuf::from("./dprint.config.json")).unwrap(), expected_text);
    }

    #[tokio::test]
    async fn it_should_error_when_config_file_exists_on_initialize() {
        let environment = TestEnvironment::new();
        environment.write_file(&PathBuf::from("./dprint.config.json"), "{}").unwrap();
        let error_message = run_test_cli(vec!["--init"], &environment).await.err().unwrap();
        assert_eq!(error_message.to_string(), "Configuration file 'dprint.config.json' already exists in current working directory.");
    }

    // If this file doesn't exist, run `./build.ps1` in test/plugin. (Please consider helping me do something better here :))
    static PLUGIN_BYTES: &'static [u8] = include_bytes!("../test/test_plugin.wasm");
    lazy_static! {
        // cache the compilation so this only has to be done once across all tests
        static ref COMPILED_PLUGIN_BYTES: Vec<u8> = {
            crate::plugins::wasm::compile(PLUGIN_BYTES).unwrap()
        };
    }

    async fn get_initialized_test_environment_with_remote_plugin() -> Result<TestEnvironment, ErrBox> {
        let environment = get_test_environment_with_remote_plugin();
        environment.write_file(&PathBuf::from("./dprint.config.json"), r#"{
            "projectType": "openSource",
            "plugins": ["https://plugins.dprint.dev/test-plugin.wasm"]
        }"#).unwrap();
        run_test_cli(vec!["--version"], &environment).await.unwrap(); // cause initialization
        environment.clear_logs();
        Ok(environment)
    }

    fn get_test_environment_with_remote_plugin() -> TestEnvironment {
        let environment = TestEnvironment::new();
        environment.add_remote_file("https://plugins.dprint.dev/test-plugin.wasm", PLUGIN_BYTES);
        environment
    }

    pub fn quick_compile(wasm_bytes: &[u8]) -> Result<Vec<u8>, ErrBox> {
        if wasm_bytes == PLUGIN_BYTES {
            Ok(COMPILED_PLUGIN_BYTES.clone())
        } else {
            unreachable!()
        }
    }
}
