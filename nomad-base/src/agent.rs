use crate::{
    cancel_task,
    metrics::CoreMetrics,
    settings::{IndexSettings, Settings},
    trace::{
        fmt::{log_level_to_level_filter, LogOutputLayer},
        TimeSpanLifetime,
    },
    BaseError, CachingHome, CachingReplica, NomadDB,
};
use async_trait::async_trait;
use color_eyre::{eyre::WrapErr, Result};
use futures_util::future::select_all;
use nomad_core::{db::DB, Common};
use tracing::{dispatcher::DefaultGuard, instrument::Instrumented};
use tracing::{error, info_span, warn, Instrument};
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{task::JoinHandle, time::sleep};

const MAX_EXPONENTIAL: u32 = 7; // 2^7 = 128 second timeout

/// General or agent-specific connection map
#[derive(Debug, Clone)]
pub enum AgentConnections {
    /// Connections for watchers
    Watcher {
        /// A map of boxed Homes
        homes: HashMap<String, Arc<CachingHome>>,
        // ...
    },
    /// Connections for other agents
    Default {
        /// A boxed Home
        home: Arc<CachingHome>,
        /// A map of boxed Replicas
        replicas: HashMap<String, Arc<CachingReplica>>,
    },
}

/// Accessor methods for AgentConnections
impl AgentConnections {
    /// Get an optional clone of home
    pub fn home(&self) -> Option<Arc<CachingHome>> {
        use AgentConnections::*;
        match self {
            Default { home, .. } => Some(home.clone()),
            _ => None,
        }
    }

    /// Get an optional clone of the map of replicas
    pub fn replicas(&self) -> Option<HashMap<String, Arc<CachingReplica>>> {
        use AgentConnections::*;
        match self {
            Default { replicas, .. } => Some(replicas.clone()),
            _ => None,
        }
    }

    /// Get an optional clone of a replica by its name
    pub fn replica_by_name(&self, name: &str) -> Option<Arc<CachingReplica>> {
        self.replicas().and_then(|r| r.get(name).map(Clone::clone))
    }
}

/// Properties shared across all agents
#[derive(Debug, Clone)]
pub struct AgentCore {
    /// Agent connections
    pub connections: AgentConnections,
    /// A persistent KV Store (currently implemented as rocksdb)
    pub db: DB,
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
        let home = self.connections().home().expect("!home");
        ChannelBase {
            home: home.clone(),
            replica: self
                .connections()
                .replica_by_name(replica)
                .expect("!replica exist"),
            db: NomadDB::new(home.name(), self.db()),
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

    /// Return a reference to the connections object
    fn connections(&self) -> &AgentConnections {
        &self.as_ref().connections
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

        // easy check that the slice is non-empty
        replicas
            .first()
            .expect("Attempted to run without any replicas");

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
            let replicas = self.connections().replicas().expect("!replicas");
            let names: Vec<&str> = replicas.keys().map(|k| k.as_str()).collect();

            // quick check that at least 1 replica is configured
            names
                .first()
                .expect("Attempted to run without any replicas");

            let run_task = self.run_many(&names);
            let mut tasks = vec![run_task];

            // kludge
            if Self::AGENT_NAME != "kathy" {
                // Only the processor needs to index messages so default is
                // just indexing updates
                let sync_task = self.connections().home().expect("!home").sync();

                tasks.push(sync_task);
            }

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
        let home = self.connections().home().expect("!home");
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
        let home = self.connections().home().expect("!home");
        tokio::spawn(async move {
            if home.state().await? == nomad_core::State::Failed {
                Err(BaseError::FailedHome.into())
            } else {
                Ok(())
            }
        })
        .instrument(span)
    }

    /// Attempt to instantiate and register a tracing subscriber setup from
    /// settings.
    ///
    /// Returns a default guard that will set the agent's tracing as the
    /// default subscriber
    fn start_tracing(&self, latencies: prometheus::HistogramVec) -> DefaultGuard {
        let log = self.as_ref().settings.logging;
        let level_filter = log_level_to_level_filter(log.level);
        let fmt_layer: LogOutputLayer<_> = log.fmt.into();
        let err_layer = tracing_error::ErrorLayer::default();

        let subscriber = tracing_subscriber::Registry::default()
            .with(TimeSpanLifetime::new(latencies))
            .with(level_filter)
            .with(fmt_layer)
            .with(err_layer);

        subscriber.set_default()
    }
}
