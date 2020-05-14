use async_trait::async_trait;
use crate::environment::Environment;
use crate::types::ErrBox;
use super::super::cache::Cache;
use super::super::{Plugin, PluginContainer, CompileFn, PluginLoader};
use super::{WasmPlugin, compile};

pub struct WasmPluginLoader<'a, TEnvironment : Environment> {
    environment: &'a TEnvironment,
}

#[async_trait(?Send)]
impl<'a, TEnvironment : Environment> PluginLoader for WasmPluginLoader<'a, TEnvironment> {
    async fn load_plugins(&self, urls: &Vec<String>) -> Result<PluginContainer, ErrBox> {
        let mut cache = Cache::new(self.environment, compile)?;
        let mut plugin_container = Vec::new();

        for url in urls.iter() {
            let plugin = match self.load_plugin(url, &mut cache).await {
                Ok(plugin) => plugin,
                Err(err) => {
                    cache.forget_url(url)?;
                    return err!("Error loading plugin at url {}: {}", url, err);
                }
            };
            plugin_container.push(plugin);
        }

        Ok(PluginContainer::new(plugin_container))
    }
}

impl<'a, TEnvironment : Environment> WasmPluginLoader<'a, TEnvironment> {
    pub fn new(environment: &'a TEnvironment) -> WasmPluginLoader<'a, TEnvironment> {
        WasmPluginLoader { environment }
    }

    async fn load_plugin<TCompileFn : CompileFn>(
        &self,
        url: &str,
        cache: &mut Cache<'a, TEnvironment, TCompileFn>,
    ) -> Result<Box<dyn Plugin>, ErrBox> {
        let file_path = cache.get_plugin_file_path(url).await?;
        let file_bytes = match self.environment.read_file_bytes(&file_path) {
            Ok(file_bytes) => file_bytes,
            Err(err) => {
                self.environment.log_error(&format!(
                    "Error reading plugin file bytes. Forgetting from cache and attempting redownload. Message: {:?}",
                    err
                ));

                // try again
                cache.forget_url(url)?;
                let file_path = cache.get_plugin_file_path(url).await?;
                self.environment.read_file_bytes(&file_path)?
            }
        };

        Ok(Box::new(WasmPlugin::new(file_bytes)?))
    }
}
