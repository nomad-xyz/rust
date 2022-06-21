use ethers::types::transaction::eip2718::TypedTransaction;
use nomad_core::{PersistedTransaction, TxTranslator};

/// Implements PersistedTransaction to TypedTransaction
#[derive(Debug)]
pub struct EthereumTxTranslator {}

impl TxTranslator for EthereumTxTranslator {
    type Transaction = TypedTransaction;

    fn convert(&self, tx: &PersistedTransaction) -> TypedTransaction {
        unimplemented!()
    }
}
