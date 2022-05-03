//! Pre-set configs bundled with the lib

use std::collections::HashMap;

use eyre::Context;
use once_cell::sync::OnceCell;

use crate::NomadConfig;

// built-in config objects
static TEST_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/test.json"));
static DEVELOPMENT_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/configs/development.json"
));
static STAGING_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/staging.json"));
static PRODUCTION_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/configs/production.json"
));
static BUILTINS: OnceCell<HashMap<&'static str, OnceCell<NomadConfig>>> = OnceCell::new();

fn deser(name: &str, json: &str) -> NomadConfig {
    serde_json::from_str(json)
        .wrap_err_with(|| format!("Configuration {}.json is malformed", name))
        .unwrap()
}

/// Get a built-in config object
pub fn get_builtin(name: &str) -> Option<&NomadConfig> {
    let builtins = BUILTINS.get_or_init(|| {
        let mut map: HashMap<_, _> = Default::default();

        map.insert("test", Default::default());
        map.insert("development", Default::default());
        map.insert("staging", Default::default());
        map.insert("production", Default::default());
        map
    });

    Some(builtins.get(name)?.get_or_init(|| match name {
        "test" => deser("test", TEST_JSON),
        "development" => deser("development", DEVELOPMENT_JSON),
        "staging" => deser("staging", STAGING_JSON),
        "production" => deser("production", PRODUCTION_JSON),
        _ => panic!("unknown builtin {}", name),
    }))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_loads_builtins() {
//         dbg!(get_builtin("test"));
//     }

//     #[test]
//     fn test_validates() {
//         dbg!(get_builtin("test")
//             .expect("config not found")
//             .validate()
//             .expect("invalid config"));
//     }

//     #[test]
//     fn development_loads_builtins() {
//         dbg!(get_builtin("development"));
//     }

//     #[test]
//     fn development_validates() {
//         dbg!(get_builtin("development")
//             .expect("config not found")
//             .validate()
//             .expect("invalid config"));
//     }

//     #[test]
//     fn staging_loads_builtins() {
//         dbg!(get_builtin("staging"));
//     }

//     #[test]
//     fn staging_validates() {
//         dbg!(get_builtin("staging")
//             .expect("config not found")
//             .validate()
//             .expect("invalid config"));
//     }

//     #[test]
//     fn production_loads_builtins() {
//         dbg!(get_builtin("production"));
//     }

//     #[test]
//     fn production_validates() {
//         dbg!(get_builtin("production")
//             .expect("config not found")
//             .validate()
//             .expect("invalid config"));
//     }
// }
