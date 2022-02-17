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
        let config = deser!($val, NomadConfig)
            .chained_validate()
            .map_err(format_errs)?;
        config
    }};
}

macro_rules! to_js_val {
    ($item:expr) => {
        JsValue::from_serde(&$item)
            .wrap_err("Error serializing value for return to Javascript")
            .map_err(format_errs)
    };
}

macro_rules! ret_config {
    ($config:expr) => {
        to_js_val!($config.chained_validate().map_err(format_errs)?)
    };
}

type JsResult<T> = std::result::Result<T, wasm_bindgen::prelude::JsValue>;

/// Convert any display type into a string for javascript errors
fn format_errs(e: impl std::fmt::Display) -> wasm_bindgen::prelude::JsValue {
    format!("{:#}", e).into()
}

/// Syntactically validate a config. Throw an error if invalid
#[wasm_bindgen(js_name = validateConfig)]
pub fn validate_config(val: &JsValue) -> JsResult<()> {
    deser_config!(val);
    Ok(())
}

/// Make a new blank config
#[wasm_bindgen(js_name = blankConfig)]
pub fn blank_config() -> JsValue {
    to_js_val!(NomadConfig::default()).unwrap()
}

/// Parse a json string into a config
#[wasm_bindgen(js_name = configFromString)]
pub fn config_from_string(s: &str) -> JsResult<JsValue> {
    let config = serde_json::from_str::<crate::NomadConfig>(s)
        .wrap_err("Unable to deserialize config from string")
        .map_err(format_errs)?;

    ret_config!(config)
}

/// Add a network to the config
#[wasm_bindgen(js_name = addNetwork)]
pub fn add_network(config: &JsValue, network: &JsValue) -> JsResult<JsValue> {
    let mut config = deser_config!(config);
    let network = deser!(network, crate::core_deploy::CoreNetwork);
    config.add_network(network).map_err(format_errs)?;
    ret_config!(config)
}

/// Add a bridge to a config
#[wasm_bindgen(js_name = addBridge)]
pub fn add_bridge(config: &JsValue, name: &str, bridge: &JsValue) -> JsResult<JsValue> {
    let mut config = deser_config!(config);
    let bridge = deser!(bridge, crate::contracts::BridgeContracts);
    config.add_bridge(name, bridge).map_err(format_errs)?;
    ret_config!(config)
}
