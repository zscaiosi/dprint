use wasmer_runtime::{Instance, Func, WasmPtr, Array, Memory};

use super::super::super::types::{Error, ErrBox};

const CURRENT_SCHEMA_VERSION: u32 = 1;

pub enum FormatResult {
    NoChange = 0,
    Change = 1,
    Error = 2,
}

impl From<u8> for FormatResult {
    fn from(orig: u8) -> Self {
        match orig {
            0 => FormatResult::NoChange,
            1 => FormatResult::Change,
            2 => FormatResult::Error,
            _ => unreachable!(),
        }
    }
}

pub struct WasmFunctions<'a> {
    memory: &'a Memory,
    // high level
    set_global_config_func: Func<'a>,
    set_plugin_config_func: Func<'a>,
    get_plugin_info_func: Func<'a, (), u32>,
    get_resolved_config_func: Func<'a, (), u32>,
    get_config_diagnostics_func: Func<'a, (), u32>,
    set_file_path_func: Func<'a>,
    format_func: Func<'a, (), u8>,
    get_formatted_text_func: Func<'a, (), u32>,
    get_error_text_func: Func<'a, (), u32>,
    // low level
    get_wasm_memory_buffer_func: Func<'a, (), WasmPtr<u8, Array>>,
    get_wasm_memory_buffer_size_func: Func<'a, (), u32>,
    add_to_shared_bytes_from_buffer_func: Func<'a, u32>,
    set_buffer_with_shared_bytes_func: Func<'a, (u32, u32)>,
    clear_shared_bytes_func: Func<'a, u32>,
}

impl<'a> WasmFunctions<'a> {
    pub fn new(instance: &'a Instance) -> Result<Self, ErrBox> {
        let context = instance.context();
        let plugin_schema_version_func: Func<(), u32> = instance.exports.get("get_plugin_schema_version")?;
        let plugin_schema_version = plugin_schema_version_func.call().unwrap();

        if plugin_schema_version != CURRENT_SCHEMA_VERSION {
            return Err(Error::new(&format!(
                "Invalid schema version: {} -- Expected: {}. This may indicate you should upgrade your Dprint cli",
                plugin_schema_version,
                CURRENT_SCHEMA_VERSION
            )));
        }

        Ok(WasmFunctions {
            memory: context.memory(0),
            // high level
            set_global_config_func: instance.exports.get("set_global_config")?,
            set_plugin_config_func: instance.exports.get("set_plugin_config")?,
            get_plugin_info_func: instance.exports.get("get_plugin_info")?,
            get_resolved_config_func: instance.exports.get("get_resolved_config")?,
            get_config_diagnostics_func: instance.exports.get("get_config_diagnostics")?,
            set_file_path_func: instance.exports.get("set_file_path")?,
            format_func: instance.exports.get("format")?,
            get_formatted_text_func: instance.exports.get("get_formatted_text")?,
            get_error_text_func: instance.exports.get("get_error_text")?,
            // low level
            get_wasm_memory_buffer_func: instance.exports.get("get_wasm_memory_buffer")?,
            get_wasm_memory_buffer_size_func: instance.exports.get("get_wasm_memory_buffer_size")?,
            add_to_shared_bytes_from_buffer_func: instance.exports.get("add_to_shared_bytes_from_buffer")?,
            set_buffer_with_shared_bytes_func: instance.exports.get("set_buffer_with_shared_bytes")?,
            clear_shared_bytes_func: instance.exports.get("clear_shared_bytes")?,
        })
    }

    #[inline]
    pub fn set_global_config(&self) {
        self.set_global_config_func.call().unwrap()
    }

    #[inline]
    pub fn set_plugin_config(&self) {
        self.set_plugin_config_func.call().unwrap()
    }

    #[inline]
    pub fn get_plugin_info(&self) -> usize {
        self.get_plugin_info_func.call().unwrap() as usize
    }

    #[inline]
    pub fn get_resolved_config(&self) -> usize {
        self.get_resolved_config_func.call().unwrap() as usize
    }

    #[inline]
    pub fn get_config_diagnostics(&self) -> usize {
        self.get_config_diagnostics_func.call().unwrap() as usize
    }

    #[inline]
    pub fn set_file_path(&self) {
        self.set_file_path_func.call().unwrap()
    }

    #[inline]
    pub fn format(&self) -> FormatResult {
        let result = self.format_func.call().unwrap();
        println!("{}", result);
        result.into()
    }

    #[inline]
    pub fn get_formatted_text(&self) -> usize {
        self.get_formatted_text_func.call().unwrap() as usize
    }

    #[inline]
    pub fn get_error_text(&self) -> usize {
        self.get_error_text_func.call().unwrap() as usize
    }

    #[inline]
    pub fn get_memory(&self) -> &'a Memory {
        &self.memory
    }

    #[inline]
    pub fn clear_shared_bytes(&self, capacity: usize) {
        self.clear_shared_bytes_func.call(capacity as u32).unwrap();
    }

    #[inline]
    pub fn get_wasm_memory_buffer_size(&self) -> usize {
        self.get_wasm_memory_buffer_size_func.call().unwrap() as usize
    }

    #[inline]
    pub fn get_wasm_memory_buffer_ptr(&self) -> WasmPtr<u8, Array> {
        self.get_wasm_memory_buffer_func.call().unwrap()
    }

    #[inline]
    pub fn set_buffer_with_shared_bytes(&self, offset: usize, length: usize) {
        self.set_buffer_with_shared_bytes_func.call(offset as u32, length as u32).unwrap();
    }

    #[inline]
    pub fn add_to_shared_bytes_from_buffer(&self, length: usize) {
        self.add_to_shared_bytes_from_buffer_func.call(length as u32).unwrap();
    }
}
