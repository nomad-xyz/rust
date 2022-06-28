use crate::{
    cancel_task,
    metrics::CoreMetrics,
    settings::{IndexSettings, Settings},
    trace::{
        fmt::{log_level_to_level_filter, LogOutputLayer},
        TimeSpanLifetime,
    },
    BaseError, CachingHome, CachingReplica, NomadDB, TxSender,
};
use async_trait::async_trait;
use color_eyre::{eyre::WrapErr, Result};
use futures_util::future::select_all;
use nomad_core::{db::DB, Common};
use tracing::instrument::Instrumented;
use tracing::{error, info_span, warn, Instrument};
use tracing_subscriber::prelude::*;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{task::JoinHandle, time::sleep};

const MAX_EXPONENTIAL: u32 = 7; // 2^7 = 128 second timeout

/// Properties shared across all agents
#[derive(Debug, Clone)]
pub struct AgentCore {
    /// A boxed Home
    pub home: Arc<CachingHome>,
    /// A map of boxed Replicas
    pub replicas: HashMap<String, Arc<CachingReplica>>,
    /// A persistent KV Store (currently implemented as rocksdb)
    pub db: DB,
    /// A map of tx senders per network
    pub tx_senders: HashMap<String, TxSender>,
    /// Prometheus metrics
    pub metrics: Arc<CoreMetrics>,
    /// The height at which to start indexing the Home
    pub indexer: IndexSettings,
    /// Settings this agent was created with
    pub settings: crate::settings::Settings,
}

/// Commmon data needed for a single agent channel
#[derive(Debug, Clone)]
pub struct ChannelBase {
    /// Home
    pub home: Arc<CachingHome>,
    /// Replica
    pub replica: Arc<CachingReplica>,
    /// NomadDB keyed by home
    pub db: NomadDB,
}

/// A trait for an application:
///      that runs on a replica
/// and:
///     a reference to a home.
#[async_trait]
pub trait NomadAgent: Send + Sync + Sized + std::fmt::Debug + AsRef<AgentCore> {
    /// The agent's name
    const AGENT_NAME: &'static str;

    /// The settings object for this agent
    type Settings: AsRef<Settings>;

    /// The data needed for a single channel's run task
    type Channel: 'static + Send + Sync + Clone;

    /// Instantiate the agent from the standard settings object
    async fn from_settings(settings: Self::Settings) -> Result<Self>
    where
        Self: Sized;

    /// Build a channel struct for a given home <> replica channel
    fn build_channel(&self, replica: &str) -> Self::Channel;

    /// Build channel base for home <> replica channel
    fn channel_base(&self, replica: &str) -> ChannelBase {
        ChannelBase {
            home: self.home(),
            replica: self.replica_by_name(replica).expect("!replica exist"),
            db: NomadDB::new(self.home().name(), self.db()),
        }
    }

    /// Return a handle to the metrics registry
    fn metrics(&self) -> Arc<CoreMetrics> {
        self.as_ref().metrics.clone()
    }

    /// Return a handle to the DB
    fn db(&self) -> DB {
        self.as_ref().db.clone()
    }

    /// Return a handle to the tx senders
    fn tx_senders(&self) -> HashMap<String, TxSender> {
        self.as_ref().tx_senders.clone()
    }

    /// Return a reference to a home contract
    fn home(&self) -> Arc<CachingHome> {
        self.as_ref().home.clone()
    }

    /// Get a reference to the replicas map
    fn replicas(&self) -> &HashMap<String, Arc<CachingReplica>> {
        &self.as_ref().replicas
    }

    /// Get a reference to a replica by its name
    fn replica_by_name(&self, name: &str) -> Option<Arc<CachingReplica>> {
        self.replicas().get(name).map(Clone::clone)
    }

    /// Run the agent with the given home and replica
    fn run(channel: Self::Channel) -> Instrumented<JoinHandle<Result<()>>>;

    /// Run the agent for a given channel. If the channel dies, exponentially
    /// retry. If failures are more than 5 minutes apart, reset exponential
    /// backoff (likely unrelated after that point).
    #[allow(clippy::unit_arg)]
    #[tracing::instrument]
    fn run_report_error(&self, replica: String) -> Instrumented<JoinHandle<Result<()>>> {
        let channel = self.build_channel(&replica);
        let channel_faults_gauge = self.metrics().channel_faults_gauge(&replica);

        tokio::spawn(async move {
            let mut exponential = 0;
            loop {
                let running_time = SystemTime::now();

                let handle = Self::run(channel.clone());
                let res = handle
                    .await?
                    .wrap_err(format!("Task for replica named {} failed", &replica));

                match res {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        error!(
                            "Channel for replica {} errored out! Error: {:?}",
                            &replica, e
                        );
                        channel_faults_gauge.inc();

                        // If running time >= 5 minutes, current failure likely
                        // unrelated to previous
                        if running_time.elapsed().unwrap().as_secs() >= 300 {
                            exponential = 0;
                        } else if exponential < MAX_EXPONENTIAL {
                            exponential += 1;
                        }

                        let sleep_time = 2u64.pow(exponential);
                        warn!(
                            "Restarting channel to {} in {} seconds",
                            &replica, sleep_time
                        );

                        sleep(Duration::from_secs(sleep_time)).await;
                    }
                }
            }
        })
        .in_current_span()
    }

    /// Run several agents by replica name
    #[allow(clippy::unit_arg)]
    fn run_many(&self, replicas: &[&str]) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("run_many");
        let handles: Vec<_> = replicas
            .iter()
            .map(|replica| self.run_report_error(replica.to_string()))
            .collect();

        tokio::spawn(async move {
            // This gets the first future to resolve.
            let (res, _, remaining) = select_all(handles).await;

            for task in remaining.into_iter() {
                cancel_task!(task);
            }

            res?
        })
        .instrument(span)
    }

    /// Run several agents
    #[allow(clippy::unit_arg, unused_must_use)]
    fn run_all(self) -> Instrumented<JoinHandle<Result<()>>>
    where
        Self: Sized + 'static,
    {
        let span = info_span!("run_all");
        tokio::spawn(async move {
            // this is the unused must use
            let names: Vec<&str> = self.replicas().keys().map(|k| k.as_str()).collect();

            let run_task = self.run_many(&names);
            let mut tasks = vec![run_task];

            // kludge
            if Self::AGENT_NAME != "kathy" {
                // Only the processor needs to index messages so default is
                // just indexing updates
                let sync_task = self.home().sync();

                tasks.push(sync_task);
            }

            let sender_task = self.run_tx_senders();
            tasks.push(sender_task);

            let (res, _, remaining) = select_all(tasks).await;

            for task in remaining.into_iter() {
                cancel_task!(task);
            }

            res?
        })
        .instrument(span)
    }

    /// Spawn a task which continuously watch home for getting into failed state
    /// and resolve once it happened.
    /// `Reported` flag turns `Ok(())` into `Err(Report)` on failed home.
    #[allow(clippy::unit_arg)]
    fn watch_home_fail(&self, interval: u64) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("home_watch");
        let home = self.home();
        let home_failure_checks = self.metrics().home_failure_checks();
        let home_failure_observations = self.metrics().home_failure_observations();

        tokio::spawn(async move {
            loop {
                if home.state().await? == nomad_core::State::Failed {
                    home_failure_observations.inc();
                    return Err(BaseError::FailedHome.into());
                }

                home_failure_checks.inc();
                sleep(Duration::from_secs(interval)).await;
            }
        })
        .instrument(span)
    }

    /// Returns `true` if home is in failed state. Intended to return once and immediately
    #[allow(clippy::unit_arg)]
    fn assert_home_not_failed(&self) -> Instrumented<JoinHandle<Result<()>>> {
        use nomad_core::Common;
        let span = info_span!("check_home_state");
        let home = self.home();
        tokio::spawn(async move {
            if home.state().await? == nomad_core::State::Failed {
                Err(BaseError::FailedHome.into())
            } else {
                Ok(())
            }
        })
        .instrument(span)
    }

    /// Attempt to instantiate and register a tracing subscriber setup from settings.
    fn start_tracing(&self, latencies: prometheus::HistogramVec) -> Result<()> {
        let log = self.as_ref().settings.logging;
        let level_filter = log_level_to_level_filter(log.level);
        let fmt_layer: LogOutputLayer<_> = log.fmt.into();
        let err_layer = tracing_error::ErrorLayer::default();

        let subscriber = tracing_subscriber::Registry::default()
            .with(TimeSpanLifetime::new(latencies))
            .with(level_filter)
            .with(fmt_layer)
            .with(err_layer);

        subscriber.try_init()?;
        Ok(())
    }

    /// Run tx senders
    fn run_tx_senders(&self) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("run_tx_senders");

        let handles = self
            .tx_senders()
            .into_iter()
            .map(|(_, tx_sender)| {
                tokio::spawn(async move { tx_sender.run().await }).in_current_span()
            })
            .collect::<Vec<_>>();

        tokio::spawn(async move {
            let (res, _, remaining) = select_all(handles).await;
            for task in remaining.into_iter() {
                cancel_task!(task);
            }
            res?
        })
        .instrument(span)
    }
}
