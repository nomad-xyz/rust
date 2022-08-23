//! Interfaces to the substrate chains

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

/// Substrate home
pub mod home;
pub use home::*;

/// Substrate replica
pub mod replica;
pub use replica::*;

/// Substrate xapp connection manager
pub mod xapp;
pub use xapp::*;

mod avail_subxt_config;
pub use avail_subxt_config::{avail, AvailConfig};

mod nomad_core;
pub use crate::nomad_core::*;

mod nomad_base;
pub use nomad_base::*;

mod client;
pub use client::*;

mod signer;
pub use signer::*;

#[macro_use]
mod macros;
pub use macros::*;

mod utils;
pub use utils::*;

mod error;
pub use error::*;

use ::nomad_core::Home;
use std::str::FromStr;
use subxt::Config;

#[derive(Debug, Clone)]
pub(crate) enum SubstrateChains {
    Avail,
}

impl FromStr for SubstrateChains {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "a" => Ok(Self::Avail),
            _ => panic!("Unknown substrate chain"),
        }
    }
}

/// Substrate signer
pub type SubstrateSigner<T> = dyn subxt::tx::Signer<T> + Send + Sync;

boxed_object!(
    make_avail_home,
    Avail,
    SubstrateHome,
    Home<Error = SubstrateError>,
);

/// Make substrate home object
pub async fn make_home<T: Config>(
    conn: nomad_xyz_configuration::Connection,
    name: &str,
    domain: u32,
    submitter_conf: Option<nomad_xyz_configuration::substrate::TxSubmitterConf>,
    timelag: Option<u8>,
) -> color_eyre::Result<Box<dyn Home<Error = SubstrateError>>> {
    let chain: SubstrateChains = name
        .parse()
        .unwrap_or_else(|_| panic!("Unrecognized chain name: {}", name));

    match chain {
        SubstrateChains::Avail => {
            make_avail_home(conn, name, domain, submitter_conf, timelag).await
        }
    }
}
