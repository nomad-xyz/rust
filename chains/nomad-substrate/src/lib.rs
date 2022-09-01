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

mod configs;
pub use configs::*;

mod nomad_core;
pub use crate::nomad_core::*;

mod nomad_base;
pub(crate) use nomad_base::*;

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

use ::nomad_core::{Home, HomeIndexer};
use std::str::FromStr;

boxed_signing_object!(
    make_avail_home,
    Avail,
    SubstrateHome,
    Home<Error = SubstrateError>,
);

boxed_indexer!(
    make_avail_home_indexer,
    Avail,
    SubstrateHomeIndexer,
    HomeIndexer,
);

#[derive(Debug, Clone)]
pub(crate) enum SubstrateChains {
    Avail,
}

impl FromStr for SubstrateChains {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "avail" => Ok(Self::Avail),
            _ => panic!("Unknown substrate chain: {}", s),
        }
    }
}

/// Substrate signer
pub type SubstrateSigner<T> = dyn subxt::tx::Signer<T> + Send + Sync;

/// Make substrate home object
pub async fn make_home(
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

/// Make substrate home object
pub async fn make_home_indexer(
    conn: nomad_xyz_configuration::Connection,
    name: &str,
    timelag: Option<u8>,
) -> color_eyre::Result<Box<dyn HomeIndexer>> {
    let chain: SubstrateChains = name
        .parse()
        .unwrap_or_else(|_| panic!("Unrecognized chain name: {}", name));

    match chain {
        SubstrateChains::Avail => make_avail_home_indexer(conn, timelag).await,
    }
}
