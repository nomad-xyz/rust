use ethers::types::transaction::eip2718::TypedTransaction;
use nomad_core::{PersistedTransaction, TxTranslator};

/// Implements PersistedTransaction to TypedTransaction
#[derive(Debug)]
pub struct EthereumTxTranslator {}

impl TxTranslator<TypedTransaction> for EthereumTxTranslator {
    fn convert(&self, tx: &PersistedTransaction) -> TypedTransaction {
        unimplemented!()
    }
}
