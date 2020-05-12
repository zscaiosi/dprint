use wasmer_runtime::{Instance, Func, WasmPtr, Array, Ctx, Memory};

use super::super::super::types::ErrBox;

pub struct PluginWasmFunctions<'a> {
    instance: &'a Instance,
    context: &'a Ctx,
    memory: &'a Memory,
    get_wasm_memory_buffer_func: Func<'a, (), WasmPtr<u8, Array>>,
    get_wasm_memory_buffer_size_func: Func<'a, (), u32>,
    add_to_shared_bytes_from_buffer_func: Func<'a, u32>,
    set_buffer_with_shared_bytes_func: Func<'a, (u32, u32)>,
    clear_shared_bytes_func: Func<'a>,
}

impl<'a> PluginWasmFunctions<'a> {
    pub fn new(instance: &'a Instance) -> Result<Self, ErrBox> {
        let context = instance.context();
        Ok(PluginWasmFunctions {
            instance,
            context,
            memory: context.memory(0),
            get_wasm_memory_buffer_func: instance.exports.get("get_wasm_memory_buffer")?,
            get_wasm_memory_buffer_size_func: instance.exports.get("get_wasm_memory_buffer_size")?,
            add_to_shared_bytes_from_buffer_func: instance.exports.get("add_to_shared_bytes_from_buffer")?,
            set_buffer_with_shared_bytes_func: instance.exports.get("set_buffer_with_shared_bytes")?,
            clear_shared_bytes_func: instance.exports.get("clear_shared_bytes")?,
        })
    }

    pub fn get_memory(&self) -> &'a Memory {
        &self.memory
    }

    pub fn clear_shared_bytes(&self) {
        self.clear_shared_bytes_func.call().unwrap();
    }

    pub fn get_wasm_memory_buffer_size(&self) -> usize {
        self.get_wasm_memory_buffer_size_func.call().unwrap() as usize
    }

    pub fn get_wasm_memory_buffer_ptr(&self) -> WasmPtr<u8, Array> {
        self.get_wasm_memory_buffer_func.call().unwrap()
    }

    pub fn set_buffer_with_shared_bytes(&self, offset: usize, length: usize) {
        self.set_buffer_with_shared_bytes_func.call(offset as u32, length as u32).unwrap();
    }

    pub fn add_to_shared_bytes_from_buffer(&self, length: usize) {
        self.add_to_shared_bytes_from_buffer_func.call(length as u32).unwrap();
    }
}
