use std::path::PathBuf;
use dprint_core::generate_plugin_code;
use super::configuration::{Configuration, resolve_config};
use super::formatter::Formatter;

fn get_plugin_config_keys() -> Vec<String> {
    vec![String::from("typescript"), String::from("javascript")]
}

fn get_plugin_file_extensions() -> Vec<String> {
    vec![String::from("ts"), String::from("tsx"), String::from("js"), String::from("jsx")]
}

static mut FORMATTER: Option<Formatter> = None;

fn format_text(file_path: &PathBuf, file_text: &str, config: &Configuration) -> Result<String, String> {
    let formatter = unsafe { if let Some(formatter) = FORMATTER.as_ref() {
            formatter
        } else {
            let formatter = Formatter::new(config.clone());
            FORMATTER.replace(formatter);
            FORMATTER.as_ref().unwrap()
        }
    };
    formatter.format_text(&file_path, &file_text)
}

generate_plugin_code!();
