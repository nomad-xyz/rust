use std::{collections::HashMap, pin::Pin};

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{HomeReplicaMap, ProcessStep, Restartable, StepHandle};

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
