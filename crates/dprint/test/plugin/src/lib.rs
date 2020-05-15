use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use dprint_core::generate_plugin_code;
use dprint_core::configuration::{GlobalConfiguration, ResolveConfigurationResult, ConfigurationDiagnostic};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    ending: String,
}

pub fn resolve_config(config: HashMap<String, String>, _: &GlobalConfiguration) -> ResolveConfigurationResult<Configuration> {
    let mut config = config;
    let ending = config.remove("ending").unwrap_or(String::from("formatted"));
    let mut diagnostics = Vec::new();

    for (key, _) in config.iter() {
        diagnostics.push(ConfigurationDiagnostic {
            property_name: String::from(key),
            message: format!("Unknown property in configuration: {}", key),
        });
    }

    ResolveConfigurationResult {
        config: Configuration { ending },
        diagnostics,
    }
}

fn get_plugin_config_keys() -> Vec<String> {
    vec![String::from("test-plugin")]
}

fn get_plugin_file_extensions() -> Vec<String> {
    vec![String::from("txt")]
}

fn format_text(_: &PathBuf, file_text: &str, config: &Configuration) -> Result<String, String> {
    if file_text.ends_with(&config.ending) {
        Ok(String::from(file_text))
    } else {
        Ok(format!("{}_{}", file_text, config.ending))
    }
}

generate_plugin_code!();
