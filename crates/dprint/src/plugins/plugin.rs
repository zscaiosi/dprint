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
    fn should_format_file(&self, file_path: &PathBuf, file_text: &str) -> bool;
    /// Gets the configuration as a collection of key value pairs.
    fn get_resolved_config(&self) -> String;
    /// Gets the configuration diagnostics.
    fn get_config_diagnostics(&self) -> Vec<ConfigurationDiagnostic>;
    /// Formats the text in memory based on the file path and file text.
    fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<String, String>;
}
