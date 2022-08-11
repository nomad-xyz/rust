use std::{collections::HashMap, pin::Pin};

use ethers::prelude::rand::{rngs::ThreadRng, Rng};
use tokio::sync::mpsc::UnboundedReceiver;

// polls all channels in a hashmap
pub fn nexts<K: ToOwned, T>(
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

// adds up to a second of random delay to cause production tasks to not be synced
pub fn noisy_sleep(approx_millis: u64) -> tokio::time::Sleep {
    let noise = ThreadRng::default().gen_range(0..1000u64);
    let duration = std::time::Duration::from_millis(approx_millis + noise);
    tokio::time::sleep(duration)
}
