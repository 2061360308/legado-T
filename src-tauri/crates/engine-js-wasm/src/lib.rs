mod cookie;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub fn engine_wasm_native_placeholder() {}
