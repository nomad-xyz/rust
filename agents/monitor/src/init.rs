use std::{collections::HashMap, sync::Arc};

use ethers::{
    middleware::TimeLag,
    prelude::{Http, Provider as EthersProvider},
};

use nomad_ethereum::bindings::{
    home::{DispatchFilter, UpdateFilter},
    replica::{ProcessFilter, UpdateFilter as RelayFilter},
};
use nomad_xyz_configuration::{get_builtin, NomadConfig};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tracing_subscriber::EnvFilter;

use crate::{
    annotate::WithMeta,
    between::BetweenHandle,
    dispatch_wait::DispatchWaitHandle,
    domain::Domain,
    metrics::Metrics,
    producer::{
        DispatchProducerHandle, ProcessProducerHandle, RelayProducerHandle, UpdateProducerHandle,
    },
    ArcProvider, HomeReplicaMap,
};

pub(crate) fn config_from_file() -> Option<NomadConfig> {
    std::env::var("CONFIG_PATH")
        .ok()
        .and_then(|path| NomadConfig::from_file(path).ok())
}

pub(crate) fn config_from_env() -> Option<NomadConfig> {
    std::env::var("RUN_ENV")
        .ok()
        .and_then(|env| get_builtin(&env))
        .map(ToOwned::to_owned)
}

pub(crate) fn config() -> eyre::Result<NomadConfig> {
    config_from_file()
        .or_else(config_from_env)
        .ok_or_else(|| eyre::eyre!("Unable to load config from file or env"))
}

pub(crate) fn init_tracing() {
    tracing_subscriber::FmtSubscriber::builder()
        .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .with_level(true)
        .init();
}

pub(crate) fn rpc_from_env(network: &str) -> Option<String> {
    std::env::var(format!("{}_CONNECTION_URL", network.to_uppercase())).ok()
}

pub(crate) fn provider_for(config: &NomadConfig, network: &str) -> eyre::Result<ArcProvider> {
    tracing::info!(network, "Instantiating provider");

    let url = rpc_from_env(network).or_else(|| {
        config
            .rpcs
            .get(network)
            .and_then(|set| set.iter().next().cloned())
    });

    eyre::ensure!(
        url.is_some(),
        "Missing Url. Please specify by config or env var."
    );

    let url = url.expect("checked on previous line");
    let provider = EthersProvider::<Http>::try_from(&url)?;

    let timelag = config
        .protocol()
        .networks
        .get(network)
        .expect("missing protocol block in config")
        .specs
        .finalization_blocks;

    tracing::debug!(url = url.as_str(), timelag, network, "Connect network");
    Ok(TimeLag::new(provider, timelag).into())
}

pub(crate) fn monitor() -> eyre::Result<Monitor> {
    Monitor::from_config(&config()?)
}

#[derive(Debug)]
pub(crate) struct Monitor {
    networks: HashMap<String, Domain>,
    metrics: Arc<Metrics>,
}

impl Monitor {
    pub(crate) fn from_config(config: &NomadConfig) -> eyre::Result<Self> {
        let mut networks = HashMap::new();
        for network in config.networks.iter() {
            networks.insert(network.to_owned(), Domain::from_config(config, network)?);
        }
        let metrics = Metrics::new()?.into();
        Ok(Monitor { networks, metrics })
    }

    pub(crate) fn run_http_server(&self) -> JoinHandle<()> {
        self.metrics.clone().run_http_server()
    }

    pub(crate) fn run_dispatch_producers(&self) -> HashMap<&str, DispatchProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.dispatch_producer()))
            .collect()
    }

    pub(crate) fn run_update_producers(&self) -> HashMap<&str, UpdateProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.update_producer()))
            .collect()
    }

    pub(crate) fn run_relay_producers(&self) -> HomeReplicaMap<RelayProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.relay_producers()))
            .collect()
    }

    pub(crate) fn run_process_producers(&self) -> HomeReplicaMap<ProcessProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.process_producers()))
            .collect()
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn run_between_dispatch(
        &self,
        mut incomings: HashMap<&str, UnboundedReceiver<WithMeta<DispatchFilter>>>,
    ) -> HashMap<&str, BetweenHandle<WithMeta<DispatchFilter>>> {
        self.networks
            .iter()
            .map(|(chain, domain)| {
                let emitter = domain.home_address();
                let event = "dispatch";

                let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

                let producer = incomings.remove(chain.as_str()).expect("missing producer");

                let between = domain.count_dispatches(producer, metrics, event);

                (chain.as_str(), between)
            })
            .collect()
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn run_between_update(
        &self,
        mut incomings: HashMap<&str, UnboundedReceiver<WithMeta<UpdateFilter>>>,
    ) -> HashMap<&str, BetweenHandle<WithMeta<UpdateFilter>>> {
        self.networks
            .iter()
            .map(|(chain, domain)| {
                let emitter = format!("{:?}", domain.home().address());
                let event = "update";

                let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

                let producer = incomings.remove(chain.as_str()).expect("missing producer");

                let between = domain.count_updates(producer, metrics, event);

                (chain.as_str(), between)
            })
            .collect()
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn run_between_relay(
        &self,
        mut incomings: HomeReplicaMap<UnboundedReceiver<WithMeta<RelayFilter>>>,
    ) -> HomeReplicaMap<BetweenHandle<WithMeta<RelayFilter>>> {
        self.networks
            .iter()
            .map(|(network, domain)| {
                let incomings = incomings
                    .remove(network.as_str())
                    .expect("missing producer")
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect();
                (
                    network.as_str(),
                    domain.count_relays(incomings, self.metrics.clone()),
                )
            })
            .collect()
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn run_between_process(
        &self,
        mut incomings: HomeReplicaMap<UnboundedReceiver<WithMeta<ProcessFilter>>>,
    ) -> HomeReplicaMap<BetweenHandle<WithMeta<ProcessFilter>>> {
        self.networks
            .iter()
            .map(|(network, domain)| {
                let incomings = incomings
                    .remove(network.as_str())
                    .expect("missing producer")
                    .into_iter()
                    .collect();
                (
                    network.as_str(),
                    domain.count_processes(incomings, self.metrics.clone()),
                )
            })
            .collect()
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn run_dispatch_to_update(
        &self,
        mut incoming_dispatches: HashMap<&str, UnboundedReceiver<WithMeta<DispatchFilter>>>,
        mut incoming_updates: HashMap<&str, UnboundedReceiver<WithMeta<UpdateFilter>>>,
    ) -> HashMap<&str, DispatchWaitHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| {
                let incoming_dispatch = incoming_dispatches
                    .remove(network.as_str())
                    .expect("missing incoming dispatch");
                let incoming_update = incoming_updates
                    .remove(network.as_str())
                    .expect("missing incoming update");

                let d_to_r = domain.dispatch_to_update(
                    incoming_dispatch,
                    incoming_update,
                    self.metrics.clone(),
                );
                (network.as_str(), d_to_r)
            })
            .collect()
    }
}
