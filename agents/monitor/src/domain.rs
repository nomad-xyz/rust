use std::{collections::HashMap, sync::Arc};

use nomad_ethereum::bindings::{
    home::Home,
    replica::{ProcessFilter, Replica, UpdateFilter as RelayFilter},
};
use nomad_xyz_configuration::{contracts::CoreContracts, NomadConfig};
use tokio::sync::mpsc;

use crate::{
    annotate::WithMeta,
    between::{BetweenEvents, BetweenHandle, BetweenMetrics},
    init::provider_for,
    metrics::Metrics,
    producer::{
        DispatchProducer, DispatchProducerHandle, ProcessProducer, ProcessProducerHandle,
        RelayProducer, RelayProducerHandle, UpdateProducer, UpdateProducerHandle,
    },
    ProcessStep, Provider, StepHandle,
};

#[derive(Debug)]
pub(crate) struct Domain {
    pub(crate) network: String,
    pub(crate) home: Home<Provider>,
    pub(crate) replicas: HashMap<String, Replica<Provider>>,
}

impl Domain {
    pub(crate) fn from_config(config: &NomadConfig, network: &str) -> eyre::Result<Self> {
        let network = network.to_owned();
        let provider = provider_for(config, &network)?;

        let CoreContracts::Evm(core) = config.core().get(&network).expect("invalid config");

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
            network,
            home,
            replicas,
        })
    }

    pub(crate) fn name(&self) -> &str {
        self.network.as_ref()
    }

    pub(crate) fn home(&self) -> &Home<Provider> {
        &self.home
    }

    pub(crate) fn replicas(&self) -> &HashMap<String, Replica<Provider>> {
        &self.replicas
    }

    pub(crate) fn dispatch_producer(&self) -> DispatchProducerHandle {
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = DispatchProducer::new(self.home.clone(), self.network.clone(), tx).spawn();

        StepHandle { handle, rx }
    }

    pub(crate) fn update_producer(&self) -> UpdateProducerHandle {
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = UpdateProducer::new(self.home.clone(), self.network.clone(), tx).spawn();

        StepHandle { handle, rx }
    }

    pub fn relay_producer_for(
        &self,
        replica: &Replica<Provider>,
        replica_of: &str,
    ) -> RelayProducerHandle {
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = RelayProducer::new(replica.clone(), &self.network, replica_of, tx).spawn();
        StepHandle { handle, rx }
    }

    pub(crate) fn relay_producers(&self) -> HashMap<&str, RelayProducerHandle> {
        self.replicas()
            .iter()
            .map(|(network, replica)| {
                let producer = self.relay_producer_for(replica, network);
                (network.as_str(), producer)
            })
            .collect()
    }

    pub(crate) fn process_producer_for(
        &self,
        replica: &Replica<Provider>,
        replica_of: &str,
    ) -> ProcessProducerHandle {
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = ProcessProducer::new(replica.clone(), &self.network, replica_of, tx).spawn();
        StepHandle { handle, rx }
    }

    pub(crate) fn process_producers(&self) -> HashMap<&str, ProcessProducerHandle> {
        self.replicas()
            .iter()
            .map(|(replica_of, replica)| {
                let producer = self.process_producer_for(replica, replica_of);
                (replica_of.as_str(), producer)
            })
            .collect()
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

    pub(crate) fn count_relays(
        &self,
        metrics: Arc<Metrics>,
    ) -> HashMap<&str, BetweenHandle<WithMeta<RelayFilter>>> {
        self.relay_producers()
            .into_iter()
            .map(|(replica_of, producer)| {
                let emitter = format!(
                    "{:?}",
                    self.replicas.get(&replica_of.to_owned()).unwrap().address()
                );
                let network = &self.network;
                let event = "relay";
                tracing::info!(
                    network = self.name(),
                    replica = emitter,
                    event,
                    "starting relay counter",
                );

                let metrics = metrics.between_metrics(network, event, &emitter, Some(replica_of));

                let between = self.count(producer.rx, metrics, "relay");

                (replica_of, between)
            })
            .collect()
    }

    pub(crate) fn count_processes(
        &self,
        metrics: Arc<Metrics>,
    ) -> HashMap<&str, BetweenHandle<WithMeta<ProcessFilter>>> {
        self.process_producers()
            .into_iter()
            .map(|(replica_of, producer)| {
                let emitter = format!(
                    "{:?}",
                    self.replicas.get(&replica_of.to_owned()).unwrap().address()
                );
                let network = &self.network;
                let event = "process";
                tracing::info!(
                    network = self.name(),
                    replica = emitter,
                    event,
                    "starting process counter",
                );

                let metrics = metrics.between_metrics(network, event, &emitter, Some(replica_of));

                let between = self.count(producer.rx, metrics, "relay");

                (replica_of, between)
            })
            .collect()
    }
}
