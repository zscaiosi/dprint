use dprint_core::configuration::{ConfigurationDiagnostic, GlobalConfiguration};
use dprint_core::plugins::PluginInfo;
use wasmer_runtime::{Instance};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use super::super::super::types::ErrBox;
use super::{BytesTransmitter, WasmFunctions, FormatResult};

pub struct WasmPlugin<'a> {
    wasm_functions: Rc<WasmFunctions<'a>>,
    bytes_transmitter: BytesTransmitter<'a>,
}

impl<'a> WasmPlugin<'a> {
    pub fn new(instance: &'a Instance) -> Result<Self, ErrBox> {
        let wasm_functions = Rc::new(WasmFunctions::new(&instance)?);
        let bytes_transmitter = BytesTransmitter::new(wasm_functions.clone());

        Ok(WasmPlugin {
            wasm_functions,
            bytes_transmitter,
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
        let len = self.wasm_functions.get_plugin_info();
        let json_text = self.bytes_transmitter.receive_string(len);
        serde_json::from_str(&json_text).unwrap()
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
