/// Dispatches an extrinsic, waits for inclusion, and logs details
#[macro_export]
macro_rules! report_tx {
    ($method:expr, $client:expr, $signer:expr, $tx:expr) => {{
        let pending_tx = $client
            .tx()
            .sign_and_submit_then_watch_default(&$tx, $signer.as_ref())
            .await?;

        info!(
            method = $method,
            tx_hash = ?pending_tx.extrinsic_hash(),
            "Dispatched {} tx, waiting for inclusion.",
            $method,
        );

        // TODO: can a tx deterministically revert here?
        let tx_in_block = pending_tx
            .wait_for_in_block()
            .await?;

        // Try to detect reverting txs that were submitted to chain
        let successful_tx = crate::utils::try_tx_in_block_to_successful_tx_events(tx_in_block).await?;

        info!(
            tx_hash = ?successful_tx.extrinsic_hash(),
            block_hash = ?successful_tx.block_hash(),
            "Confirmed {} tx success.",
            $method,
        );

        Ok(TxOutcome { txid: successful_tx.extrinsic_hash().into() })
    }}
}
