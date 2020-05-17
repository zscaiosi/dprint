use std::path::PathBuf;
use std::collections::HashSet;
use core::slice::{Iter, IterMut};

use super::Plugin;
use crate::utils::get_lowercase_file_extension;

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

    /// Iterates over the plugins.
    pub fn iter_mut(&mut self) -> IterMut<'_, Box<dyn Plugin>> {
        self.0.iter_mut()
    }

    pub fn remove_plugins_without_extensions(&mut self, extensions: &HashSet<String>) -> Vec<Box<dyn Plugin>> {
        let mut indexes_to_remove = Vec::new();
        let mut removed_plugins = Vec::new();

        for (i, plugin) in self.iter().enumerate() {
            if !plugin_has_extension(plugin, extensions) {
                indexes_to_remove.push(i);
            }
        }

        for index in indexes_to_remove.iter().rev() {
            removed_plugins.push(self.0.remove(*index));
        }

        removed_plugins
    }

    /// Formats the file text with one of the plugins.
    ///
    /// Returns the string when a plugin formatted or error. Otherwise None when no plugin was found.
    pub fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<Option<String>, String> {
        let extension = get_lowercase_file_extension(file_path);

        if let Some(extension) = extension {
            for plugin in self.iter() {
                if plugin.file_extensions().contains(&extension) {
                    return plugin.format_text(file_path, file_text).map(|x| Some(x));
                }
            }
        }

        Ok(None)
    }
}

fn plugin_has_extension(plugin: &Box<dyn Plugin>, extensions: &HashSet<String>) -> bool {
    for ext in plugin.file_extensions() {
        if extensions.contains(ext) {
            return true;
        }
    }
    false
}
