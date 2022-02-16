use crate::{agent, common, contracts, core_deploy};
use std::path;
use wasm_bindgen::prelude::*;

macro_rules! basic_impl {
    ($name:ident) => {
        #[wasm_bindgen::prelude::wasm_bindgen]
        impl $name {
            #[wasm_bindgen::prelude::wasm_bindgen(js_name = toJSON)]
            pub fn to_json(&self) -> wasm_bindgen::prelude::JsValue {
                wasm_bindgen::prelude::JsValue::from_serde(&self.0).unwrap()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = fromString)]
            pub fn from_json(json: &str) -> crate::wasm::wasm_utils::JsResult<$name> {
                let c = serde_json::from_str(json).map_err(crate::wasm::wasm_utils::format_errs)?;
                Ok(Self(c))
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = clone)]
            pub fn _clone(&self) -> $name {
                $name(self.0.clone())
            }
        }
    };
}

macro_rules! conversion_impl {
    ($module:ident::$name:ident) => {
        impl From<$module::$name> for $name {
            fn from(n: $module::$name) -> $name {
                Self(n)
            }
        }

        impl From<&$module::$name> for $name {
            fn from(n: &$module::$name) -> $name {
                Self(n.clone())
            }
        }

        impl From<$name> for $module::$name {
            fn from(f: $name) -> Self {
                f.0
            }
        }
    };
}

macro_rules! wrap_struct {
    (
        $(#[$outer:meta])*
        $module:ident::$name:ident
    ) => {
        $(#[$outer])*
        #[wasm_bindgen::prelude::wasm_bindgen(inspectable)]
        #[derive(Clone, Debug)]
        pub struct $name($module::$name);

        conversion_impl!($module::$name);
        basic_impl!($name);
    };
    (
        $(#[$outer:meta])*
        $module:ident::$name:ident + Default
    ) => {
        $(#[$outer])*
        #[wasm_bindgen::prelude::wasm_bindgen(inspectable)]
        #[derive(Clone, Debug, Default)]
        pub struct $name($module::$name);

        conversion_impl!($module::$name);
        basic_impl!($name);

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl $name {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new() -> $name {
                Default::default()
            }
        }
    };
}

macro_rules! decl_getter {
    ($name:ty, $prop:ident: $type:ty) => {
        affix::paste! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            impl $name {


                #[wasm_bindgen::prelude::wasm_bindgen(method, getter = [<$prop:camel>])]
                pub fn [<get_ $prop>](&self) -> $type {
                    self.0.$prop.clone().into()
                }
            }
        }
    };
}

macro_rules! decl_setter {
    ($name:ty, $prop:ident: $type:ty) => {
        affix::paste! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            impl $name {
                #[wasm_bindgen::prelude::wasm_bindgen(method, setter = [<$prop:camel>])]
                pub fn [<set_ $prop>](&mut self, prop: $type) {
                    self.0.$prop = prop.into();
                }
            }
        }
    };
}

macro_rules! impl_prop_access {
    ($name:ty, $($prop:ident: $type:ty,)+) => {
            $(
                decl_getter!($name, $prop: $type);
                decl_setter!($name, $prop: $type);
            )+
    };
}

pub(crate) mod wasm_utils {
    pub use eyre::WrapErr;
    pub use wasm_bindgen::prelude::*;
    pub type JsResult<T> = std::result::Result<T, wasm_bindgen::prelude::JsValue>;

    /// Convert any display type into a string for javascript errors
    pub(crate) fn format_errs(e: impl std::fmt::Display) -> wasm_bindgen::prelude::JsValue {
        format!("{}", e).into()
    }
}

wrap_struct!(path::PathBuf + Default);

wrap_struct!(
    /// NomadIdentifier
    common::NomadIdentifier
        + Default
);
wrap_struct!(
    /// NameOrDomain
    common::NameOrDomain
);

#[wasm_bindgen]
impl NameOrDomain {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        Self(crate::common::NameOrDomain::Name(name.to_owned()))
    }

    #[wasm_bindgen(js_name = "fromDomainNumber")]
    pub fn from_domain_number(number: u32) -> Self {
        Self(crate::common::NameOrDomain::Domain(number))
    }
}

wrap_struct!(
    /// RpcStyles
    agent::RpcStyles
        + Default
);

wrap_struct!(
    /// LogStyle
    agent::LogStyle
        + Default
);
wrap_struct!(
    /// LogLevel
    agent::LogLevel
        + Default
);
wrap_struct!(
    /// LogConfig
    agent::LogConfig
        + Default
);
impl_prop_access!(LogConfig, fmt: LogStyle, level: LogLevel,);

wrap_struct!(
    /// IndexConfig
    agent::IndexConfig
        + Default
);
impl_prop_access!(IndexConfig, from: u64, chunk: u64,);

wrap_struct!(
    /// BaseAgentConfig
    agent::BaseAgentConfig
        + Default
);
impl_prop_access!(BaseAgentConfig, enabled: bool, interval: u64,);

wrap_struct!(
    /// AgentConfig
    agent::AgentConfig
        + Default
);
impl_prop_access!(
    AgentConfig,
    rpc_style: RpcStyles,
    timelag: u64,
    db: PathBuf,
    logging: LogConfig,
    index: IndexConfig,
    updater: BaseAgentConfig,
    relayer: BaseAgentConfig,
    processor: BaseAgentConfig,
    watcher: BaseAgentConfig,
    kathy: BaseAgentConfig,
);

wrap_struct!(
    /// Proxy
    contracts::Proxy
        + Default
);
impl_prop_access!(
    Proxy,
    implementation: NomadIdentifier,
    proxy: NomadIdentifier,
    beacon: NomadIdentifier,
);

wrap_struct!(
    /// EvmCoreContracts
    contracts::EvmCoreContracts
        + Default
);

impl_prop_access!(
    EvmCoreContracts,
    upgrade_beacon_controller: NomadIdentifier,
    x_app_connection_manager: NomadIdentifier,
    updater_manager: NomadIdentifier,
    home: Proxy,
    // TODO: replicas
);

wrap_struct!(
    /// CoreContracts
    contracts::CoreContracts
        + Default
);

wrap_struct!(
    /// EvmBridgeContracts
    contracts::EvmBridgeContracts
        + Default
);
impl_prop_access!(
    EvmBridgeContracts,
    bridge_router: Proxy,
    token_registry: Proxy,
    bridge_token: Proxy,
    // eth_helper: Option<NomadIdentifier>,
);

wrap_struct!(
    /// BridgeContracts
    contracts::BridgeContracts
        + Default
);

wrap_struct!(
    /// Governor
    core_deploy::Governor
        + Default
);
impl_prop_access!(Governor, address: NomadIdentifier, domain: u64,);

wrap_struct!(
    /// Governance
    core_deploy::Governance
        + Default
);
impl_prop_access!(
    Governance,
    recovery_manager: NomadIdentifier,
    recovery_timelock: u64,
);

wrap_struct!(
    /// CoreNetwork
    core_deploy::CoreNetwork
        + Default
);
impl_prop_access!(
    CoreNetwork,
    name: String,
    domain: u64,
    // connections: Vec<String>,
    contracts: CoreContracts,
    governance: Governance,
    // updaters: Vec<NomadIdentifier>,
    // watchers: Vec<NomadIdentifier>,
    agents: AgentConfig,
);

wrap_struct!(
    /// CoreDeploy
    core_deploy::CoreDeploy
        + Default
);
impl_prop_access!(
    CoreDeploy,
    governor: Governor,
    // networks: HashMap<String, CoreNetwork>,
);

wrap_struct!(
    /// NomadConfig
    crate::NomadConfig
        + Default
);

impl_prop_access!(
    NomadConfig,
    environment: String,
    // networks: HashSet<String>,
    // rpcs: HashMap<String, HashSet<String>>,
    core: CoreDeploy,
    // bridge: HashMap<String, BridgeContracts>,
);
