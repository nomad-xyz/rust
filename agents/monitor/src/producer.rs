use crate::{annotate::WithMeta, task_bail_if, ProcessStep, Restartable, StepHandle};

use ethers::prelude::Middleware;
use nomad_ethereum::bindings::{
    home::{DispatchFilter, Home, UpdateFilter as HomeUpdateFilter},
    replica::{ProcessFilter, Replica, UpdateFilter as ReplicaUpdateFilter},
};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::{info_span, Instrument};

#[derive(Debug)]
pub(crate) struct DispatchProducer {
    home: Home<crate::Provider>,
    network: String,
    tx: UnboundedSender<WithMeta<DispatchFilter>>,
}

impl DispatchProducer {
    pub(crate) fn new(
        home: Home<crate::Provider>,
        network: String,
        tx: UnboundedSender<WithMeta<DispatchFilter>>,
    ) -> Self {
        Self { home, network, tx }
    }
}

pub(crate) type DispatchProducerTask = Restartable<DispatchProducer>;
pub(crate) type DispatchProducerHandle = StepHandle<DispatchProducer, DispatchFilter>;

impl ProcessStep<WithMeta<DispatchFilter>> for DispatchProducer {
    fn spawn(self) -> DispatchProducerTask {
        let span = info_span!(
            "DispatchProducer",
            home = format!("{:?}", self.home.address()),
            network = self.network.as_str(),
            event = "dispatch",
        );

        tokio::spawn(async move {
            let provider = self.home.client();
            let height = provider.get_block_number().await.unwrap();
            let mut from = height - 10;
            let mut to = height - 5;
            loop {
                if from < to {
                    let res = self
                        .home
                        .dispatch_filter()
                        .from_block(from)
                        .to_block(to)
                        .query_with_meta()
                        .await;

                    task_bail_if!(res.is_err(), self, res.unwrap_err());

                    for event in res.unwrap().into_iter() {
                        let res = self.tx.send(event.into());
                        task_bail_if!(res.is_err(), self, res.unwrap_err());
                    }
                }
                let tip = provider.get_block_number().await.unwrap() - 5;
                from = to;
                to = std::cmp::min(to, tip);

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        })
        .instrument(span)
    }
}
