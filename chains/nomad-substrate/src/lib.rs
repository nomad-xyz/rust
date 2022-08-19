//! Interfaces to the substrate chains

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

/// Substrate home
pub mod home;
pub use home::*;

mod avail_subxt_config;
pub use avail_subxt_config::{avail, AvailConfig};

mod nomad_core;
pub use crate::nomad_core::*;

mod nomad_base;
pub use nomad_base::*;

#[macro_use]
mod macros;
pub use macros::*;

use subxt::ext::scale_value;

/// Substrate signer
pub type SubstrateSigner<T> = dyn subxt::tx::Signer<T> + Send + Sync;

/// Substrate-specific error wrapper
#[derive(Debug, thiserror::Error)]
pub enum SubstrateError {
    /// Substrate provider error
    #[error("{0}")]
    ProviderError(#[from] subxt::Error),
    /// Scale value deserialization error
    #[error("{0}")]
    DeserializationError(#[from] scale_value::serde::DeserializerError),
}
