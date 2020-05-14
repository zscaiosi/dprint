use std::path::PathBuf;
use core::slice::{Iter};

use super::Plugin;

pub struct PluginContainer(Vec<Box<dyn Plugin>>);

impl PluginContainer {
    /// Creates a new plugin container.
    pub fn new(plugins: Vec<Box<dyn Plugin>>) -> PluginContainer {
        PluginContainer(plugins)
    }

    /// Iterates over the plugins.
    pub fn iter(&self) -> Iter<'_, Box<dyn Plugin>> {
        self.0.iter()
    }

    /// Formats the file text with one of the plugins.
    ///
    /// Returns the string when a plugin formatted or error. Otherwise None when no plugin was found.
    pub fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<Option<String>, String> {
        for plugin in self.iter() {
            if plugin.should_format_file(file_path) {
                return plugin.format_text(file_path, file_text).map(|x| Some(x));
            }
        }

        Ok(None)
    }
}
