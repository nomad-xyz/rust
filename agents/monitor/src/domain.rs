use std::collections::HashMap;

use ethers::{
    contract::builders::Event,
    middleware::TimeLag,
    prelude::{Http, Provider as EthersProvider, StreamExt},
};

use nomad_ethereum::bindings::{
    home::{DispatchFilter, Home, UpdateFilter as HomeUpdateFilter},
    replica::{ProcessFilter, Replica, UpdateFilter as ReplicaUpdateFilter},
};
use nomad_xyz_configuration::{contracts::CoreContracts, NomadConfig};
use prometheus::{Histogram, IntCounter};
use tokio::sync::mpsc;
use tracing::{info_span, Instrument};

use crate::{
    annotate::WithMeta,
    between::{BetweenEvents, BetweenHandle, BetweenMetrics, BetweenTask},
    init::provider_for,
    producer::{DispatchProducer, DispatchProducerHandle},
    ArcProvider, ProcessStep, Provider, StepHandle,
};

macro_rules! unwrap_event_stream_item {
    ($event:ident, $net:ident, $name:literal) => {{
        match $event {
            None => break,
            Some(Err(error)) => {
                tracing::error!(%error, net = AsRef::<str>::as_ref(&$net), event = $name, "Stream ended");
                break;
            }
            Some(Ok(event)) => event,
        }
    }}
}

#[derive(Debug)]
pub(crate) struct Domain {
    pub(crate) network: String,
    pub(crate) provider: ArcProvider,
    pub(crate) home: Home<Provider>,
    pub(crate) replicas: HashMap<String, Replica<Provider>>,
}

impl Domain {
    pub(crate) fn from_config(config: &NomadConfig, network: &str) -> eyre::Result<Self> {
        let name = network.to_owned();
        let provider = provider_for(config, network)?;

        let CoreContracts::Evm(core) = config.core().get(network).expect("invalid config");

        let home = Home::new(core.home.proxy.as_ethereum_address()?, provider.clone());

        let replicas = core
            .replicas
            .iter()
            .map(|(k, v)| {
                let replica = Replica::new(
                    v.proxy.as_ethereum_address().expect("invalid addr"),
                    provider.clone(),
                );
                (k.clone(), replica)
            })
            .collect();

        Ok(Domain {
            network: name,
            provider,
            home,
            replicas,
        })
    }

    pub(crate) fn name(&self) -> &str {
        self.network.as_ref()
    }

    pub(crate) fn provider(&self) -> &TimeLag<EthersProvider<Http>> {
        self.provider.as_ref()
    }

    pub(crate) fn home(&self) -> &Home<Provider> {
        &self.home
    }

    pub(crate) fn replicas(&self) -> &HashMap<String, Replica<Provider>> {
        &self.replicas
    }

    pub(crate) fn dispatch_producer(
        &self,
    ) -> StepHandle<DispatchProducer, WithMeta<DispatchFilter>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = DispatchProducer::new(self.home.clone(), self.network.clone(), tx).spawn();

        StepHandle { handle, rx }
    }

    pub(crate) fn count<T>(
        &self,
        incoming: mpsc::UnboundedReceiver<WithMeta<T>>,
        metrics: BetweenMetrics,
        event: impl AsRef<str>,
    ) -> BetweenHandle<WithMeta<T>>
    where
        T: 'static + Send + Sync + std::fmt::Debug,
    {
        let network = self.network.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        let handle =
            BetweenEvents::<WithMeta<T>>::new(incoming, metrics, network, event, tx).spawn();

        StepHandle { handle, rx }
    }
}
