use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;

#[wasm_bindgen]
pub fn call_to_rust() {
    console::log_1(&"Hello world from Rust!".into());
    web_sys::window()
        .expect("Cannot find current JS window")
        .document()
        .expect("Cannot get window.document")
        .get_element_by_id("output")
        .expect("Cannot find element by ID of 'output'")
        .set_text_content(Some("Hewwo From Rust!! hehe"));
}
