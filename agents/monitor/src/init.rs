use std::{collections::HashMap, sync::Arc};

use ethers::{
    middleware::TimeLag,
    prelude::{Http, Provider as EthersProvider},
};

use nomad_xyz_configuration::{get_builtin, NomadConfig};
use tokio::task::JoinHandle;

use crate::{domain::Domain, metrics::Metrics, ArcProvider};

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
    Monitor::from_config(config()?)
}

#[derive(Debug)]
pub(crate) struct Monitor {
    config: NomadConfig,
    networks: HashMap<String, Domain>,
    metrics: Arc<Metrics>,
}

impl Monitor {
    pub(crate) fn from_config(config: NomadConfig) -> eyre::Result<Self> {
        let mut networks = HashMap::new();
        for network in config.networks.iter() {
            networks.insert(network.to_owned(), Domain::from_config(&config, network)?);
        }
        let metrics = Metrics::new()?.into();
        Ok(Monitor {
            config,
            networks,
            metrics,
        })
    }

    pub(crate) fn run_http_server(&self) -> JoinHandle<()> {
        self.metrics.clone().run_http_server()
    }
}
