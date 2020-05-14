use std::path::PathBuf;
use core::slice::{Iter};

use super::super::environment::Environment;
use super::super::types::ErrBox;
use super::Cache;
use super::wasm::{WasmPlugin};
use super::Plugin;

pub struct PluginContainer(Vec<Box<dyn Plugin>>);

impl PluginContainer {
    /// Iterates over the plugins.
    pub fn iter(&self) -> Iter<'_, Box<dyn Plugin>> {
        self.0.iter()
    }

    /// Formats the file text with one of the plugins.
    ///
    /// Returns the string when a plugin formatted or error. Otherwise None when no plugin was found.
    pub fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<Option<String>, String> {
        for plugin in self.iter() {
            if plugin.should_format_file(file_path, file_text) {
                return plugin.format_text(file_path, file_text).map(|x| Some(x));
            }
        }

        Ok(None)
    }
}

pub async fn load_plugins(urls: Vec<String>, environment: &impl Environment) -> Result<PluginContainer, ErrBox> {
    let mut cache = Cache::new(environment)?;
    let mut plugin_container = Vec::new();

    for url in urls.iter() {
        let plugin = match load_plugin(url, &mut cache, environment).await {
            Ok(plugin) => plugin,
            Err(err) => {
                cache.forget_url(url)?;
                return err!("Error loading plugin at url {}: {}", url, err);
            }
        };
        plugin_container.push(plugin);
    }

    Ok(PluginContainer(plugin_container))
}

async fn load_plugin<'a, TEnvironment : Environment>(
    url: &str,
    cache: &mut Cache<'a, TEnvironment>,
    environment: &TEnvironment,
) -> Result<Box<dyn Plugin>, ErrBox> {
    let file_path = cache.get_plugin_file_path(url).await?;
    let file_bytes = match environment.read_file_bytes(&file_path) {
        Ok(file_bytes) => file_bytes,
        Err(err) => {
            environment.log_error(&format!(
                "Error reading plugin file bytes. Forgetting from cache and attempting redownload. Message: {:?}",
                err
            ));

            cache.forget_url(url)?;
            let file_path = cache.get_plugin_file_path(url).await?;
            environment.read_file_bytes(&file_path)?
        }
    };
    let plugin = WasmPlugin::new(file_bytes)?;

    Ok(Box::new(plugin))
}
