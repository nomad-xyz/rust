use std::collections::HashMap;

use tokio::{sync::mpsc, task::JoinHandle};

pub type Restartable<Task> = JoinHandle<crate::TaskResult<Task>>;

pub type Faucet<T> = mpsc::UnboundedReceiver<T>;
pub type Sink<T> = mpsc::UnboundedSender<T>;

pub type NetworkMap<'a, T> = HashMap<&'a str, T>;
pub type HomeReplicaMap<'a, T> = HashMap<&'a str, std::collections::HashMap<&'a str, T>>;
