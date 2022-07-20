use std::{collections::HashMap, fmt::Display, pin::Pin};

use futures_util::future::select_all;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{info_span, Instrument};

use crate::{bail_task_if, HomeReplicaMap, ProcessStep, Restartable, StepHandle};

// split handles from outputs
pub(crate) fn split<T, U>(
    map: HashMap<&str, StepHandle<T, U>>,
) -> (Vec<Restartable<T>>, HashMap<&str, U>)
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
pub(crate) fn nested_split<T, U>(
    map: HomeReplicaMap<StepHandle<T, U>>,
) -> (Vec<Restartable<T>>, HomeReplicaMap<U>)
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
