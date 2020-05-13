use super::super::environment::Environment;
use super::super::types::{ErrBox, Error};
use super::PluginCache;
use super::wasm::{WasmPlugin};
use super::Plugin;

pub type PluginContainer = Vec<Box<dyn Plugin>>;

pub async fn load_plugins(urls: Vec<String>, environment: &impl Environment) -> Result<PluginContainer, ErrBox> {
    let mut cache = PluginCache::new(environment)?;
    let mut plugin_container = Vec::new();

    for url in urls.iter() {
        let plugin = match load_plugin(url, &mut cache, environment).await {
            Ok(plugin) => plugin,
            Err(err) => {
                return err!("Error loading plugin at url {}: {}", url, err);
            }
        };
        plugin_container.push(plugin);
    }

    Ok(plugin_container)
}

async fn load_plugin<'a, TEnvironment : Environment>(
    url: &str,
    cache: &mut PluginCache<'a, TEnvironment>,
    environment: &TEnvironment,
) -> Result<Box<dyn Plugin>, ErrBox> {
    let file_path = cache.get_plugin_file_path(url).await?;
    let file_bytes = environment.read_file_bytes(&file_path)?;
    let plugin = WasmPlugin::new(file_bytes)?;

    Ok(Box::new(plugin))
}
