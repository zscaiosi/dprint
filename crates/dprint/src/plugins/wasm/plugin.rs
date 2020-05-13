use std::cell::RefCell;
use dprint_core::configuration::{ConfigurationDiagnostic, GlobalConfiguration};
use dprint_core::plugins::PluginInfo;
use bytes::Bytes;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use super::super::super::types::ErrBox;
use super::super::Plugin;
use super::{BytesTransmitter, WasmFunctions, FormatResult, load};

pub struct WasmPlugin {
    wasm_functions: Rc<WasmFunctions>,
    bytes_transmitter: BytesTransmitter,
    cached_plugin_info: RefCell<Option<PluginInfo>>,
}

impl WasmPlugin {
    pub fn new(compiled_wasm_bytes: Bytes) -> Result<Self, ErrBox> {
        // todo: implement a wasm instance pool
        let instance = load(&compiled_wasm_bytes)?;
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

impl Plugin for WasmPlugin {
    fn name(&self) -> String {
        self.get_plugin_info().name
    }

    fn version(&self) -> String {
        self.get_plugin_info().version
    }

    fn config_keys(&self) -> Vec<String> {
        self.get_plugin_info().config_keys
    }

    fn initialize(&self, plugin_config: HashMap<String, String>, global_config: &GlobalConfiguration) {
        self.set_global_config(global_config);
        self.set_plugin_config(&plugin_config);
    }

    fn should_format_file(&self, file_path: &PathBuf, _: &str) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext = String::from(ext).to_lowercase();
            self.get_plugin_info().config_keys.contains(&ext)
        } else {
            false
        }

    }
    fn get_resolved_config(&self) -> String {
        self.get_resolved_config()
    }

    fn get_config_diagnostics(&self) -> Vec<ConfigurationDiagnostic> {
        self.get_config_diagnostics()
    }

    fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<String, String> {
        self.format_text(file_path, file_text)
    }
}
