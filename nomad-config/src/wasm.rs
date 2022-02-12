use wasm_bindgen::prelude::*;

use crate::NomadConfig;
use eyre::WrapErr;

type JsResult<T> = std::result::Result<T, wasm_bindgen::prelude::JsValue>;

macro_rules! deser {
    ($val:ident, $expected:ty) => {{
        $val.into_serde::<$expected>()
            .wrap_err(format!(
                "Error while deserializing Javascript object to {}",
                stringify!($expected)
            ))
            .map_err(format_errs)
    }};
}

macro_rules! deser_config {
    ($val:ident) => {
        deser!($val, crate::NomadConfig)?
            .validate()
            .map_err(format_errs)?
    };
}

/// Convert any display type into a string for javascript errors
fn format_errs(e: impl std::fmt::Display) -> wasm_bindgen::prelude::JsValue {
    format!("{}", e).into()
}

#[wasm_bindgen(js_name = validateConfig)]
pub fn validate_config(val: &JsValue) -> JsResult<()> {
    Ok(deser_config!(val))
}

#[wasm_bindgen(js_name = blankConfig)]
pub fn new_config() -> JsValue {
    JsValue::from_serde(&NomadConfig::default()).unwrap()
}

#[wasm_bindgen(js_name = fromString)]
pub fn from_string(s: &str) -> JsResult<JsValue> {
    serde_json::from_str::<crate::NomadConfig>(s)
        .wrap_err("Unable to deserialize config from string")
        .map_err(format_errs)
        .map(|v| JsValue::from_serde(&v).map_err(format_errs))?
}

#[wasm_bindgen(js_name = addNetwork)]
pub fn add_network(config: &JsValue, network: &JsValue) -> JsResult<JsValue> {
    let config = deser_config!(config);

    todo!()
}
