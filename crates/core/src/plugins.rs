use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::configuration::{ConfigurationDiagnostic, GlobalConfiguration};

// Note: All plugin methods must use &self instead of &mut self
// Read more: https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

/// Plugin that can be implemented for use in the CLI.
pub trait Plugin : std::any::Any + Send + std::marker::Sync {
    /// Frees any resources held by the plugin.
    fn dispose(&self);
    /// The name of the plugin.
    fn name(&self) -> &'static str;
    /// The version of the plugin.
    fn version(&self) -> &'static str;
    /// Gets the possible keys that can be used in the configuration JSON.
    fn config_keys(&self) -> Vec<&'static str>;
    /// Initializes the plugin.
    fn initialize(&self, plugin_config: HashMap<String, String>, global_config: &GlobalConfiguration);
    /// Gets whether the specified file should be formatted.
    fn should_format_file(&self, file_path: &PathBuf, file_text: &str) -> bool;
    /// Gets the configuration as a collection of key value pairs.
    fn get_resolved_config(&self) -> String;
    /// Gets the configuration diagnostics.
    fn get_configuration_diagnostics(&self) -> Vec<ConfigurationDiagnostic>;
    /// Formats the text in memory based on the file path and file text.
    fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<String, String>;
}

/// Information about a plugin.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    /// The name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
    /// Gets the possible keys that can be used in the configuration JSON.
    pub config_keys: Vec<String>,
    /// The file extensions this plugin supports.
    pub file_extensions: Vec<String>,
}
