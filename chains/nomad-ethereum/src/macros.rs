/// Dispatches a transaction, logs the tx id, and returns the result
#[allow(unused_macros)]
macro_rules! report_tx {
    ($tx:expr, $provider:expr, $($tail:tt)*) => {{
        // Simple switch between 2 implementations:
        //  * @escalating for new implementation with gas escalation
        //  * @legacy for simple transaction send implementation
        report_tx!(@legacy $tx, $provider, $($tail)*)
    }};

    // Escalating cutting edge implementation, but we have to put it aside, while there is a bug
    (@escalating $tx:expr, $provider:expr) => {{
        log_tx_details!($tx);

        let dispatch_fut = $provider.send_escalating(
            &$tx,
            5,
            Box::new(|original, index| original * (index + 1))
        );

        let escalator = dispatch_fut
            .await
            .map_err(Box::new)
            .map_err(|e| ChainCommunicationError::CustomError(e))?
            .with_broadcast_interval(std::time::Duration::from_secs(60));

        let result = escalator
            .await?;

        tracing::info!(
            tx_hash = ?result.transaction_hash,
            "Confirmed transaction",
        );

        result
    }};

    // Legacy way of sending transactions.
    (@legacy $tx:expr, $provider:expr,) => {{
        log_tx_details!($tx);

        let dispatched = $provider
            .send_transaction($tx, None)
            .await
            .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))?;

        let tx_hash: ethers::core::types::H256 = *dispatched;
        let result = dispatched
            .await?
            .ok_or_else(|| nomad_core::ChainCommunicationError::DroppedError(tx_hash))?;

        tracing::info!(
            tx_hash = ?tx_hash,
            "Confirmed transaction",
        );

        result
    }};
}

#[allow(unused_macros)]
macro_rules! log_tx_details {
    ($tx:expr) => {
        // "0x..."
        let data = format!("0x{}", hex::encode(&$tx.data().map(|b| b.to_vec()).unwrap_or_default()));

        let to = $tx.to().cloned().unwrap_or_else(|| ethers::types::NameOrAddress::Address(Default::default()));

        tracing::info!(
            to = ?to,
            data = %data,
            "Dispatching transaction"
        );
    };
}

macro_rules! boxed_indexer {
    (@timelag $provider:expr, $abi:ident, $timelag:ident, $($tail:tt)*) => {{
        if let Some(lag) = $timelag {
            let provider: Arc<_> = ethers::middleware::TimeLag::new($provider, lag).into();
            Box::new(crate::$abi::new(provider, $($tail)*))
        } else {
            Box::new(crate::$abi::new($provider, $($tail)*))
        }
    }};
    (@ws $url:expr, $($tail:tt)*) => {{
        let ws = ethers::providers::Ws::connect($url).await?;
        let provider = Arc::new(ethers::providers::Provider::new(ws));
        boxed_indexer!(@timelag provider, $($tail)*)
    }};
    (@http $url:expr, $($tail:tt)*) => {{
        let provider: crate::retrying::RetryingProvider<ethers::providers::Http> = $url.parse()?;
        let provider = Arc::new(ethers::providers::Provider::new(provider));
        boxed_indexer!(@timelag provider, $($tail)*)
    }};
    ($name:ident, $abi:ident, $trait:ident, $($n:ident:$t:ty),*)  => {
        #[doc = "Cast a contract locator to a live contract handle"]
        pub async fn $name(conn: nomad_xyz_configuration::chains::ethereum::Connection, locator: &ContractLocator, timelag: Option<u8>, $($n:$t),*) -> color_eyre::Result<Box<dyn $trait>> {
            let b: Box<dyn $trait> = match conn {
                nomad_xyz_configuration::chains::ethereum::Connection::Http (url) => {
                    boxed_indexer!(@http url, $abi, timelag, locator, $($n),*)
                }
                nomad_xyz_configuration::chains::ethereum::Connection::Ws (url) => {
                    boxed_indexer!(@ws url, $abi, timelag, locator, $($n),*)
                }
            };
            Ok(b)
        }
    };
}

/// Create base http retrying provider
#[macro_export]
macro_rules! http_provider {
    ($url:expr) => {{
        let provider: crate::retrying::RetryingProvider<ethers::providers::Http> = $url.parse()?;
        Arc::new(ethers::providers::Provider::new(provider))
    }};
}

/// Create base ws provider
#[macro_export]
macro_rules! ws_provider {
    ($url:expr) => {{
        let ws = ethers::providers::Ws::connect($url).await?;
        Arc::new(ethers::providers::Provider::new(ws))
    }};
}

/// Create ethers::SignerMiddleware from websockets connection
#[macro_export]
macro_rules! wrap_with_signer {
    ($provider:expr, $signer:expr) => {{
        // First set the chain ID locally
        let provider_chain_id = $provider.get_chainid().await?;
        let signer = ethers::signers::Signer::with_chain_id($signer, provider_chain_id.as_u64());

        // Manage the nonce locally
        let address = ethers::prelude::Signer::address(&signer);
        let provider =
            ethers::middleware::nonce_manager::NonceManagerMiddleware::new($provider, address);

        // Kludge. Increase the gas by multiplication of every estimated gas by
        // 2, except the gas for chain id 1 (Ethereum Mainnet)
        let provider = crate::gas::GasAdjusterMiddleware::with_default_policy(
            provider,
            provider_chain_id.as_u64(),
        );

        // Manage signing locally
        Arc::new(ethers::middleware::SignerMiddleware::new(provider, signer))
    }};
}

/// Create TxSubmitter::Local
#[macro_export]
macro_rules! tx_submitter_local {
    ($base_provider:expr, $signer_conf:ident) => {{
        let signer = nomad_core::Signers::try_from_signer_conf(&$signer_conf).await?;
        let signing_provider: Arc<_> = wrap_with_signer!($base_provider.clone(), signer);
        TxSubmitter::new(signing_provider.into())
    }};
}

/// Create TxSubmitter::Gelato
#[macro_export]
macro_rules! tx_submitter_gelato {
    ($base_provider:expr, $gelato_conf:ident) => {{
        let signer = nomad_core::Signers::try_from_signer_conf(&$gelato_conf.sponsor).await?;
        let sponsor = signer.clone();
        let chain_id = $base_provider.get_chainid().await?.as_u64();
        let signing_provider: Arc<_> = wrap_with_signer!($base_provider.clone(), signer); // kludge: only using signing provider for type consistency with TxSubmitter::Local

        let client = SingleChainGelatoClient::with_default_url(
            signing_provider,
            sponsor,
            chain_id,
            $gelato_conf.fee_token.parse::<H160>().expect("invalid gelato fee token"),
            false,
        );
        TxSubmitter::new(client.into())
    }};
}

macro_rules! boxed_contract {
    (@timelag $base_provider:expr, $submitter:expr, $abi:ident, $timelag:ident, $($tail:tt)*) => {{
        if let Some(lag) = $timelag {
            let read_provider: Arc<_> = ethers::middleware::TimeLag::new($base_provider, lag).into();
            Box::new(crate::$abi::new($submitter, read_provider, $($tail)*))
        } else {
            Box::new(crate::$abi::new($submitter, $base_provider, $($tail)*))
        }
    }};
    (@submitter $base_provider:expr, $submitter_conf:ident, $($tail:tt)*) => {{
        if let Some(conf) = $submitter_conf {
            let submitter = match conf {
                nomad_xyz_configuration::ethereum::TxSubmitterConf::Local(signer_conf) => {
                    tx_submitter_local!($base_provider, signer_conf)
                }
                nomad_xyz_configuration::ethereum::TxSubmitterConf::Gelato(gelato_conf) => {
                    tx_submitter_gelato!($base_provider, gelato_conf)
                }
            };

            boxed_contract!(@timelag $base_provider, submitter, $($tail)*)
        } else {
            panic!("Not supporting contracts with tx submitter"); // TODO: allow readonly contracts?
        }
    }};
    (@ws $url:expr, $($tail:tt)*) => {{
        let provider = ws_provider!($url);
        boxed_contract!(@submitter provider, $($tail)*)
    }};
    (@http $url:expr, $($tail:tt)*) => {{
        let provider = http_provider!($url);
        boxed_contract!(@submitter provider, $($tail)*)
    }};
    ($name:ident, $abi:ident, $trait:ident, $($n:ident:$t:ty),*)  => {
        #[doc = "Cast a contract locator to a live contract handle"]
        pub async fn $name(conn: nomad_xyz_configuration::ethereum::Connection, locator: &ContractLocator, submitter_conf: Option<nomad_xyz_configuration::ethereum::TxSubmitterConf>, timelag: Option<u8>, $($n:$t),*) -> color_eyre::Result<Box<dyn $trait>> {
            let b: Box<dyn $trait> = match conn {
                nomad_xyz_configuration::chains::ethereum::Connection::Http (url) => {
                    boxed_contract!(@http url, submitter_conf, $abi, timelag, locator, $($n),*)
                }
                nomad_xyz_configuration::chains::ethereum::Connection::Ws (url) => {
                    boxed_contract!(@ws url, submitter_conf, $abi, timelag, locator, $($n),*)
                }
            };
            Ok(b)
        }
    };
}
