use ethers::prelude::{Middleware, U64};
use nomad_ethereum::bindings::{
    home::{DispatchFilter, Home, UpdateFilter},
    replica::{ProcessFilter, Replica, UpdateFilter as RelayFilter},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info_span, trace, Instrument};

use crate::{
    annotate::WithMeta, bail_task_if, send_unrecoverable, steps::noisy_sleep,
    unwrap_result_recoverable, DispatchSink, ProcessStep, Restartable,
};

pub const POLLING_INTERVAL_MILLIS: u64 = 5000;
pub const BEHIND_TIP: u64 = 5;

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct DispatchProducer {
    home: Home<crate::Provider>,
    network: String,
    from: Option<U64>,
    tx: DispatchSink,
}

impl std::fmt::Display for DispatchProducer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DispatchProducer - {}'s home @ {}",
            self.network,
            self.home_address()
        )
    }
}

impl DispatchProducer {
    pub(crate) fn home_address(&self) -> String {
        format!("{:?}", self.home.address())
    }

    pub(crate) fn new(
        home: Home<crate::Provider>,
        network: impl AsRef<str>,
        tx: UnboundedSender<WithMeta<DispatchFilter>>,
    ) -> Self {
        Self {
            home,
            network: network.as_ref().to_owned(),
            from: None,
            tx,
        }
    }
}

pub(crate) type DispatchProducerTask = Restartable<DispatchProducer>;

impl ProcessStep for DispatchProducer {
    fn spawn(mut self) -> DispatchProducerTask {
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
                let mut to = height - BEHIND_TIP;
                loop {
                    let from = self.from.unwrap_or(height - (BEHIND_TIP * 2));
                    if from < to {
                        trace!(from = %from, to = %to, progress = %(to - from), "querying dispatch events");
                        let res = self
                            .home
                            .dispatch_filter()
                            .from_block(from)
                            .to_block(to)
                            .query_with_meta()
                            .await;

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        let events = res.unwrap();

                        if !events.is_empty() {
                            trace!(
                                count = events.len(),
                                from = %from,
                                to = %to,
                                progress = %(to - from),
                                "Received dispatch events"
                            );
                        }

                        for event in events.into_iter() {
                            send_unrecoverable!(self.tx, event.into(), self);
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    let tip = unwrap_result_recoverable!(tip_res, self) - BEHIND_TIP;

                    self.from = Some(to + 1);
                    to = std::cmp::max(to, tip);

                    noisy_sleep(POLLING_INTERVAL_MILLIS).await;
                }
            }
            .instrument(span),
        )
    }
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct UpdateProducer {
    home: Home<crate::Provider>,
    network: String,
    from: Option<U64>,
    tx: UnboundedSender<WithMeta<UpdateFilter>>,
}

impl std::fmt::Display for UpdateProducer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UpdateProducer - {}'s home @ {}",
            self.network,
            self.home_address()
        )
    }
}

impl UpdateProducer {
    pub(crate) fn home_address(&self) -> String {
        format!("{:?}", self.home.address())
    }

    pub(crate) fn new(
        home: Home<crate::Provider>,
        network: impl AsRef<str>,
        tx: UnboundedSender<WithMeta<UpdateFilter>>,
    ) -> Self {
        Self {
            home,
            network: network.as_ref().to_owned(),
            from: None,
            tx,
        }
    }
}

pub(crate) type UpdateProducerTask = Restartable<UpdateProducer>;

impl ProcessStep for UpdateProducer {
    fn spawn(mut self) -> UpdateProducerTask {
        let span = info_span!(
            "UpdateProducer",
            home = format!("{:?}", self.home.address()),
            network = self.network.as_str(),
            event = "update",
        );

        tokio::spawn(
            async move {
                let provider = self.home.client();
                let height = provider.get_block_number().await;

                bail_task_if!(height.is_err(), self, "Err retrieving height");
                let height = height.expect("checked");
                let mut to = height - BEHIND_TIP;
                loop {
                    let from = self.from.unwrap_or(height - (BEHIND_TIP * 2));
                    if from < to {
                        trace!(from = %from, to = %to, progress = %(to - from), "querying update events");
                        let res = self
                            .home
                            .update_filter()
                            .from_block(from)
                            .to_block(to)
                            .query_with_meta()
                            .await;

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        let events = res.unwrap();

                        if !events.is_empty() {
                            trace!(
                                count = events.len(),
                                from = %from,
                                to = %to,
                                progress = %(to - from),
                                "Received update events"
                            );
                        }

                        for event in events.into_iter() {
                            send_unrecoverable!(self.tx, event.into(), self);
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    let tip = unwrap_result_recoverable!(tip_res, self) - BEHIND_TIP;

                    self.from = Some(to + 1);
                    to = std::cmp::max(to, tip);

                    noisy_sleep(POLLING_INTERVAL_MILLIS).await;
                }
            }
            .instrument(span),
        )
    }
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct RelayProducer {
    replica: Replica<crate::Provider>,
    network: String,
    from: Option<U64>,
    replica_of: String,
    tx: UnboundedSender<WithMeta<RelayFilter>>,
}

impl std::fmt::Display for RelayProducer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RelayProducer - {}'s replica of {} @ {}",
            self.network,
            self.replica_of,
            self.replica_address()
        )
    }
}

impl RelayProducer {
    pub(crate) fn replica_address(&self) -> String {
        format!("{:?}", self.replica.address())
    }

    pub(crate) fn new(
        replica: Replica<crate::Provider>,
        network: impl AsRef<str>,
        replica_of: impl AsRef<str>,
        tx: UnboundedSender<WithMeta<RelayFilter>>,
    ) -> Self {
        Self {
            replica,
            network: network.as_ref().to_owned(),
            from: None,
            replica_of: replica_of.as_ref().to_owned(),
            tx,
        }
    }
}

pub(crate) type RelayProducerTask = Restartable<RelayProducer>;

impl ProcessStep for RelayProducer {
    fn spawn(mut self) -> RelayProducerTask {
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
                let mut to = height - BEHIND_TIP;
                loop {
                    let from = self.from.unwrap_or(height - (BEHIND_TIP * 2));
                    if from < to {
                        trace!(from = %from, to = %to, progress = %(to - from), "querying relay events");
                        let res = self
                            .replica
                            .update_filter()
                            .from_block(from)
                            .to_block(to)
                            .query_with_meta()
                            .await;

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        let events = res.unwrap();

                        if !events.is_empty() {
                            trace!(
                                count = events.len(),
                                from=%from,
                                to = %to,
                                "Received relay events"
                            );
                        }

                        for event in events.into_iter() {
                            send_unrecoverable!(self.tx, event.into(), self);
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    let tip = unwrap_result_recoverable!(tip_res, self) - BEHIND_TIP;

                    self.from = Some(to + 1);
                    to = std::cmp::max(to, tip);

                    noisy_sleep(POLLING_INTERVAL_MILLIS).await;
                }
            }
            .instrument(span),
        )
    }
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct ProcessProducer {
    replica: Replica<crate::Provider>,
    network: String,
    from: Option<U64>,
    replica_of: String,
    tx: UnboundedSender<WithMeta<ProcessFilter>>,
}

impl std::fmt::Display for ProcessProducer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProcessProducer for {}'s replica of {} @ {}",
            self.network,
            self.replica_of,
            self.replica_address()
        )
    }
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
            from: None,
            replica_of: replica_of.as_ref().to_owned(),
            tx,
        }
    }

    pub(crate) fn replica_address(&self) -> String {
        format!("{:?}", self.replica.address())
    }
}

pub(crate) type ProcessProducerTask = Restartable<ProcessProducer>;

impl ProcessStep for ProcessProducer {
    fn spawn(mut self) -> ProcessProducerTask {
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
                let mut to = height - BEHIND_TIP;
                loop {
                    let from = self.from.unwrap_or(height - (BEHIND_TIP * 2));
                    if from < to {
                        trace!(from = %from, to = %to, progress = %(to - from), "querying process events");
                        let res = self
                            .replica
                            .process_filter()
                            .from_block(from)
                            .to_block(to)
                            .query_with_meta()
                            .await;

                        bail_task_if!(res.is_err(), self, res.unwrap_err());

                        let events = res.unwrap();

                        if !events.is_empty() {
                            trace!(
                                count = events.len(),
                                from = %from,
                                to = %to,
                                progress = %(to - from),
                                "Received process events"
                            );
                        }

                        for event in events.into_iter() {
                            send_unrecoverable!(self.tx, event.into(), self);
                        }
                    }
                    let tip_res = provider.get_block_number().await;
                    let tip = unwrap_result_recoverable!(tip_res, self) - BEHIND_TIP;

                    self.from = Some(to + 1);
                    to = std::cmp::max(to, tip);

                    noisy_sleep(POLLING_INTERVAL_MILLIS).await;
                }
            }
            .instrument(span),
        )
    }
}
