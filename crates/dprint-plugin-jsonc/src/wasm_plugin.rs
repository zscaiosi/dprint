use std::path::PathBuf;
use dprint_core::generate_plugin_code;
use super::configuration::{Configuration, resolve_config};

fn get_plugin_config_keys() -> Vec<String> {
    vec![String::from("json"), String::from("jsonc")]
}

fn get_plugin_file_extensions() -> Vec<String> {
    vec![String::from("json")]
}

fn format_text(_: &PathBuf, file_text: &str, config: &Configuration) -> Result<String, String> {
    super::format_text::format_text(file_text, config)
}

generate_plugin_code!(Configuration, resolve_config);
