use ethers::types::transaction::eip2718::TypedTransaction;
use nomad_core::Signers;
use nomad_xyz_configuration::{agent::SignerConf, ethereum::Connection};
use crate::http_signer_middleware;
use std::{collections::HashMap, sync::Arc};
use color_eyre::Result;
use ethers::prelude::*;

/// Configuration for creating an ethers::SignerMiddleware
pub struct SigningProviderConfig {
    /// Signer configuration
    pub signer: SignerConf,
    /// Connection configuration
    pub connection: Connection,
}

/// Component responsible for submitting transactions to the chain. Can 
/// sign/submit locally or use a transaction relay service.
pub enum EthChainSubmitter {
    /// Sign/submit txs locally
    Local(HashMap<u32, SigningProviderConfig>),
}

impl EthChainSubmitter {
    /// Submit transaction to chain
    pub async fn submit_to_chain(
        &self,
        domain: u32,
        tx: impl Into<TypedTransaction>,
    ) -> Result<()> {
        let tx: TypedTransaction = tx.into();

        match self {
            EthChainSubmitter::Local(client_map) => {
                let client_config = client_map.get(&domain).expect("!eth client");
                let signer = Signers::try_from_signer_conf(&client_config.signer).await?;

                let client = match &client_config.connection {
                    Connection::Http { url } => http_signer_middleware!(url, signer),
                    Connection::Ws { url: _ } => panic!("not supporting ws"),
                };

                let dispatched = client.send_transaction(tx, None).await?;
                let tx_hash: ethers::core::types::H256 = *dispatched;
                let result = dispatched
                    .await?
                    .ok_or_else(|| nomad_core::ChainCommunicationError::DroppedError(tx_hash))?;

                tracing::info!(
                    "confirmed transaction with tx_hash {:?}",
                    result.transaction_hash
                );

                Ok(())
            }
            // ChainSubmitter::Gelato(client) => {
            //     client.send_relay_transaction(
            //         chain_id, // translate util
            //         dest, // contract address
            //         data, // TypedTransaction
            //         token, // configurable fee token
            //         relayer_fee // configurable fee amount
            //     ).await?
            // }
        }
    }
}
