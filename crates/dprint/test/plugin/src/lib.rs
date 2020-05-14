use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use dprint_core::generate_plugin_code;
use dprint_core::configuration::{GlobalConfiguration, ResolveConfigurationResult};

#[derive(Clone, Serialize, Deserialize)]
pub struct Configuration {
}

pub fn resolve_config(_: HashMap<String, String>, _: &GlobalConfiguration) -> ResolveConfigurationResult<Configuration> {
    ResolveConfigurationResult {
        config: Configuration {},
        diagnostics: Vec::new(),
    }
}

fn get_plugin_config_keys() -> Vec<String> {
    vec![String::from("my-plugin")]
}

fn get_plugin_file_extensions() -> Vec<String> {
    vec![String::from("txt")]
}

fn format_text(_: &PathBuf, file_text: &str, _: &Configuration) -> Result<String, String> {
    Ok(format!("{}_formatted", file_text))
}

generate_plugin_code!();
