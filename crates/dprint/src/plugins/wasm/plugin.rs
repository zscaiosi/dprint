use std::cell::RefCell;
use dprint_core::configuration::{ConfigurationDiagnostic, GlobalConfiguration};
use dprint_core::plugins::PluginInfo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use bytes::Bytes;

use crate::types::ErrBox;
use super::super::Plugin;
use super::{BytesTransmitter, WasmFunctions, FormatResult, load_instance};

/// A lazily created wasm plugin.
pub struct LazyWasmPlugin {
    compiled_wasm_bytes: Option<Bytes>,
    plugin_info: PluginInfo,
    wasm_plugin: Option<WasmPlugin>,
}

impl LazyWasmPlugin {
    pub fn new(compiled_wasm_bytes: Bytes, plugin_info: PluginInfo) -> LazyWasmPlugin {
        LazyWasmPlugin {
            compiled_wasm_bytes: Some(compiled_wasm_bytes),
            plugin_info,
            wasm_plugin: None,
        }
    }

    pub fn get_wasm_plugin(&self) -> &WasmPlugin {
        self.wasm_plugin.as_ref().expect("Expected WASM plugin to be initialized.")
    }
}

impl Plugin for LazyWasmPlugin {
    fn name(&self) -> &str {
        &self.plugin_info.name
    }

    fn version(&self) -> &str {
        &self.plugin_info.version
    }

    fn config_keys(&self) -> &Vec<String> {
        &self.plugin_info.config_keys
    }

    fn file_extensions(&self) -> &Vec<String> {
        &self.plugin_info.file_extensions
    }

    fn initialize(&mut self, plugin_config: HashMap<String, String>, global_config: &GlobalConfiguration) -> Result<(), ErrBox> {
        let wasm_bytes = self.compiled_wasm_bytes.take().unwrap(); // free memory
        let wasm_plugin = WasmPlugin::new(&wasm_bytes)?;

        wasm_plugin.set_global_config(global_config);
        wasm_plugin.set_plugin_config(&plugin_config);

        self.wasm_plugin.replace(wasm_plugin);

        Ok(())
    }

    fn get_resolved_config(&self) -> String {
        self.get_wasm_plugin().get_resolved_config()
    }

    fn get_config_diagnostics(&self) -> Vec<ConfigurationDiagnostic> {
        self.get_wasm_plugin().get_config_diagnostics()
    }

    fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<String, String> {
        self.get_wasm_plugin().format_text(file_path, file_text)
    }
}

pub struct WasmPlugin {
    wasm_functions: Rc<WasmFunctions>,
    bytes_transmitter: BytesTransmitter,
    cached_plugin_info: RefCell<Option<PluginInfo>>,
}

impl WasmPlugin {
    pub fn new(compiled_wasm_bytes: &[u8]) -> Result<Self, ErrBox> {
        let instance = load_instance(compiled_wasm_bytes)?;
        let wasm_functions = Rc::new(WasmFunctions::new(instance)?);
        let bytes_transmitter = BytesTransmitter::new(wasm_functions.clone());

        Ok(WasmPlugin {
            wasm_functions,
            bytes_transmitter,
            cached_plugin_info: RefCell::new(None),
        })
    }

    pub fn set_global_config(&self, global_config: &GlobalConfiguration) {
        let json = serde_json::to_string(global_config).unwrap();
        self.bytes_transmitter.send_string(&json);
        self.wasm_functions.set_global_config();
    }

    pub fn set_plugin_config(&self, plugin_config: &HashMap<String, String>) {
        let json = serde_json::to_string(plugin_config).unwrap();
        self.bytes_transmitter.send_string(&json);
        self.wasm_functions.set_plugin_config();
    }

    pub fn get_resolved_config(&self) -> String {
        let len = self.wasm_functions.get_resolved_config();
        self.bytes_transmitter.receive_string(len)
    }

    pub fn get_config_diagnostics(&self) -> Vec<ConfigurationDiagnostic> {
        let len = self.wasm_functions.get_config_diagnostics();
        let json_text = self.bytes_transmitter.receive_string(len);
        serde_json::from_str(&json_text).unwrap()
    }

    pub fn get_plugin_info(&self) -> PluginInfo {
        if self.cached_plugin_info.borrow().is_none() {
            let len = self.wasm_functions.get_plugin_info();
            let json_text = self.bytes_transmitter.receive_string(len);
            let plugin_info = serde_json::from_str(&json_text).unwrap();
            self.cached_plugin_info.borrow_mut().replace(plugin_info);
        }

        // todo: avoid cloning
        self.cached_plugin_info.borrow().as_ref().unwrap().clone()
    }

    pub fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<String, String> {
        // send file path
        self.bytes_transmitter.send_string(&file_path.to_string_lossy());
        self.wasm_functions.set_file_path();

        // send file text and format
        self.bytes_transmitter.send_string(file_text);
        let response_code = self.wasm_functions.format();

        // handle the response
        match response_code {
            FormatResult::NoChange => Ok(String::from(file_text)),
            FormatResult::Change => {
                let len = self.wasm_functions.get_formatted_text();
                Ok(self.bytes_transmitter.receive_string(len))
            }
            FormatResult::Error => {
                let len = self.wasm_functions.get_error_text();
                Err(self.bytes_transmitter.receive_string(len))
            }
        }
    }
}
