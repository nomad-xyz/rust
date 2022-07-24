use tracing::{info_span, Instrument};

use crate::{annotate::WithMeta, pipe::Pipe, unwrap_pipe_item, ProcessStep, Restartable};

pub(crate) struct BetweenMetrics {
    pub(crate) count: prometheus::IntCounter,
    pub(crate) wallclock_latency: prometheus::Histogram,
    pub(crate) block_latency: prometheus::Histogram,
}

// Track time between events of the same kind
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct BetweenEvents<T> {
    pub(crate) pipe: Pipe<T>,
    pub(crate) metrics: BetweenMetrics,
    pub(crate) network: String,
    pub(crate) event: String,
    pub(crate) emitter: String,
}

impl<T> std::fmt::Display for BetweenEvents<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BetweenEvents - network {}'s {} @ {}",
            self.network, self.event, self.emitter
        )
    }
}

impl<T> std::fmt::Debug for BetweenEvents<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BetweenEvents")
            .field("network", &self.network)
            .field("event", &self.event)
            .finish()
    }
}

/// Track latency between blockchain events
impl<T> BetweenEvents<T>
where
    T: 'static + std::fmt::Debug,
{
    pub(crate) fn new(
        pipe: Pipe<T>,
        metrics: BetweenMetrics,
        network: impl AsRef<str>,
        event: impl AsRef<str>,
        emitter: impl AsRef<str>,
    ) -> Self {
        Self {
            pipe,
            metrics,
            network: network.as_ref().to_owned(),
            event: event.as_ref().to_owned(),
            emitter: emitter.as_ref().to_owned(),
        }
    }
}

pub(crate) type BetweenTask<T> = Restartable<BetweenEvents<T>>;

impl<T> ProcessStep for BetweenEvents<WithMeta<T>>
where
    T: 'static + Send + Sync + std::fmt::Debug,
{
    fn spawn(mut self) -> BetweenTask<WithMeta<T>> {
        let span = info_span!(
            target: "monitor::between",
            "LatencyMetricsTask",
            network = self.network.as_str(),
            event = self.event.as_str()
        );

        tokio::spawn(
            async move {
                let mut last_block_number = 0;
                let mut wallclock_latency = self.metrics.wallclock_latency.start_timer();

                loop {
                    // get the next event from the channel
                    let incoming = self.pipe.next().await;

                    let incoming = unwrap_pipe_item!(incoming, self);

                    tracing::debug!(
                        target: "monitor::between",
                        block_number = %incoming.meta.block_number,
                        event = self.event.as_str(),
                        "received incoming event"
                    );

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

                    // restart the timer
                    wallclock_latency = self.metrics.wallclock_latency.start_timer();
                }
            }
            .instrument(span),
        )
    }
}
