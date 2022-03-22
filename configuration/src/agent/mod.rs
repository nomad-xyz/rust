//! Agent configuration (logging, intervals, addresses, etc).
//!
//! All structs defined in this module include public data only. The real agent
//! settings blocks are separate/different from these {Agent}Config blocks and
//! can contain signers. Functionality of these config blocks is minimized to
//! just the data itself.

mod logging;
pub use logging::*;

pub mod kathy;
pub mod processor;
pub mod relayer;
pub mod updater;
pub mod watcher;

use std::path::PathBuf;

use self::{
    kathy::KathyConfig, processor::ProcessorConfig, relayer::RelayerConfig, updater::UpdaterConfig,
    watcher::WatcherConfig,
};

/// Full agent configuration
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// RPC specifier
    pub rpc_style: RpcStyles,
    /// Path to the DB
    pub db: PathBuf,
    /// Metrics port
    pub metrics: Option<u16>,
    /// Logging configuration
    pub logging: LogConfig,
    /// Updater configuration
    pub updater: UpdaterConfig,
    /// Relayer configuration
    pub relayer: RelayerConfig,
    /// Processor configuration
    pub processor: ProcessorConfig,
    /// Watcher configuration
    pub watcher: WatcherConfig,
    /// Kathy configuration
    pub kathy: KathyConfig,
}

#[macro_export]
/// Creates agent config block on that comes with interval and enabled by
/// default
macro_rules! decl_config {
    (
        $name:ident {
            $($(#[$tags:meta])* $prop:ident: $type:ty,)*
        }
    ) => {
        affix::paste! {
            #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[doc = "Config for `" $name]
            #[allow(missing_copy_implementations)]
            pub struct [<$name Config>] {
                $(
                    $(#[$tags])*
                    pub $prop: $type,
                )*
                /// Agent interval
                pub interval: u64,
                /// Whether or not agent is enabled
                pub enabled: bool,
            }
        }
    }
}
