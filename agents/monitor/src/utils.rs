use std::{collections::HashMap, fmt::Display, pin::Pin};

use futures_util::future::select_all;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{info_span, Instrument};

use crate::{
    bail_task_if, dispatch_wait::DispatchWaitOutput, DispatchFaucet, NetworkMap, UpdateFaucet,
};

use super::{HomeReplicaMap, ProcessStep, Restartable, StepHandle};

// split handles from outputs
pub(crate) fn split<T>(
    map: HashMap<&str, StepHandle<T>>,
) -> (
    Vec<Restartable<T>>,
    HashMap<&str, <T as ProcessStep>::Output>,
)
where
    T: ProcessStep,
{
    let mut handles = vec![];
    let map = map
        .into_iter()
        .map(|(name, out)| {
            handles.push(out.handle);
            (name, out.rx)
        })
        .collect();
    (handles, map)
}

// split handles from outputs in a nested map
pub(crate) fn nested_split<T>(
    map: HomeReplicaMap<StepHandle<T>>,
) -> (
    Vec<Restartable<T>>,
    HomeReplicaMap<<T as ProcessStep>::Output>,
)
where
    T: ProcessStep,
{
    let mut handles = vec![];
    let map = map
        .into_iter()
        .map(|(name, map)| {
            let (mut h, m) = split(map);
            handles.append(&mut h);
            (name, m)
        })
        .collect();
    (handles, map)
}

// polls all channels in a hashmap
pub(crate) fn nexts<K: ToOwned, T>(
    map: &mut HashMap<K, UnboundedReceiver<T>>,
) -> Vec<Pin<Box<impl std::future::Future<Output = (<K as ToOwned>::Owned, Option<T>)> + '_>>> {
    map.iter_mut()
        .map(|(k, rx)| {
            let k = k.to_owned();
            let fut = rx.recv();
            async move { (k, fut.await) }
        })
        .map(Box::pin)
        .collect()
}

#[derive(Debug)]
pub(crate) struct SelectChannels<T> {
    channels: HashMap<String, UnboundedReceiver<T>>,
    outbound: UnboundedSender<(String, T)>,
}

impl<T> SelectChannels<T> {
    pub(crate) fn new(
        channels: HashMap<String, UnboundedReceiver<T>>,
        outbound: UnboundedSender<(String, T)>,
    ) -> Self {
        Self { channels, outbound }
    }
}

impl<T> Display for SelectChannels<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectChannels")
    }
}

impl<T> ProcessStep for SelectChannels<T>
where
    T: 'static + Send + Sync + std::fmt::Debug,
{
    type Output = UnboundedReceiver<(String, T)>;

    fn spawn(mut self) -> Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!("SelectChannels");
        tokio::spawn(
            async move {
                loop {
                    let ((net, next_opt), _, _) = select_all(nexts(&mut self.channels)).await;
                    bail_task_if!(
                        next_opt.is_none(),
                        self,
                        format!("Inbound from {} broke", net),
                    );
                    let next = next_opt.expect("checked");
                    bail_task_if!(
                        self.outbound.send((net, next)).is_err(),
                        self,
                        "outbound channel broke"
                    );
                }
            }
            .instrument(span),
        )
    }
}

pub(crate) fn split_dispatch_wait_output(
    mut map: HashMap<&str, DispatchWaitOutput>,
) -> (NetworkMap<DispatchFaucet>, NetworkMap<UpdateFaucet>) {
    let mut dispatches = HashMap::new();
    let mut updates = HashMap::new();

    map.drain().for_each(|(k, v)| {
        dispatches.insert(k, v.dispatches);
        updates.insert(k, v.updates);
    });
    (dispatches, updates)
}
