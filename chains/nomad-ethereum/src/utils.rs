use ethers::prelude::TransactionReceipt;
use nomad_core::TxOutcome;

use crate::EthereumError;

/// Try to convert ethers `TransactionReceipt` into `TxOutcome`. We use this
/// function instead of `From<TransactionReceipt> for TxOutcome` because
/// TxOutcome belongs to `nomad-core`.
pub fn try_transaction_receipt_to_tx_outcome(
    receipt: TransactionReceipt,
) -> Result<TxOutcome, EthereumError> {
    if receipt.status.unwrap().low_u32() == 1 {
        Ok(TxOutcome {
            txid: receipt.transaction_hash,
        })
    } else {
        Err(EthereumError::TxNotExecuted(receipt.transaction_hash))
    }
}
