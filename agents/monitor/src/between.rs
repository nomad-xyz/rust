use tracing::{info_span, Instrument};

use tokio::sync::mpsc;

use crate::{annotate::Annotated, ProcessStep, StepHandle};

// Track time between events of the same kind
pub(crate) struct BetweenEvents<T> {
    pub(crate) incoming: mpsc::UnboundedReceiver<Annotated<T>>,
    pub(crate) count: prometheus::IntCounter,
    pub(crate) wallclock_latency: prometheus::Histogram,
    pub(crate) block_latency: prometheus::Histogram,
    pub(crate) network: String,
}

/// Track latency between blockchain events
impl<T> BetweenEvents<T>
where
    T: 'static,
{
    pub(crate) fn new(
        incoming: mpsc::UnboundedReceiver<Annotated<T>>,
        count: prometheus::IntCounter,
        wallclock_latency: prometheus::Histogram,
        block_latency: prometheus::Histogram,
        network: String,
    ) -> Self {
        Self {
            incoming,
            count,
            wallclock_latency,
            block_latency,
            network,
        }
    }
}

impl<T> ProcessStep<T> for BetweenEvents<T>
where
    T: 'static + Send + Sync,
{
    fn spawn(mut self) -> StepHandle<T> {
        let span = info_span!("LatencyMetricsTask", network = self.network.as_str());

        let (outgoing, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            let mut last_block_number = 0;
            let mut wallclock_latency = self.wallclock_latency.start_timer();

            loop {
                // get the next event from the channel
                let incoming = self.incoming.recv().await;
                if incoming.is_none() {
                    break;
                }
                let incoming = incoming.unwrap();

                // calculate the blockchain-reported latency in seconds
                let block_number = incoming.meta.block_number.as_u64();
                let event_latency = block_number.saturating_sub(last_block_number);
                last_block_number = block_number;

                if event_latency != last_block_number {
                    self.block_latency.observe(event_latency as f64);
                }

                // update our metrics
                self.count.inc();
                wallclock_latency.observe_duration();

                // send the next event out
                if outgoing.send(incoming).is_err() {
                    break;
                }

                // restart the timer
                wallclock_latency = self.wallclock_latency.start_timer();
            }
        })
        .instrument(span);

        StepHandle { handle, rx }
    }
}
