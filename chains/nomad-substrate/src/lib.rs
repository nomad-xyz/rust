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

mod client;
pub use client::*;

#[macro_use]
mod macros;
pub use macros::*;

mod utils;
pub use utils::*;

mod error;
pub use error::*;

/// Substrate signer
pub type SubstrateSigner<T> = dyn subxt::tx::Signer<T> + Send + Sync;
