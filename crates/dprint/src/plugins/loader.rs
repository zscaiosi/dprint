use crate::environment::Environment;
use crate::types::ErrBox;
use super::cache::Cache;
use super::wasm::{WasmPlugin, compile};
use super::{Plugin, PluginContainer, CompileFn};

pub async fn load_plugins(urls: Vec<String>, environment: &impl Environment) -> Result<PluginContainer, ErrBox> {
    let mut cache = Cache::new(environment, compile)?;
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

    Ok(PluginContainer::new(plugin_container))
}

async fn load_plugin<'a, TEnvironment : Environment, TCompileFn : CompileFn>(
    url: &str,
    cache: &mut Cache<'a, TEnvironment, TCompileFn>,
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
