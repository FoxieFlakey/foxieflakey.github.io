use std::borrow::Cow;

use crate::config;

pub struct Wasm {}

impl Wasm {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, path: &str) -> Option<Cow<'static, [u8]>> {
        config::INITIAL_WASM_BLOBS
            .get(path)
            .map(|x| Cow::Borrowed(x.1))
    }
}
