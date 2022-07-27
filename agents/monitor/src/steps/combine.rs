use futures_util::future::select_all;
use std::{collections::HashMap, fmt::Display};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{info_span, Instrument};

use crate::{unwrap_channel_item_unrecoverable, utils::nexts, ProcessStep, Restartable};

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct CombineChannels<T> {
    pub(crate) faucets: HashMap<String, UnboundedReceiver<T>>,
    pub(crate) sink: UnboundedSender<(String, T)>,
}

impl<T> CombineChannels<T> {
    pub(crate) fn new(
        faucets: HashMap<String, UnboundedReceiver<T>>,
        sink: UnboundedSender<(String, T)>,
    ) -> Self {
        Self { faucets, sink }
    }

    pub(crate) fn nested(
        faucets: HashMap<String, HashMap<String, UnboundedReceiver<T>>>,
        sink: UnboundedSender<(String, (String, T))>,
    ) -> CombineChannels<(String, T)>
    where
        T: 'static + Send + Sync + std::fmt::Debug,
    {
        let faucets = faucets
            .into_iter()
            .map(|(k, v)| {
                let (sink, faucet) = unbounded_channel();
                CombineChannels::<T>::new(v, sink).run_until_panic();
                (k, faucet)
            })
            .collect();

        CombineChannels::<(String, T)>::new(faucets, sink)
    }
}

impl<T> Display for CombineChannels<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CombineChannels")
    }
}

impl<T> ProcessStep for CombineChannels<T>
where
    T: 'static + Send + Sync + std::fmt::Debug,
{
    fn spawn(mut self) -> Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!("CombineChannels");
        tokio::spawn(
            async move {
                loop {
                    let ((net, next_opt), _, _) = select_all(nexts(&mut self.faucets)).await;
                    let next = unwrap_channel_item_unrecoverable!(next_opt, self);

                    self.sink
                        .send((net, next))
                        .expect("outbound channel failed");
                }
            }
            .instrument(span),
        )
    }
}
