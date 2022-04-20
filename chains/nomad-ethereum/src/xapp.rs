#![allow(clippy::enum_variant_names)]
#![allow(missing_docs)]

use async_trait::async_trait;
use nomad_core::*;
use nomad_types::NomadIdentifier;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::bindings::xappconnectionmanager::XAppConnectionManager as EthereumConnectionManagerInternal;

use crate::report_tx;

/// A reference to a XAppConnectionManager contract on some Ethereum chain
#[derive(Debug)]
pub struct EthereumConnectionManager<W, R>
where
    W: ethers::providers::Middleware + 'static,
    R: ethers::providers::Middleware + 'static,
{
    write_contract: Arc<EthereumConnectionManagerInternal<W>>,
    read_contract: Arc<EthereumConnectionManagerInternal<R>>,
    domain: u32,
    name: String,
    _phantom: PhantomData<W>,
}

impl<W, R> EthereumConnectionManager<W, R>
where
    W: ethers::providers::Middleware + 'static,
    R: ethers::providers::Middleware + 'static,
{
    /// Create a reference to a XAppConnectionManager at a specific Ethereum
    /// address on some chain
    #[allow(dead_code)]
    pub fn new(
        write_provider: Arc<W>,
        read_provider: Arc<R>,
        ContractLocator {
            name,
            domain,
            address,
        }: &ContractLocator,
    ) -> Self {
        Self {
            write_contract: Arc::new(EthereumConnectionManagerInternal::new(
                address.as_ethereum_address().expect("!eth address"),
                write_provider.clone(),
            )),
            read_contract: Arc::new(EthereumConnectionManagerInternal::new(
                address.as_ethereum_address().expect("!eth address"),
                read_provider.clone(),
            )),
            domain: *domain,
            name: name.to_owned(),
            _phantom: Default::default(),
        }
    }
}

#[async_trait]
impl<W, R> ConnectionManager for EthereumConnectionManager<W, R>
where
    W: ethers::providers::Middleware + 'static,
    R: ethers::providers::Middleware + 'static,
{
    fn local_domain(&self) -> u32 {
        self.domain
    }

    #[tracing::instrument(err)]
    async fn is_replica(&self, address: NomadIdentifier) -> Result<bool, ChainCommunicationError> {
        Ok(self
            .read_contract
            .is_replica(address.as_ethereum_address().expect("!eth address"))
            .call()
            .await?)
    }

    #[tracing::instrument(err)]
    async fn watcher_permission(
        &self,
        address: NomadIdentifier,
        domain: u32,
    ) -> Result<bool, ChainCommunicationError> {
        Ok(self
            .read_contract
            .watcher_permission(address.as_ethereum_address().expect("!eth address"), domain)
            .call()
            .await?)
    }

    #[tracing::instrument(err)]
    async fn owner_enroll_replica(
        &self,
        replica: NomadIdentifier,
        domain: u32,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        let tx = self
            .write_contract
            .owner_enroll_replica(replica.as_ethereum_address().expect("!eth address"), domain);

        report_tx!(tx, &self.provider).try_into()
    }

    #[tracing::instrument(err)]
    async fn owner_unenroll_replica(
        &self,
        replica: NomadIdentifier,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        let tx = self
            .write_contract
            .owner_unenroll_replica(replica.as_ethereum_address().expect("!eth address"));

        report_tx!(tx, &self.provider).try_into()
    }

    #[tracing::instrument(err)]
    async fn set_home(&self, home: NomadIdentifier) -> Result<TxOutcome, ChainCommunicationError> {
        let tx = self
            .write_contract
            .set_home(home.as_ethereum_address().expect("!eth address"));

        report_tx!(tx, &self.provider).try_into()
    }

    #[tracing::instrument(err)]
    async fn set_watcher_permission(
        &self,
        watcher: NomadIdentifier,
        domain: u32,
        access: bool,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        let tx = self.write_contract.set_watcher_permission(
            watcher.as_ethereum_address().expect("!eth address"),
            domain,
            access,
        );

        report_tx!(tx, &self.provider).try_into()
    }

    #[tracing::instrument(err)]
    async fn unenroll_replica(
        &self,
        signed_failure: &SignedFailureNotification,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        let tx = self.write_contract.unenroll_replica(
            signed_failure.notification.home_domain,
            signed_failure.notification.updater.into(),
            signed_failure.signature.to_vec().into(),
        );

        report_tx!(tx, &self.provider).try_into()
    }
}
