use crate::http_signer_middleware;
use color_eyre::Result;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use nomad_core::Signers;
use nomad_xyz_configuration::ethereum::Connection;
use std::sync::Arc;

/// Configuration for creating an ethers::SignerMiddleware
pub struct SigningProviderConfig {
    /// Signer configuration
    pub signer: Signers,
    /// Connection configuration
    pub connection: Connection,
}

impl SigningProviderConfig {
    /// Instantiate new signing provider config
    pub fn new(signer: Signers, connection: Connection) -> Self {
        Self { signer, connection }
    }
}

/// Component responsible for submitting transactions to the chain. Can
/// sign/submit locally or use a transaction relay service.
pub enum ChainSubmitter {
    /// Sign/submit txs locally
    Local(SigningProviderConfig),
}

impl ChainSubmitter {
    /// Submit transaction to chain
    pub async fn submit_to_chain(
        &self,
        _domain: u32,
        _contract_address: Address,
        tx: impl Into<TypedTransaction>,
    ) -> Result<()> {
        let tx: TypedTransaction = tx.into();

        match self {
            ChainSubmitter::Local(config) => {
                let signer = config.signer.clone();
                let client = match &config.connection {
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
            } // ChainSubmitter::Gelato(client) => {
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
