use std::{collections::HashMap, sync::Arc};

use ethers::{
    middleware::TimeLag,
    prelude::{Http, Provider as EthersProvider},
};

use nomad_xyz_configuration::{get_builtin, NomadConfig};
use tokio::task::JoinHandle;
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::{
    domain::Domain,
    faucets::Faucets,
    metrics::Metrics,
    steps::{e2e::E2ELatency, terminal::Terminal},
    ArcProvider, DispatchFaucet, HomeReplicaMap, ProcessFaucet, ProcessStep, RelayFaucet,
    UpdateFaucet,
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
        .with_max_level(Level::INFO)
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

    fn run_dispatch_producers(&self) -> HashMap<&str, DispatchFaucet> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.dispatch_producer()))
            .collect()
    }

    fn run_update_producers(&self) -> HashMap<&str, UpdateFaucet> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.update_producer()))
            .collect()
    }

    fn run_relay_producers(&self) -> HomeReplicaMap<RelayFaucet> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.relay_producers()))
            .collect()
    }

    fn run_process_producers(&self) -> HomeReplicaMap<ProcessFaucet> {
        self.networks
            .iter()
            .map(|(network, domain)| (network.as_str(), domain.process_producers()))
            .collect()
    }

    pub(crate) fn producers(&self) -> Faucets {
        Faucets {
            dispatches: self.run_dispatch_producers(),
            updates: self.run_update_producers(),
            relays: self.run_relay_producers(),
            processes: self.run_process_producers(),
        }
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_dispatch<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks.iter().for_each(|(chain, domain)| {
            let emitter = domain.home_address();
            let event = "dispatch";

            let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

            domain.count_dispatches(faucets, metrics, event);
        })
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_update<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks.iter().for_each(|(chain, domain)| {
            let emitter = format!("{:?}", domain.home().address());
            let event = "update";

            let metrics = self.metrics.between_metrics(chain, event, &emitter, None);

            domain.count_updates(faucets, metrics, event);
        })
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_relay<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks.values().for_each(|domain| {
            domain.count_relays(faucets, self.metrics.clone());
        });
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn run_between_process<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks.values().for_each(|domain| {
            domain.count_processes(faucets, self.metrics.clone());
        });
    }

    pub(crate) fn run_betweens<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.run_between_dispatch(faucets);
        self.run_between_update(faucets);
        self.run_between_relay(faucets);
        self.run_between_process(faucets);
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn run_dispatch_to_update<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks.values().for_each(|domain| {
            domain.dispatch_to_update(faucets, self.metrics.clone());
        });
    }

    pub(crate) fn run_update_to_relay<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks
            .values()
            .for_each(|v| v.update_to_relay(faucets, self.metrics.clone()));
    }

    pub(crate) fn run_relay_to_process<'a>(&'a self, faucets: &mut Faucets<'a>) {
        self.networks
            .values()
            .for_each(|domain| domain.relay_to_process(faucets, self.metrics.clone()));
    }

    pub(crate) fn run_e2e<'a>(&'a self, faucets: &mut Faucets<'a>) {
        let (process_sinks, process_faucets) = faucets.swap_all_processes();
        let (dispatch_sinks, dispatch_faucets) = faucets.swap_all_dispatches();

        let metrics = self
            .metrics
            .e2e_metrics(process_sinks.keys().map(AsRef::as_ref));

        let domain_to_network = self
            .networks
            .values()
            .map(|domain| (domain.domain_number, domain.name().to_owned()))
            .collect();

        E2ELatency::new(
            dispatch_faucets,
            process_faucets,
            domain_to_network,
            metrics,
            dispatch_sinks,
            process_sinks,
        )
        .run_until_panic();
    }

    /// take ownership of all faucets and terminate them
    pub(crate) fn run_terminals<'a>(&'a self, faucets: Faucets<'a>) -> Vec<JoinHandle<()>> {
        let mut tasks = vec![];

        faucets.dispatches.into_iter().for_each(|(_, v)| {
            tasks.push(Terminal::new(v).run_until_panic());
        });

        faucets.updates.into_iter().for_each(|(_, v)| {
            tasks.push(Terminal::new(v).run_until_panic());
        });

        faucets.relays.into_iter().for_each(|(_, v)| {
            v.into_iter().for_each(|(_, v)| {
                tasks.push(Terminal::new(v).run_until_panic());
            });
        });

        faucets.processes.into_iter().for_each(|(_, v)| {
            v.into_iter().for_each(|(_, v)| {
                tasks.push(Terminal::new(v).run_until_panic());
            });
        });

        tasks
    }
}
