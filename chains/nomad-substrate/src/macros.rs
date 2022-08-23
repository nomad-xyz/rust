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
        let successful_tx = utils::try_tx_in_block_to_successful_tx_events(tx_in_block).await?;

        info!(
            tx_hash = ?successful_tx.extrinsic_hash(),
            block_hash = ?successful_tx.block_hash(),
            "Confirmed {} tx success.",
            $method,
        );

        Ok(TxOutcome { txid: successful_tx.extrinsic_hash().into() })
    }}
}

/// Generate function that creates boxed non_signing object (i.e. an indexer)
macro_rules! boxed_indexer {
    ($fn_name:ident, $chain_name:ident, $abi:ident, $trait:path, $($n:ident:$t:ty),*)  => {
        affix::paste! {
            #[doc = "Cast a connection into a non-signing trait object"]
            pub(crate) async fn $fn_name(conn: nomad_xyz_configuration::Connection, _timelag: Option<u8>, $($n:$t),*) -> color_eyre::Result<Box<dyn $trait>> {
                let api = match conn {
                    nomad_xyz_configuration::Connection::Http(url) =>
                        subxt::OnlineClient::<[<$chain_name Config>]>::from_url(url).await?,
                    nomad_xyz_configuration::Connection::Ws(url) =>
                        subxt::OnlineClient::<[<$chain_name Config>]>::from_url(url).await?,
                };

                Ok(Box::new($abi::<[<$chain_name Config>]>::new(api.into())))
            }
        }
    }
}

/// Generate function that creates boxed signing object (home, replica,
/// connection manager)
macro_rules! boxed_signing_object {
    ($fn_name:ident, $chain_name:ident, $abi:ident, $trait:path, $($n:ident:$t:ty),*)  => {
        affix::paste! {
            #[doc = "Cast a connection into a signing trait object"]
            pub(crate) async fn $fn_name(conn: nomad_xyz_configuration::Connection, name: &str, domain: u32, submitter_conf: Option<nomad_xyz_configuration::substrate::TxSubmitterConf>, _timelag: Option<u8>, $($n:$t),*) -> color_eyre::Result<Box<dyn $trait>> {
                let api = match conn {
                    nomad_xyz_configuration::Connection::Http(url) =>
                        subxt::OnlineClient::<[<$chain_name Config>]>::from_url(url).await?,
                    nomad_xyz_configuration::Connection::Ws(url) =>
                        subxt::OnlineClient::<[<$chain_name Config>]>::from_url(url).await?,
                };

                let signer = if let Some(conf) = submitter_conf {
                    use ::nomad_core::FromSignerConf;

                    match conf {
                        nomad_xyz_configuration::substrate::TxSubmitterConf::Local(signer_conf) => {
                            SubstrateSigners::<[<$chain_name Config>], subxt::ext::sp_core::ecdsa::Pair>::try_from_signer_conf(&signer_conf)
                                .await?
                        }
                    }
                } else {
                    panic!("Not supporting connected objects without tx submission")
                };

                Ok(Box::new($abi::<[<$chain_name Config>]>::new(
                    api.into(),
                    std::sync::Arc::new(signer),
                    domain,
                    name,
                )))
            }
        }
    }
}
