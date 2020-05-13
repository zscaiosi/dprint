use super::super::super::types::ErrBox;

/// Compiles a WASM module.
pub fn compile(wasm_bytes: &[u8]) -> Result<Vec<u8>, ErrBox> {
    let compile_result = wasmer_runtime::compile(&wasm_bytes)?;
    let artifact = compile_result.cache();
    // they didn't implement Error so need to manually handle it here
    match artifact {
        Ok(artifact) => match artifact.serialize() {
            Ok(bytes) => Ok(bytes),
            Err(err) => err!("Error serializing wasm module: {:?}", err),
        },
        Err(err) => err!("Error caching wasm module: {:?}", err),
    }
}
