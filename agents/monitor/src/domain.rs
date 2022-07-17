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
    annotate::Annotated, between::BetweenEvents, init::provider_for, ArcProvider, ProcessStep,
    Provider, StepHandle,
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
    pub(crate) name: String,
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
            name,
            provider,
            home,
            replicas,
        })
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_ref()
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

    pub(crate) fn update_filter(&self) -> Event<Provider, HomeUpdateFilter> {
        self.home.update_filter()
    }

    pub(crate) fn relay_filters(&self) -> HashMap<&str, Event<Provider, ReplicaUpdateFilter>> {
        self.replicas
            .iter()
            .map(|(k, v)| (k.as_str(), v.update_filter()))
            .collect()
    }

    pub(crate) fn process_filters(&self) -> HashMap<&str, Event<Provider, ProcessFilter>> {
        self.replicas
            .iter()
            .map(|(k, v)| (k.as_str(), v.process_filter()))
            .collect()
    }

    pub(crate) fn dispatch_stream(&self) -> StepHandle<DispatchFilter> {
        let home = self.home.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        let name = self.name.clone();

        let span = info_span!("dispatch stream convert loop", name = name.as_str());

        let handle = tokio::spawn(async move {
            let filter = home.dispatch_filter();
            let mut stream = filter
                .stream()
                .await
                .expect("unable to get dispatch stream");
            loop {
                let event = stream.next().await;
                let event = unwrap_event_stream_item!(event, name, "dispatch");
                tx.send(event).unwrap();
            }
        })
        .instrument(span);

        todo!("use new stream_with_meta")
        // StepHandle { handle, rx }
    }

    pub(crate) fn count<T>(
        &self,
        incoming: mpsc::UnboundedReceiver<Annotated<T>>,
        count: IntCounter,
        wallclock_latency: Histogram,
        timestamp_latency: Histogram,
    ) -> StepHandle<T>
    where
        T: 'static + Send + Sync,
    {
        BetweenEvents::new(
            incoming,
            count,
            wallclock_latency,
            timestamp_latency,
            self.name.clone(),
        )
        .spawn()
    }

    // fn update_stream(&self) -> mpsc::UnboundedReceiver<HomeUpdateFilter> {
    //     let home = self.home.clone();
    //     let (tx, rx) = mpsc::unbounded_channel();
    //     let name = self.name.clone();

    //     tokio::spawn(async move {
    //         let filter = home.update_filter();
    //         let mut stream = filter.stream().await.expect("unable to get update stream");
    //         loop {
    //             let event = stream.next().await;
    //             let event = unwrap_event_stream_item!(event, name, "update");
    //             tx.send(event).unwrap();
    //         }
    //     });

    //     rx
    // }
}
