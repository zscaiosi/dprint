use dprint_core::configuration::{ConfigurationDiagnostic, GlobalConfiguration};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::ErrBox;

pub trait Plugin : std::marker::Send {
    /// The name of the plugin.
    fn name(&self) -> &str;
    /// The version of the plugin.
    fn version(&self) -> &str;
    /// Gets the possible keys that can be used in the configuration JSON.
    fn config_keys(&self) -> &Vec<String>;
    /// Gets the file extensions.
    fn file_extensions(&self) -> &Vec<String>;
    /// Initializes the plugin.
    fn initialize(&mut self, plugin_config: HashMap<String, String>, global_config: &GlobalConfiguration) -> Result<Box<dyn InitializedPlugin>, ErrBox>;
}

pub trait InitializedPlugin {
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
    initialized_test_plugin: Option<InitializedTestPlugin>,
}

#[cfg(test)]
impl TestPlugin {
    pub fn new(name: &'static str, config_keys: Vec<&'static str>, file_extensions: Vec<&'static str>) -> TestPlugin {
        TestPlugin {
            name,
            config_keys: config_keys.into_iter().map(String::from).collect(),
            file_extensions: file_extensions.into_iter().map(String::from).collect(),
            initialized_test_plugin: Some(InitializedTestPlugin::new()),
        }
    }
}

#[cfg(test)]
impl Plugin for TestPlugin {
    fn name(&self) -> &str { &self.name }
    fn version(&self) -> &str { "1.0.0" }
    fn config_keys(&self) -> &Vec<String> { &self.config_keys }
    fn file_extensions(&self) -> &Vec<String> { &self.file_extensions }
    fn initialize(&mut self, _: HashMap<String, String>, _: &GlobalConfiguration) -> Result<Box<dyn InitializedPlugin>, ErrBox> {
        Ok(Box::new(self.initialized_test_plugin.take().unwrap()))
    }
}

#[cfg(test)]
pub struct InitializedTestPlugin {
    diagnostics: Vec<ConfigurationDiagnostic>,
}

#[cfg(test)]
impl InitializedTestPlugin {
    pub fn new() -> InitializedTestPlugin {
        InitializedTestPlugin {
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
impl InitializedPlugin for InitializedTestPlugin {
    fn get_resolved_config(&self) -> String {
        String::from("{}")
    }
    fn get_config_diagnostics(&self) -> Vec<ConfigurationDiagnostic> { self.diagnostics.clone() }
    fn format_text(&self, _: &PathBuf, text: &str) -> Result<String, String> {
        Ok(format!("{}_formatted", text))
    }
}
