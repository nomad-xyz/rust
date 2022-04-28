use gelato_relay::GelatoClient;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::providers::Middleware;
use crate::report_tx;

use std::sync::Arc;

pub enum ChainSubmitter {
    Local(Arc<Middleware>),
    Gelato(GelatoClient)
}

impl ChainSubmitter {
    pub async fn submit_to_chain(&self, tx: ContractCall<Middleware, ()>) -> Result<(), ChainCommunicationError> {
        match self {
            ChainSubmitter::Local(client) => {
                report_tx!(tx, client).try_into()
            }
            ChainSubmitter::Gelato(client) => {
                client.send_relay_transaction(
                    chain_id, // translate util
                    dest, // contract address
                    data, // TypedTransaction
                    token, // configurable fee token
                    relayer_fee // configurable fee amount
                ).await?
            }
        }
    }
}