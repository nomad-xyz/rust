pub mod avail_subxt_config;
pub use avail_subxt_config::{avail, AvailConfig};

pub mod home;

#[macro_use]
pub mod macros;

pub type SubstrateSigner<T: subxt::Config> = dyn subxt::tx::Signer<T> + Send + Sync;

codegen_home!(Avail, avail, AvailConfig);
