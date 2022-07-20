use std::{collections::HashMap, sync::Arc};

use ethers::{
    middleware::TimeLag,
    prelude::{Http, Provider as EthersProvider},
};

use nomad_xyz_configuration::{get_builtin, NomadConfig};
use tokio::task::JoinHandle;
use tracing_subscriber::EnvFilter;

use crate::{
    domain::Domain,
    metrics::Metrics,
    producer::{
        DispatchProducerHandle, ProcessProducerHandle, RelayProducerHandle, UpdateProducerHandle,
    },
    utils, ArcProvider, DispatchFaucet, Faucets, HomeReplicaMap, NetworkMap, ProcessFaucet,
    RelayFaucet, UpdateFaucet,
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

    fn run_dispatch_producers(&self) -> HashMap<&str, DispatchProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.dispatch_producer()))
            .collect()
    }

    fn run_update_producers(&self) -> HashMap<&str, UpdateProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.update_producer()))
            .collect()
    }

    fn run_relay_producers(&self) -> HomeReplicaMap<RelayProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.relay_producers()))
            .collect()
    }

    fn run_process_producers(&self) -> HomeReplicaMap<ProcessProducerHandle> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.process_producers()))
            .collect()
    }
    pub(crate) fn producers(&self) -> Faucets {
        Faucets {
            dispatches: utils::split(self.run_dispatch_producers()).1,
            updates: utils::split(self.run_update_producers()).1,
            relays: utils::nested_split(self.run_relay_producers()).1,
            processes: utils::nested_split(self.run_process_producers()).1,
        }
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_dispatch<'a>(&'a self, incomings: &mut NetworkMap<'a, DispatchFaucet>) {
        self.networks.iter().for_each(|(chain, domain)| {
            let emitter = domain.home_address();
            let event = "dispatch";

            let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

            let producer = incomings.remove(chain.as_str()).expect("missing producer");

            let between = domain.count_dispatches(producer, metrics, event);

            incomings.insert(chain, between.rx);
        })
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_update<'a>(&'a self, incomings: &mut NetworkMap<'a, UpdateFaucet>) {
        self.networks.iter().for_each(|(chain, domain)| {
            let emitter = format!("{:?}", domain.home().address());
            let event = "update";

            let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

            let producer = incomings.remove(chain.as_str()).expect("missing producer");

            let between = domain.count_updates(producer, metrics, event);

            incomings.insert(chain, between.rx);
        })
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_relay<'a>(&'a self, incomings: &mut HomeReplicaMap<'a, RelayFaucet>) {
        self.networks.iter().for_each(|(network, domain)| {
            let inner = incomings
                .get_mut(network.as_str())
                .expect("missing producer");

            domain.count_relays(inner, self.metrics.clone());
        });
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_process<'a>(&'a self, incomings: &mut HomeReplicaMap<'a, ProcessFaucet>) {
        self.networks.iter().for_each(|(network, domain)| {
            let inner = incomings
                .get_mut(network.as_str())
                .expect("missing producer");

            domain.count_processes(inner, self.metrics.clone());
        });
    }

    pub(crate) fn run_betweens<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.run_between_dispatch(&mut faucets.dispatches);
        self.run_between_update(&mut faucets.updates);
        self.run_between_relay(&mut faucets.relays);
        self.run_between_process(&mut faucets.processes);
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn run_dispatch_to_update<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks.iter().for_each(|(network, domain)| {
            let incoming_dispatch = faucets
                .dispatches
                .remove(network.as_str())
                .expect("missing incoming dispatch");
            let incoming_update = faucets
                .updates
                .remove(network.as_str())
                .expect("missing incoming update");

            let d_to_r =
                domain.dispatch_to_update(incoming_dispatch, incoming_update, self.metrics.clone());

            faucets.dispatches.insert(network, d_to_r.rx.dispatches);
            faucets.updates.insert(network, d_to_r.rx.updates);
        });
    }

    pub(crate) fn run_update_to_relay<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks.iter().for_each(|(_, v)| {
            v.update_to_relay(
                &mut faucets.updates,
                &mut faucets.relays,
                self.metrics.clone(),
            )
        });
    }
}
