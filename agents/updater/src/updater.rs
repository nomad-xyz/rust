use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::{eyre::ensure, Result};
use ethers::{signers::Signer, types::Address};
use futures_util::future::select_all;
use prometheus::IntCounter;
use tokio::task::JoinHandle;
use tracing::{info, instrument::Instrumented, Instrument};

use crate::{
    produce::UpdateProducer, settings::UpdaterSettings as Settings, submit::UpdateSubmitter,
};
use nomad_base::{AgentCore, CachingHome, NomadAgent, NomadDB};
use nomad_core::{Common, Signers};

/// An updater agent
#[derive(Debug)]
pub struct Updater {
    signer: Arc<Signers>,
    interval_seconds: u64,
    finalization_seconds: u64,
    pub(crate) core: AgentCore,
    signed_attestation_count: IntCounter,
    submitted_update_count: IntCounter,
}

impl AsRef<AgentCore> for Updater {
    fn as_ref(&self) -> &AgentCore {
        &self.core
    }
}

impl Updater {
    /// Instantiate a new updater
    pub fn new(
        signer: Signers,
        interval_seconds: u64,
        finalization_seconds: u64,
        core: AgentCore,
    ) -> Self {
        let home_name = core.home.name();
        let signed_attestation_count = core
            .metrics
            .new_int_counter(
                "signed_attestation_count",
                "Number of attestations signed",
                &["network", "agent"],
            )
            .expect("failed to register signed_attestation_count")
            .with_label_values(&[home_name, Self::AGENT_NAME]);

        let submitted_update_count = core
            .metrics
            .new_int_counter(
                "submitted_update_count",
                "Number of updates successfully submitted to home",
                &["network", "agent"],
            )
            .expect("failed to register submitted_update_count")
            .with_label_values(&[home_name, Self::AGENT_NAME]);

        Self {
            signer: Arc::new(signer),
            interval_seconds,
            finalization_seconds,
            core,
            signed_attestation_count,
            submitted_update_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdaterChannel {
    home: Arc<CachingHome>,
    db: NomadDB,
    signer: Arc<Signers>,
    signed_attestation_count: IntCounter,
    submitted_update_count: IntCounter,
    finalization_seconds: u64,
    interval_seconds: u64,
}

// This is a bit of a kludge to make from_settings work.
// Ideally this hould be generic across all signers.
// Right now we only have one
#[async_trait]
impl NomadAgent for Updater {
    const AGENT_NAME: &'static str = "updater";

    type Settings = Settings;

    type Channel = UpdaterChannel;

    async fn from_settings(settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        let signer = Signers::try_from_signer_conf(
            settings
                .as_ref()
                .attestation_signer
                .as_ref()
                .expect("!signer"),
        )
        .await?;
        let interval_seconds = settings.agent.interval;

        let block_time = settings.as_ref().home.block_time;
        let finality_blocks = settings.as_ref().home.finality as u64;
        let finalization_seconds = finality_blocks * block_time;

        let core = settings.as_ref().try_into_core(Self::AGENT_NAME).await?;
        Ok(Self::new(
            signer,
            interval_seconds,
            finalization_seconds,
            core,
        ))
    }

    fn build_channel(&self, _replica: &str) -> Self::Channel {
        Self::Channel {
            home: self.home(),
            db: NomadDB::new(self.home().name(), self.db()),
            signer: self.signer.clone(),
            signed_attestation_count: self.signed_attestation_count.clone(),
            submitted_update_count: self.submitted_update_count.clone(),
            finalization_seconds: self.finalization_seconds,
            interval_seconds: self.interval_seconds,
        }
    }

    fn run(channel: Self::Channel) -> Instrumented<JoinHandle<Result<()>>> {
        let home = channel.home.clone();
        let address = channel.signer.address();
        let db = channel.db.clone();

        let produce = UpdateProducer::new(
            home.clone(),
            db.clone(),
            channel.signer.clone(),
            channel.interval_seconds,
            channel.signed_attestation_count.clone(),
        );

        let submit = UpdateSubmitter::new(
            home.clone(),
            db,
            channel.interval_seconds,
            channel.finalization_seconds,
            channel.submitted_update_count,
        );

        tokio::spawn(async move {
            let expected: Address = home.updater().await?.into();
            ensure!(
                expected == address,
                "Contract updater does not match keys. On-chain: {}. Local: {}",
                expected,
                address
            );

            // Only spawn updater tasks once syncing has finished
            info!("Spawning produce and submit tasks...");
            let produce_task = produce.spawn();
            let submit_task = submit.spawn();

            let (res, _, rem) = select_all(vec![produce_task, submit_task]).await;

            for task in rem.into_iter() {
                task.into_inner().abort();
            }
            res?
        })
        .in_current_span()
    }

    fn run_many(&self, _replicas: &[&str]) -> Instrumented<JoinHandle<Result<()>>> {
        panic!("Updater::run_many should not be called. Always call run_all")
    }

    fn run_all(self) -> Instrumented<JoinHandle<Result<()>>>
    where
        Self: Sized + 'static,
    {
        tokio::spawn(async move {
            self.assert_home_not_failed().await??;

            let home_fail_watch_task = self.watch_home_fail(self.interval_seconds);
            let sync_task = self.home().sync();
            let update_task = self.run_report_error("".to_owned()); // just operates on home

            let (res, _, rem) =
                select_all(vec![home_fail_watch_task, sync_task, update_task]).await;

            for task in rem.into_iter() {
                task.into_inner().abort();
            }
            res?
        })
        .in_current_span()
    }
}

#[cfg(test)]
mod test {}
