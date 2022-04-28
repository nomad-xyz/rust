//! Interfaces to the ethereum contracts

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

use color_eyre::eyre::Result;
use ethers::prelude::*;
use nomad_core::*;
use nomad_xyz_configuration::{
    chains::ethereum::Connection, ConnectionManagerGasLimits, HomeGasLimits, ReplicaGasLimits,
};
use num::Num;
use std::sync::Arc;

#[macro_use]
mod macros;

/// Retrying Provider
mod retrying;
pub use retrying::{RetryingProvider, RetryingProviderError};

/// Chain submitter
mod chain_submitter;
pub use chain_submitter::*;

/// Contract binding
#[cfg(not(doctest))]
pub(crate) mod bindings;

/// Home abi
#[cfg(not(doctest))]
mod home;

/// Replica abi
#[cfg(not(doctest))]
mod replica;

/// XAppConnectionManager abi
#[cfg(not(doctest))]
mod xapp;

/// Gas increasing Middleware
mod gas;

#[cfg(not(doctest))]
pub use crate::{home::*, replica::*, xapp::*};

#[allow(dead_code)]
/// A live connection to an ethereum-compatible chain.
pub struct Chain {
    creation_metadata: Connection,
    ethers: ethers::providers::Provider<ethers::providers::Http>,
}

boxed_indexer!(
    make_home_indexer,
    EthereumHomeIndexer,
    HomeIndexer,
    from_height: u32,
    chunk_size: u32
);
boxed_indexer!(
    make_replica_indexer,
    EthereumReplicaIndexer,
    CommonIndexer,
    from_height: u32,
    chunk_size: u32
);

boxed_contract!(make_home, EthereumHome, Home, gas: Option<HomeGasLimits>);
boxed_contract!(
    make_replica,
    EthereumReplica,
    Replica,
    gas: Option<ReplicaGasLimits>
);
boxed_contract!(
    make_conn_manager,
    EthereumConnectionManager,
    ConnectionManager,
    gas: Option<ConnectionManagerGasLimits>
);

#[async_trait::async_trait]
impl nomad_core::Chain for Chain {
    async fn query_balance(&self, addr: nomad_core::Address) -> Result<nomad_core::Balance> {
        let balance = format!(
            "{:x}",
            self.ethers
                .get_balance(
                    NameOrAddress::Address(H160::from_slice(&addr.0[..])),
                    Some(BlockId::Number(BlockNumber::Latest))
                )
                .await?
        );

        Ok(nomad_core::Balance(num::BigInt::from_str_radix(
            &balance, 16,
        )?))
    }
}
