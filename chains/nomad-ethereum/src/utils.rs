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

#[cfg(test)]
mod test {
    use ethers::prelude::{TransactionReceipt, U64};

    use super::*;

    #[tokio::test]
    async fn turning_transaction_receipt_into_tx_outcome() {
        let receipt = TransactionReceipt {
            status: Some(U64::from(0)),
            ..Default::default()
        };
        let tx_outcome: Result<TxOutcome, EthereumError> =
            try_transaction_receipt_to_tx_outcome(receipt);
        assert!(
            tx_outcome.is_err(),
            "Turning failed transaction receipt into errored tx outcome not succeeded"
        );

        let receipt = TransactionReceipt {
            status: Some(U64::from(1)),
            ..Default::default()
        };
        let tx_outcome: Result<TxOutcome, EthereumError> =
            try_transaction_receipt_to_tx_outcome(receipt);
        assert!(
            tx_outcome.is_ok(),
            "Turning succeeded transaction receipt into successful tx outcome not succeeded"
        );
    }
}
