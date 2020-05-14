use async_trait::async_trait;
use super::PluginContainer;
use crate::types::ErrBox;

#[async_trait(?Send)]
pub trait PluginLoader {
    async fn load_plugins(&self, urls: &Vec<String>) -> Result<PluginContainer, ErrBox>;
}

#[cfg(test)]
pub struct TestPluginLoader {
    plugins: std::cell::RefCell<Option<PluginContainer>>,
}

#[cfg(test)]
impl TestPluginLoader {
    pub fn new(plugins: Vec<Box<dyn super::Plugin>>) -> TestPluginLoader {
        TestPluginLoader {
            plugins: std::cell::RefCell::new(Some(PluginContainer::new(plugins))),
        }
    }
}

#[cfg(test)]
#[async_trait(?Send)]
impl PluginLoader for TestPluginLoader {
    async fn load_plugins(&self, urls: &Vec<String>) -> Result<PluginContainer, ErrBox> {
        Ok(self.plugins.borrow_mut().take().unwrap())
    }
}

