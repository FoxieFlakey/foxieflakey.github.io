use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn call_to_rust() {
    web_sys::console::log_1(&"Hello world from Rust!".into());
}
