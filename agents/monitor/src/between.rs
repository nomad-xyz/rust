use nomad_ethereum::bindings::home::DispatchFilter;
use tracing::{info_span, instrument::Instrumented, Instrument};

use tokio::{sync::mpsc, task::JoinHandle};

use crate::{annotate::WithMeta, task_bail_if, ProcessStep, Restartable, StepHandle};

#[derive(Debug)]
pub(crate) struct BetweenMetrics {
    pub(crate) count: prometheus::IntCounter,
    pub(crate) wallclock_latency: prometheus::Histogram,
    pub(crate) block_latency: prometheus::Histogram,
}

// Track time between events of the same kind
#[derive(Debug)]
pub(crate) struct BetweenEvents<T>
where
    T: std::fmt::Debug,
{
    pub(crate) incoming: mpsc::UnboundedReceiver<T>,
    pub(crate) metrics: BetweenMetrics,
    pub(crate) network: String,
}

/// Track latency between blockchain events
impl<T> BetweenEvents<T>
where
    T: 'static + std::fmt::Debug,
{
    pub(crate) fn new(
        incoming: mpsc::UnboundedReceiver<T>,
        metrics: BetweenMetrics,
        network: String,
    ) -> Self {
        Self {
            incoming,
            metrics,
            network,
        }
    }
}

pub(crate) type BetweenHandle<T> = Restartable<BetweenEvents<T>>;

impl<T> ProcessStep<WithMeta<T>> for BetweenEvents<WithMeta<T>>
where
    T: 'static + Send + Sync + std::fmt::Debug,
{
    fn spawn(mut self) -> BetweenHandle<WithMeta<T>> {
        let span = info_span!("LatencyMetricsTask", network = self.network.as_str());

        let (outgoing, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut last_block_number = 0;
            let mut wallclock_latency = self.metrics.wallclock_latency.start_timer();

            loop {
                // get the next event from the channel
                let incoming = self.incoming.recv().await;
                task_bail_if!(incoming.is_none(), self, "inbound channel broke");

                let incoming = incoming.unwrap();

                // calculate the blockchain-reported latency in seconds
                let block_number = incoming.meta.block_number.as_u64();
                let event_latency = block_number.saturating_sub(last_block_number);
                last_block_number = block_number;

                if event_latency != last_block_number {
                    self.metrics.block_latency.observe(event_latency as f64);
                }

                // update our metrics
                self.metrics.count.inc();
                wallclock_latency.observe_duration();

                // send the next event out
                task_bail_if!(
                    outgoing.send(incoming).is_err(),
                    self,
                    "outbound channel broke"
                );

                // restart the timer
                wallclock_latency = self.metrics.wallclock_latency.start_timer();
            }
        })
        .instrument(span)
    }
}
