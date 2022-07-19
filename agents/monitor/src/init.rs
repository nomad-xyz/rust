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
use tokio::task::JoinHandle;
use tracing_subscriber::EnvFilter;

use crate::{
    annotate::WithMeta, between::BetweenHandle, domain::Domain, metrics::Metrics, ArcProvider,
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
        .json()
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

    let url = url.unwrap();
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

    pub(crate) fn run_between_dispatch(
        &self,
    ) -> HashMap<&str, BetweenHandle<WithMeta<DispatchFilter>>> {
        self.networks
            .iter()
            .map(|(chain, domain)| {
                let emitter = format!("{:?}", domain.home().address());
                let event = "dispatch";

                let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

                let producer = domain.dispatch_producer();
                let between = domain.count(producer.rx, metrics, event);

                (chain.as_str(), between)
            })
            .collect()
    }

    pub(crate) fn run_between_update(
        &self,
    ) -> HashMap<&str, BetweenHandle<WithMeta<UpdateFilter>>> {
        self.networks
            .iter()
            .map(|(chain, domain)| {
                let emitter = format!("{:?}", domain.home().address());
                let event = "update";

                let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

                let producer = domain.update_producer();
                let between = domain.count(producer.rx, metrics, event);

                (chain.as_str(), between)
            })
            .collect()
    }

    pub(crate) fn run_between_relay(
        &self,
    ) -> HashMap<&str, HashMap<&str, BetweenHandle<WithMeta<RelayFilter>>>> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.count_relays(self.metrics.clone())))
            .collect()
    }

    pub(crate) fn run_between_process(
        &self,
    ) -> HashMap<&str, HashMap<&str, BetweenHandle<WithMeta<ProcessFilter>>>> {
        self.networks
            .iter()
            .map(|(network, domain)| {
                (
                    network.as_str(),
                    domain.count_processes(self.metrics.clone()),
                )
            })
            .collect()
    }
}
