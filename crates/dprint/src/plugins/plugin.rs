use dprint_core::configuration::{ConfigurationDiagnostic, GlobalConfiguration};
use std::collections::HashMap;
use std::path::PathBuf;

pub trait Plugin {
    /// The name of the plugin.
    fn name(&self) -> String;
    /// The version of the plugin.
    fn version(&self) -> String;
    /// Gets the possible keys that can be used in the configuration JSON.
    fn config_keys(&self) -> Vec<String>;
    /// Initializes the plugin.
    fn initialize(&self, plugin_config: HashMap<String, String>, global_config: &GlobalConfiguration);
    /// Gets whether the specified file should be formatted.
    fn should_format_file(&self, file_path: &PathBuf) -> bool;
    /// Gets the configuration as a collection of key value pairs.
    fn get_resolved_config(&self) -> String;
    /// Gets the configuration diagnostics.
    fn get_config_diagnostics(&self) -> Vec<ConfigurationDiagnostic>;
    /// Formats the text in memory based on the file path and file text.
    fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<String, String>;
}

#[cfg(test)]
pub struct TestPlugin {
    name: &'static str,
    config_keys: Vec<String>,
    file_extensions: Vec<String>,
    diagnostics: Vec<ConfigurationDiagnostic>,
}

#[cfg(test)]
impl TestPlugin {
    pub fn new(name: &'static str, config_keys: Vec<&'static str>, file_extensions: Vec<&'static str>) -> TestPlugin {
        TestPlugin {
            name,
            config_keys: config_keys.into_iter().map(String::from).collect(),
            file_extensions: file_extensions.into_iter().map(String::from).collect(),
            diagnostics: vec![],
        }
    }

    pub fn set_diagnostics(&mut self, diagnostics: Vec<(&'static str, &'static str)>) {
        self.diagnostics = diagnostics.into_iter().map(|(property_name, message)| ConfigurationDiagnostic {
            property_name: String::from(property_name),
            message: String::from(message),
        }).collect()
    }
}

#[cfg(test)]
impl Plugin for TestPlugin {
    fn name(&self) -> String { String::from(self.name) }
    fn version(&self) -> String { String::from("1.0.0") }
    fn config_keys(&self) -> Vec<String> { self.config_keys.clone() }
    fn initialize(&self, _: HashMap<String, String>, _: &GlobalConfiguration) { }
    fn should_format_file(&self, file_path: &PathBuf) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext = String::from(ext).to_lowercase();
            self.file_extensions.contains(&ext)
        } else {
            false
        }
    }
    fn get_resolved_config(&self) -> String {
        String::from("{}")
    }
    fn get_config_diagnostics(&self) -> Vec<ConfigurationDiagnostic> { self.diagnostics.clone() }
    fn format_text(&self, _: &PathBuf, text: &str) -> Result<String, String> {
        Ok(format!("{}_formatted", text))
    }
}
