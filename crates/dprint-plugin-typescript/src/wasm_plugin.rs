use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Mutex;
use dprint_core::configuration::{ConfigurationDiagnostic, ResolveConfigurationResult, GlobalConfiguration};
use std::path::PathBuf;
use dprint_core::plugins::*;
use super::configuration::{Configuration, resolve_config};
use super::formatter::Formatter;

// todo: move all this code into a new crate and generate it here with a macro

// INFO AND CONFIGURATION

static mut GLOBAL_CONFIG: Option<GlobalConfiguration> = None;
static mut PLUGIN_CONFIG: Option<HashMap<String, String>> = None;
static mut RESOLVE_CONFIGURATION_RESULT: Option<ResolveConfigurationResult<Configuration>> = None;

#[no_mangle]
pub fn get_plugin_info() -> usize {
    let info_json = serde_json::to_string(&PluginInfo {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        config_keys: vec![String::from("typescript"), String::from("javascript")],
        file_extensions: vec![String::from("ts"), String::from("tsx"), String::from("js"), String::from("jsx")],
    }).unwrap();
    set_shared_bytes_str(info_json)
}

#[no_mangle]
pub fn get_resolved_config() -> usize {
    let json = serde_json::to_string(&get_resolved_config_result().config).unwrap();
    set_shared_bytes_str(json)
}

#[no_mangle]
pub fn get_config_diagnostics() -> usize {
    let json = serde_json::to_string(&get_resolved_config_result().diagnostics).unwrap();
    set_shared_bytes_str(json)
}

fn get_resolved_config_result<'a>() -> &'a ResolveConfigurationResult<Configuration> {
    unsafe {
        ensure_initialized();
        return RESOLVE_CONFIGURATION_RESULT.as_ref().unwrap();
    }
}

fn ensure_initialized() {
    unsafe {
        if RESOLVE_CONFIGURATION_RESULT.is_none() {
            if let Some(global_config) = GLOBAL_CONFIG.take() {
                if let Some(plugin_config) = PLUGIN_CONFIG.take() {
                    let config_result = resolve_config(plugin_config, &global_config);
                    let formatter = Formatter::new(config_result.config.clone());
                    RESOLVE_CONFIGURATION_RESULT.replace(config_result);
                    FORMATTER.replace(formatter);
                    return;
                }
            }

            panic!("Plugin must have global config and plugin config set before use.");
        }
    }
}

// FORMATTING

static mut FILE_PATH: Option<PathBuf> = None;
static mut FORMATTER: Option<Formatter> = None;
static mut FORMATTED_TEXT: Option<String> = None;
static mut ERROR_TEXT: Option<String> = None;

#[no_mangle]
pub fn set_file_path() {
    let text = take_string_from_shared_bytes();
    unsafe { FILE_PATH.replace(PathBuf::from(text)) };
}

#[no_mangle]
pub fn format() -> u8 {
    ensure_initialized();
    let formatter = unsafe { FORMATTER.as_ref().expect("Expected the formatter to be initialized.") };
    let file_path = unsafe { FILE_PATH.take().expect("Expected the file path to be set.") };
    let file_text = take_string_from_shared_bytes();

    let formatted_text = formatter.format_text(&file_path, &file_text);
    match formatted_text {
        Ok(formatted_text) => {
            if formatted_text == file_text {
                0 // no change
            } else {
                unsafe { FORMATTED_TEXT.replace(formatted_text) };
                1 // change
            }
        },
        Err(err_text) => {
            unsafe { ERROR_TEXT.replace(err_text) };
            2 // error
        }
    }
}

#[no_mangle]
pub fn get_formatted_text() -> usize {
    let formatted_text = unsafe { FORMATTED_TEXT.take().expect("Expected to have formatted text.") };
    set_shared_bytes_str(formatted_text)
}

#[no_mangle]
pub fn get_error_text() -> usize {
    let error_text = unsafe { ERROR_TEXT.take().expect("Expected to have error text.") };
    set_shared_bytes_str(error_text)
}

// INITIALIZATION

#[no_mangle]
pub fn set_global_config() {
    let text = take_string_from_shared_bytes();
    let global_config: GlobalConfiguration = serde_json::from_str(&text).unwrap();
    unsafe { GLOBAL_CONFIG.replace(global_config); }
}

#[no_mangle]
pub fn set_plugin_config() {
    let text = take_string_from_shared_bytes();
    let plugin_config: HashMap<String, String> = serde_json::from_str(&text).unwrap();
    unsafe { PLUGIN_CONFIG.replace(plugin_config); }
}

// LOW LEVEL SENDING AND RECEIVING

const WASM_MEMORY_BUFFER_SIZE: usize = 1024;
static mut WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [0; WASM_MEMORY_BUFFER_SIZE];
static mut SHARED_BYTES: Vec<u8> = Vec::new();

#[no_mangle]
pub fn get_plugin_schema_version() -> u32 {
    1 // version 1
}

#[no_mangle]
pub fn get_wasm_memory_buffer() -> *const u8 {
    unsafe { WASM_MEMORY_BUFFER.as_ptr() }
}

#[no_mangle]
pub fn get_wasm_memory_buffer_size() -> usize {
    WASM_MEMORY_BUFFER_SIZE
}

#[no_mangle]
pub fn add_to_shared_bytes_from_buffer(length: usize) {
    unsafe {
        SHARED_BYTES.extend(&WASM_MEMORY_BUFFER[..length])
    }
}

#[no_mangle]
pub fn set_buffer_with_shared_bytes(offset: usize, length: usize) {
    unsafe {
        let bytes = &SHARED_BYTES[offset..(offset+length)];
        &WASM_MEMORY_BUFFER[..length].copy_from_slice(bytes);
    }
}

#[no_mangle]
pub fn clear_shared_bytes(capacity: usize) {
    unsafe { SHARED_BYTES = Vec::with_capacity(capacity); }
}

fn take_string_from_shared_bytes() -> String {
    unsafe {
        let bytes = std::mem::replace(&mut SHARED_BYTES, Vec::with_capacity(0));
        String::from_utf8(bytes).unwrap()
    }
}

fn set_shared_bytes_str(text: String) -> usize {
    let length = text.len();
    unsafe { SHARED_BYTES = text.into_bytes() }
    length
}

/// TypeScript Dprint CLI Plugin.
pub struct TypeScriptPlugin {
    plugin: Mutex<RefCell<Option<InitializedTypeScriptPlugin>>>,
}

struct InitializedTypeScriptPlugin {
    resolve_config_result: ResolveConfigurationResult<Configuration>,
    formatter: Formatter,
}

impl TypeScriptPlugin {
    pub fn new() -> TypeScriptPlugin {
        TypeScriptPlugin {
            plugin: Mutex::new(RefCell::new(None)),
        }
    }
}

impl Plugin for TypeScriptPlugin {
    fn dispose(&self) {
        let plugin_lock = self.plugin.lock().unwrap();
        let mut plugin_cell = plugin_lock.borrow_mut();
        let result = plugin_cell.take();
        drop(result);
    }

    fn name(&self) -> &'static str { env!("CARGO_PKG_NAME") }
    fn version(&self) -> &'static str { env!("CARGO_PKG_VERSION") }
    fn config_keys(&self) -> Vec<&'static str> { vec!["typescript", "javascript"] }

    fn initialize(&self, plugin_config: HashMap<String, String>, global_config: &GlobalConfiguration) {
        let plugin_lock = self.plugin.lock().unwrap();
        let mut plugin_cell = plugin_lock.borrow_mut();
        let config_result = resolve_config(plugin_config, &global_config);
        plugin_cell.replace(InitializedTypeScriptPlugin {
            formatter: Formatter::new(config_result.config.clone()),
            resolve_config_result: config_result,
        });
    }

    fn should_format_file(&self, file_path: &PathBuf, _: &str) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            match String::from(ext).to_lowercase().as_ref() {
                "js" | "jsx" | "ts" | "tsx" => true,
                _ => false,
            }
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

    fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<String, String> {
        // todo: don't use a lock here... this doesn't work for parallelization
        let plugin_lock = self.plugin.lock().unwrap();
        let plugin_cell = plugin_lock.borrow();
        let plugin = plugin_cell.as_ref().expect("Plugin must be initialized before use.");
        plugin.formatter.format_text(file_path, file_text)
    }
}
