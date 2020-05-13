use std::collections::HashMap;
use dprint_core::configuration::{ConfigurationDiagnostic, ResolveConfigurationResult, GlobalConfiguration};
use std::path::PathBuf;
use std::cell::RefCell;
use std::sync::Mutex;
use dprint_core::plugins::*;
use super::configuration::{Configuration, resolve_config};
use super::format_text::format_text;

/// JSONC Dprint CLI Plugin.
pub struct JsoncPlugin {
    plugin: Mutex<RefCell<Option<InitializedJsoncPlugin>>>,
}

struct InitializedJsoncPlugin {
    resolve_config_result: ResolveConfigurationResult<Configuration>,
}

impl JsoncPlugin {
    pub fn new() -> JsoncPlugin {
        JsoncPlugin {
            plugin: Mutex::new(RefCell::new(None)),
        }
    }
}

impl Plugin for JsoncPlugin {
    fn dispose(&self) {
        let plugin_lock = self.plugin.lock().unwrap();
        let mut plugin_cell = plugin_lock.borrow_mut();
        let result = plugin_cell.take();
        drop(result);
    }

    fn name(&self) -> &'static str { env!("CARGO_PKG_NAME") }
    fn version(&self) -> &'static str { env!("CARGO_PKG_VERSION") }
    fn config_keys(&self) -> Vec<&'static str> { vec!["json", "jsonc"] }

    fn initialize(&self, plugin_config: HashMap<String, String>, global_config: &GlobalConfiguration) {
        let plugin_lock = self.plugin.lock().unwrap();
        let mut plugin_cell = plugin_lock.borrow_mut();
        let config_result = resolve_config(plugin_config, &global_config);
        plugin_cell.replace(InitializedJsoncPlugin {
            resolve_config_result: config_result,
        });
    }

    fn should_format_file(&self, file_path: &PathBuf, _: &str) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            String::from(ext).to_lowercase() == "json"
        } else {
            false
        }
    }

    fn get_resolved_config(&self) -> String {
        let plugin_lock = self.plugin.lock().unwrap();
        let plugin_cell = plugin_lock.borrow();
        let plugin = plugin_cell.as_ref().expect("Plugin must be initialized before use.");
        serde_json::to_string_pretty(&plugin.resolve_config_result.config).unwrap()
    }

    fn get_configuration_diagnostics(&self) -> Vec<ConfigurationDiagnostic> {
        let plugin_lock = self.plugin.lock().unwrap();
        let plugin_cell = plugin_lock.borrow();
        let plugin = plugin_cell.as_ref().expect("Plugin must be initialized before use.");
        plugin.resolve_config_result.diagnostics.clone()
    }

    fn format_text(&self, _: &PathBuf, file_text: &str) -> Result<String, String> {
        // todo: don't use a lock here... this doesn't work for parallelization
        let plugin_lock = self.plugin.lock().unwrap();
        let plugin_cell = plugin_lock.borrow();
        let plugin = plugin_cell.as_ref().expect("Plugin must be initialized before use.");
        format_text(file_text, &plugin.resolve_config_result.config)
    }
}
