use async_trait::async_trait;
use super::Plugins;
use crate::types::ErrBox;

#[async_trait(?Send)]
pub trait PluginResolver {
    async fn resolve_plugins(&self, urls: &Vec<String>) -> Result<Plugins, ErrBox>;
}
