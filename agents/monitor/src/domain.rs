use std::{collections::HashMap, sync::Arc};

use nomad_ethereum::bindings::{home::Home, replica::Replica};
use nomad_xyz_configuration::{contracts::CoreContracts, NomadConfig};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    annotate::WithMeta,
    between::{BetweenDispatch, BetweenEvents, BetweenHandle, BetweenMetrics, BetweenUpdate},
    dispatch_wait::{DispatchWait, DispatchWaitHandle, DispatchWaitOutput},
    init::provider_for,
    metrics::Metrics,
    producer::{
        DispatchProducer, DispatchProducerHandle, ProcessProducer, ProcessProducerHandle,
        RelayProducer, RelayProducerHandle, UpdateProducer, UpdateProducerHandle,
    },
    DispatchFaucet, Faucet, NetworkMap, ProcessFaucet, ProcessStep, Provider, RelayFaucet,
    StepHandle, UpdateFaucet,
};

#[derive(Debug)]
pub(crate) struct Domain {
    pub(crate) network: String,
    pub(crate) home: Home<Provider>,
    pub(crate) replicas: HashMap<String, Replica<Provider>>,
}

impl Domain {
    pub(crate) fn home_address(&self) -> String {
        format!("{:?}", self.home.address())
    }

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
        let (tx, rx) = unbounded_channel();

        let handle = DispatchProducer::new(self.home.clone(), self.network.clone(), tx).spawn();

        StepHandle { handle, rx }
    }

    pub(crate) fn update_producer(&self) -> UpdateProducerHandle {
        let (tx, rx) = unbounded_channel();

        let handle = UpdateProducer::new(self.home.clone(), self.network.clone(), tx).spawn();

        StepHandle { handle, rx }
    }

    pub fn relay_producer_for(
        &self,
        replica: &Replica<Provider>,
        replica_of: &str,
    ) -> RelayProducerHandle {
        let (tx, rx) = unbounded_channel();

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
        let (tx, rx) = unbounded_channel();

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
        incoming: Faucet<T>,
        metrics: BetweenMetrics,
        event: impl AsRef<str>,
    ) -> BetweenHandle<WithMeta<T>>
    where
        T: 'static + Send + Sync + std::fmt::Debug,
    {
        let network = self.network.clone();
        let (tx, rx) = unbounded_channel();
        let emitter = self.home_address();

        tracing::debug!(
            network = network,
            home = emitter,
            event = event.as_ref(),
            "starting counter",
        );
        let handle =
            BetweenEvents::<WithMeta<T>>::new(incoming, metrics, network, event, emitter, tx)
                .spawn();

        StepHandle { handle, rx }
    }

    pub(crate) fn count_dispatches(
        &self,
        incoming: DispatchFaucet,
        metrics: BetweenMetrics,
        event: impl AsRef<str>,
    ) -> BetweenDispatch {
        self.count(incoming, metrics, event)
    }

    pub(crate) fn count_updates(
        &self,
        incoming: UpdateFaucet,
        metrics: BetweenMetrics,
        event: impl AsRef<str>,
    ) -> BetweenUpdate {
        self.count(incoming, metrics, event)
    }

    pub(crate) fn count_relays<'a>(
        &'a self,
        incomings: &mut NetworkMap<'a, RelayFaucet>,
        metrics: Arc<Metrics>,
    ) {
        self.replicas.iter().for_each(|(replica_of, replica)| {
            let emitter = format!("{:?}", replica.address());
            let network = self.name();
            let event = "relay";
            tracing::debug!(
                network = network,
                replica = emitter,
                event,
                "starting counter",
            );
            let incoming = incomings
                .remove(replica_of.as_str())
                .expect("Missing channel");
            let metrics = metrics.between_metrics(network, event, &emitter, Some(replica_of));

            let between = self.count(incoming, metrics, event);
            incomings.insert(replica_of, between.rx);
        });
    }

    pub(crate) fn count_processes<'a>(
        &'a self,
        incomings: &mut NetworkMap<'a, ProcessFaucet>,
        metrics: Arc<Metrics>,
    ) {
        self.replicas.iter().for_each(|(replica_of, replica)| {
            let emitter = format!("{:?}", replica.address());
            let network = self.name();
            let event = "process";
            tracing::debug!(
                network = network,
                replica = emitter,
                event,
                "starting counter",
            );
            let incoming = incomings
                .remove(replica_of.as_str())
                .expect("Missing channel");

            let metrics = metrics.between_metrics(network, event, &emitter, Some(replica_of));
            let between = self.count(incoming, metrics, event);

            incomings.insert(replica_of, between.rx);
        });
    }

    pub(crate) fn dispatch_to_update(
        &self,
        incoming_dispatch: DispatchFaucet,
        incoming_update: UpdateFaucet,
        metrics: Arc<Metrics>,
    ) -> DispatchWaitHandle {
        let metrics = metrics.dispatch_wait_metrics(&self.network, &self.home_address());

        let (outgoing_update, updates) = unbounded_channel();
        let (outgoing_dispatch, dispatches) = unbounded_channel();

        let handle = DispatchWait::new(
            incoming_dispatch,
            incoming_update,
            self.name().to_owned(),
            self.home_address(),
            metrics,
            outgoing_update,
            outgoing_dispatch,
        )
        .spawn();

        DispatchWaitHandle {
            handle,
            rx: DispatchWaitOutput {
                dispatches,
                updates,
            },
        }
    }
}
