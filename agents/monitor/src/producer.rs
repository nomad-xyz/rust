use crate::{annotate::WithMeta, ProcessStep, StepHandle};

use ethers::prelude::Middleware;
use nomad_ethereum::bindings::{
    home::{DispatchFilter, Home, UpdateFilter as HomeUpdateFilter},
    replica::{ProcessFilter, Replica, UpdateFilter as ReplicaUpdateFilter},
};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::{info_span, instrument::Instrumented, Instrument};

#[derive(Debug)]
pub struct DispatchProducer {
    home: Home<crate::Provider>,
    network: String,
    tx: UnboundedSender<WithMeta<DispatchFilter>>,
}

pub(crate) type DispatchProducerHandle = Instrumented<JoinHandle<DispatchProducer>>;

impl ProcessStep<WithMeta<DispatchFilter>> for DispatchProducer {
    fn spawn(self) -> DispatchProducerHandle {
        let span = info_span!(
            "DispatchProducer",
            home = format!("{:?}", self.home.address()),
            network = self.network.as_str(),
            event = "dispatch",
        );

        tokio::spawn(async move {
            let provider = self.home.client();
            let height = provider.get_block_number().await.unwrap();
            loop {
                todo!()
            }
            self
        })
        .instrument(span)
    }
}
