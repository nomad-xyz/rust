use wasm_bindgen::prelude::*;

use crate::NomadConfig;
use eyre::WrapErr;

macro_rules! deser {
    ($val:ident, $expected:ty) => {{
        let val = $val
            .into_serde::<$expected>()
            .wrap_err(format!(
                "Error while deserializing Javascript object to {}",
                stringify!($expected)
            ))
            .map_err(format_errs)?;
        val
    }};
}

macro_rules! deser_config {
    ($val:ident) => {{
        let config = deser!($val, crate::NomadConfig);
        config.validate().map_err(format_errs)?;
        config
    }};
}

type JsResult<T> = std::result::Result<T, wasm_bindgen::prelude::JsValue>;

/// Convert any display type into a string for javascript errors
fn format_errs(e: impl std::fmt::Display) -> wasm_bindgen::prelude::JsValue {
    format!("{}", e).into()
}

#[wasm_bindgen(js_name = validateConfig)]
pub fn validate_config(val: &JsValue) -> JsResult<()> {
    deser_config!(val);
    Ok(())
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
    let mut config = deser_config!(config);
    let network = deser!(network, crate::core_deploy::CoreNetwork);

    config.networks.insert(network.name.to_owned());
    config
        .core
        .networks
        .insert(network.name.to_owned(), network);

    config.validate().map_err(format_errs)?;
    todo!()
}
