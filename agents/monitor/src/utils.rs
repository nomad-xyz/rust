use std::{collections::HashMap, pin::Pin};

use tokio::sync::mpsc::UnboundedReceiver;

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
