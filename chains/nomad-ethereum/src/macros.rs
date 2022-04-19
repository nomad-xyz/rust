/// Dispatches a transaction, logs the tx id, and returns the result
#[macro_export]
macro_rules! report_tx {
    ($tx:expr, $($tail:tt)*) => {{
        // Simple switch between 2 implementations:
        //  * @escalating for new implementation with gas escalation
        //  * @legacy for simple transaction send implementation
        report_tx!(@legacy $tx, $($tail)*)
    }};

    // Escalating cutting edge implementation, but we have to put it aside, while there is a bug
    (@escalating $tx:expr, $provider:expr) => {{
        log_tx_details!($tx);

        let dispatch_fut = $provider.send_escalating(
            &$tx.tx,
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
            "confirmed transaction with tx_hash {:?}",
            result.transaction_hash
        );

        result
    }};

    // Legacy way of sending transactions.
    (@legacy $tx:expr, $($tail:tt)*) => {{
        log_tx_details!($tx);

        let dispatch_fut = $tx.send();
        let dispatched = dispatch_fut.await?;

        let tx_hash: ethers::core::types::H256 = *dispatched;
        let result = dispatched
            .await?
            .ok_or_else(|| nomad_core::ChainCommunicationError::DroppedError(tx_hash))?;

        tracing::info!(
            "confirmed transaction with tx_hash {:?}",
            result.transaction_hash
        );

        result
    }};
}

macro_rules! log_tx_details {
    ($tx:expr) => {
        // "0x..."
        let data = format!("0x{}", hex::encode(&$tx.tx.data().map(|b| b.to_vec()).unwrap_or_default()));

        let to = $tx.tx.to().cloned().unwrap_or_else(|| ethers::types::NameOrAddress::Address(Default::default()));

        tracing::info!(
            to = ?to,
            data = %data,
            "Dispatching transaction"
        );
    };
}

macro_rules! boxed_trait {
    (@finish $provider:expr, $abi:ident, $signer:ident, $timelag:ident, $($tail:tt)*) => {{
        if let Some(signer) = $signer {
            // If there's a provided signer, we want to manage every aspect
            // locally

            // First set the chain ID locally
            let provider_chain_id = $provider.get_chainid().await?;
            let signer = ethers::signers::Signer::with_chain_id(signer, provider_chain_id.as_u64());

            // Manage the nonce locally
            let address = ethers::prelude::Signer::address(&signer);
            let provider =
                ethers::middleware::nonce_manager::NonceManagerMiddleware::new($provider, address);

            // Kludge. Increase the gas by multiplication of every estimated gas by 2
            // except the gas for chain id 1 (Ethereum Mainnet)
            let provider = crate::gas::GasAdjusterMiddleware::with_default_policy(provider, provider_chain_id.as_u64());

            // Manage signing locally
            let signing_provider = Arc::new(ethers::middleware::SignerMiddleware::new(provider, signer));

            let write_provider = Arc::new(ethers::middleware::TimeLag::new(signing_provider.clone(), 0));
            let read_provider = if let Some(lag) = $timelag {
                Arc::new(ethers::middleware::TimeLag::new(signing_provider, lag))
            } else {
                Arc::new(ethers::middleware::TimeLag::new(signing_provider, 0))
            };

            Box::new(crate::$abi::new(write_provider, read_provider, $($tail)*))
        } else {
            let write_provider = Arc::new(ethers::middleware::TimeLag::new($provider.clone(), 0));
            let read_provider = if let Some(lag) = $timelag {
                Arc::new(ethers::middleware::TimeLag::new($provider, lag))
            } else {
                Arc::new(ethers::middleware::TimeLag::new($provider, 0))
            };
            Box::new(crate::$abi::new(write_provider, read_provider, $($tail)*))
        }
    }};
    (@ws $url:expr, $($tail:tt)*) => {{
        let ws = ethers::providers::Ws::connect($url).await?;
        let provider = Arc::new(ethers::providers::Provider::new(ws));
        boxed_trait!(@finish provider, $($tail)*)
    }};
    (@http $url:expr, $($tail:tt)*) => {{
        let provider: crate::retrying::RetryingProvider<ethers::providers::Http> = $url.parse()?;
        let provider = Arc::new(ethers::providers::Provider::new(provider));
        boxed_trait!(@finish provider, $($tail)*)
    }};
    ($name:ident, $abi:ident, $trait:ident, $($n:ident:$t:ty),*)  => {
        #[doc = "Cast a contract locator to a live contract handle"]
        pub async fn $name(conn: nomad_xyz_configuration::chains::ethereum::Connection, locator: &ContractLocator, signer: Option<Signers>, timelag: Option<u8>, $($n:$t),*) -> color_eyre::Result<Box<dyn $trait>> {
            let b: Box<dyn $trait> = match conn {
                nomad_xyz_configuration::chains::ethereum::Connection::Http { url } => {
                    boxed_trait!(@http url, $abi, signer, timelag, locator, $($n),*)
                }
                nomad_xyz_configuration::chains::ethereum::Connection::Ws { url } => {
                    boxed_trait!(@ws url, $abi, signer, timelag, locator, $($n),*)
                }
            };
            Ok(b)
        }
    };
}
