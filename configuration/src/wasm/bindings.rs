//! Functions exported to Wasm

use wasm_bindgen::prelude::*;

use eyre::WrapErr;

use crate::wasm::types::*;

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
        config
    }};
}

macro_rules! to_js_val {
    ($item:expr) => {
        JsValue::from_serde(&$item)
            .map(Into::into)
            .wrap_err("Error serializing value for return to Javascript")
            .map_err(format_errs)
    };
}

macro_rules! ret_config {
    ($config:expr) => {
        to_js_val!($config)
    };
}

type JsResult<T> = std::result::Result<T, JsValue>;

/// Convert any display type into a string for javascript errors
fn format_errs(e: impl std::fmt::Display) -> wasm_bindgen::prelude::JsValue {
    format!("{:#}", e).into()
}

/// Get a built-in config
#[wasm_bindgen(js_name = getBuiltin)]
pub fn get_builtin(name: &str) -> JsResult<NomadConfig> {
    ret_config!(crate::builtin::get_builtin(name)
        .ok_or_else(|| eyre::eyre!("No builtin config found for environment named {}", name))
        .map_err(format_errs)?
        .clone())
}

/// Syntactically validate a config. Throw an error if invalid
#[wasm_bindgen(js_name = validateConfig)]
pub fn validate_config(val: &NomadConfig) -> JsResult<JsValue> {
    deser_config!(val);
    Ok(JsValue::NULL)
}

/// Make a new blank config
#[wasm_bindgen(js_name = blankConfig)]
pub fn blank_config() -> NomadConfig {
    to_js_val!(crate::NomadConfig::default()).unwrap()
}

/// Parse a json string into a config
#[wasm_bindgen(js_name = configFromString)]
pub fn config_from_string(s: &str) -> JsResult<NomadConfig> {
    let config = serde_json::from_str::<crate::NomadConfig>(s)
        .wrap_err("Unable to deserialize config from string")
        .map_err(format_errs)?;

    ret_config!(config)
}

/// Add a network to the config
#[wasm_bindgen(js_name = addNetwork)]
pub fn add_domain(config: &NomadConfig, domain: &Domain) -> JsResult<NomadConfig> {
    let mut config = deser_config!(config);
    let domain = deser!(domain, crate::network::Domain);
    config.add_domain(domain).map_err(format_errs)?;
    ret_config!(config)
}

/// Add a network to the config
#[wasm_bindgen(js_name = addCore)]
pub fn add_core(config: &NomadConfig, name: &str, core: &CoreContracts) -> JsResult<NomadConfig> {
    let mut config = deser_config!(config);
    let core = deser!(core, crate::contracts::CoreContracts);
    config.add_core(name, core).map_err(format_errs)?;
    ret_config!(config)
}

/// Add a network to the config
#[wasm_bindgen(js_name = addBridge)]
pub fn add_bridge(
    config: &NomadConfig,
    name: &str,
    bridge: &BridgeContracts,
) -> JsResult<NomadConfig> {
    let mut config = deser_config!(config);
    let bridge = deser!(bridge, crate::bridge::BridgeContracts);
    config.add_bridge(name, bridge).map_err(format_errs)?;
    ret_config!(config)
}
