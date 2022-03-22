use wasm_bindgen::wasm_bindgen;

#[wasm_bindgen]
pub struct Tree(crate::Tree<32>);
