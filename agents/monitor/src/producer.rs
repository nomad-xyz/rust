use crate::{annotate::WithMeta, bail_task_if, ProcessStep, Restartable, StepHandle};

use ethers::prelude::Middleware;
use nomad_ethereum::bindings::{
    home::{DispatchFilter, Home, UpdateFilter},
    replica::{ProcessFilter, Replica, UpdateFilter as RelayFilter},
};
use tokio::sync::mpsc::UnboundedSender;
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
        network: impl AsRef<str>,
        tx: UnboundedSender<WithMeta<DispatchFilter>>,
    ) -> Self {
        Self {
            home,
            network: network.as_ref().to_owned(),
            tx,
        }
    }
}

pub(crate) type DispatchProducerTask = Restartable<DispatchProducer>;
pub(crate) type DispatchProducerHandle = StepHandle<DispatchProducer, WithMeta<DispatchFilter>>;

impl ProcessStep<WithMeta<DispatchFilter>> for DispatchProducer {
    fn spawn(self) -> DispatchProducerTask {
        let span = info_span!(
            "DispatchProducer",
            home = format!("{:?}", self.home.address()),
            network = self.network.as_str(),
            event = "dispatch",
        );

        tokio::spawn(
            async move {
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

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        for event in res.unwrap().into_iter() {
                            let res = self.tx.send(event.into());
                            bail_task_if!(res.is_err(), self, res.unwrap_err());
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    bail_task_if!(tip_res.is_err(), self, tip_res.unwrap_err());

                    let tip = tip_res.unwrap() - 5;
                    from = to;
                    to = std::cmp::max(to, tip);

                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
            .instrument(span),
        )
    }
}

#[derive(Debug)]
pub(crate) struct UpdateProducer {
    home: Home<crate::Provider>,
    network: String,
    tx: UnboundedSender<WithMeta<UpdateFilter>>,
}

impl UpdateProducer {
    pub(crate) fn new(
        home: Home<crate::Provider>,
        network: impl AsRef<str>,
        tx: UnboundedSender<WithMeta<UpdateFilter>>,
    ) -> Self {
        Self {
            home,
            network: network.as_ref().to_owned(),
            tx,
        }
    }
}

pub(crate) type UpdateProducerTask = Restartable<UpdateProducer>;
pub(crate) type UpdateProducerHandle = StepHandle<UpdateProducer, WithMeta<UpdateFilter>>;

impl ProcessStep<WithMeta<UpdateFilter>> for UpdateProducer {
    fn spawn(self) -> UpdateProducerTask {
        let span = info_span!(
            "UpdateProducer",
            home = format!("{:?}", self.home.address()),
            network = self.network.as_str(),
            event = "update",
        );

        tokio::spawn(
            async move {
                let provider = self.home.client();
                let height = provider.get_block_number().await.unwrap();
                let mut from = height - 10;
                let mut to = height - 5;
                loop {
                    if from < to {
                        let res = self
                            .home
                            .update_filter()
                            .from_block(from)
                            .to_block(to)
                            .query_with_meta()
                            .await;

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        for event in res.unwrap().into_iter() {
                            let res = self.tx.send(event.into());
                            bail_task_if!(res.is_err(), self, res.unwrap_err());
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    bail_task_if!(tip_res.is_err(), self, tip_res.unwrap_err());

                    let tip = tip_res.unwrap() - 5;
                    from = to;
                    to = std::cmp::max(to, tip);

                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
            .instrument(span),
        )
    }
}

#[derive(Debug)]
pub(crate) struct RelayProducer {
    replica: Replica<crate::Provider>,
    network: String,
    replica_of: String,
    tx: UnboundedSender<WithMeta<RelayFilter>>,
}

impl RelayProducer {
    pub(crate) fn new(
        replica: Replica<crate::Provider>,
        network: impl AsRef<str>,
        replica_of: impl AsRef<str>,
        tx: UnboundedSender<WithMeta<RelayFilter>>,
    ) -> Self {
        Self {
            replica,
            network: network.as_ref().to_owned(),
            replica_of: replica_of.as_ref().to_owned(),
            tx,
        }
    }
}

pub(crate) type RelayProducerTask = Restartable<RelayProducer>;
pub(crate) type RelayProducerHandle = StepHandle<RelayProducer, WithMeta<RelayFilter>>;

impl ProcessStep<WithMeta<RelayFilter>> for RelayProducer {
    fn spawn(self) -> RelayProducerTask {
        let span = info_span!(
            "RelayProducer",
            replica = format!("{:?}", self.replica.address()),
            network = self.network.as_str(),
            event = "relay",
            replica_of = self.replica_of.as_str()
        );

        tokio::spawn(
            async move {
                let provider = self.replica.client();
                let height = provider.get_block_number().await.unwrap();
                let mut from = height - 10;
                let mut to = height - 5;
                loop {
                    tracing::trace!(from = from.as_u64(), to = to.as_u64(), "produce_loop");
                    if from < to {
                        let res = self
                            .replica
                            .update_filter()
                            .from_block(from)
                            .to_block(to)
                            .query_with_meta()
                            .await;

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        for event in res.unwrap().into_iter() {
                            let res = self.tx.send(event.into());
                            bail_task_if!(res.is_err(), self, res.unwrap_err());
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    bail_task_if!(tip_res.is_err(), self, tip_res.unwrap_err());
                    let tip = tip_res.unwrap() - 5;
                    from = to;
                    to = std::cmp::max(to, tip);

                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
            .instrument(span),
        )
    }
}

#[derive(Debug)]
pub(crate) struct ProcessProducer {
    replica: Replica<crate::Provider>,
    network: String,
    replica_of: String,
    tx: UnboundedSender<WithMeta<ProcessFilter>>,
}

impl ProcessProducer {
    pub(crate) fn new(
        replica: Replica<crate::Provider>,
        network: impl AsRef<str>,
        replica_of: impl AsRef<str>,
        tx: UnboundedSender<WithMeta<ProcessFilter>>,
    ) -> Self {
        Self {
            replica,
            network: network.as_ref().to_owned(),
            replica_of: replica_of.as_ref().to_owned(),
            tx,
        }
    }
}

pub(crate) type ProcessProducerTask = Restartable<ProcessProducer>;
pub(crate) type ProcessProducerHandle = StepHandle<ProcessProducer, WithMeta<ProcessFilter>>;

impl ProcessStep<WithMeta<ProcessFilter>> for ProcessProducer {
    fn spawn(self) -> ProcessProducerTask {
        let span = info_span!(
            "ProcessProducer",
            replica = format!("{:?}", self.replica.address()),
            network = self.network.as_str(),
            event = "process",
            replica_of = self.replica_of.as_str(),
        );

        tokio::spawn(
            async move {
                let provider = self.replica.client();
                let height = provider.get_block_number().await.unwrap();
                let mut from = height - 10;
                let mut to = height - 5;
                loop {
                    if from < to {
                        let res = self
                            .replica
                            .process_filter()
                            .from_block(from)
                            .to_block(to)
                            .query_with_meta()
                            .await;

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        for event in res.unwrap().into_iter() {
                            let res = self.tx.send(event.into());
                            bail_task_if!(res.is_err(), self, res.unwrap_err());
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    bail_task_if!(tip_res.is_err(), self, tip_res.unwrap_err());

                    let tip = tip_res.unwrap() - 5;
                    from = to;
                    to = std::cmp::max(to, tip);

                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
            .instrument(span),
        )
    }
}
