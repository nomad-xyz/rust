use tokio::task::JoinHandle;
use tracing::{info_span, instrument::Instrumented, Instrument};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}

/// Has timestamp
pub trait NomadEvent: Send + Sync {
    /// Get the timestamp
    fn timestamp(&self) -> u32;

    /// block number
    fn block_number(&self) -> u64;

    /// tx hash
    fn tx_hash(&self) -> ethers::types::H256;
}

// Simple struct
pub struct BetweenEvents<T> {
    incoming: tokio::sync::mpsc::Receiver<T>,
    outgoing: tokio::sync::mpsc::Sender<T>,
    count: prometheus::IntCounter,
    wallclock_latency: prometheus::Histogram,
    timestamp_latency: prometheus::Histogram,
    network: String,
}

/// Track latency between blockchain events
impl<T> BetweenEvents<T>
where
    T: NomadEvent + 'static,
{
    pub fn new(
        incoming: tokio::sync::mpsc::Receiver<T>,
        outgoing: tokio::sync::mpsc::Sender<T>,
        count: prometheus::IntCounter,
        wallclock_latency: prometheus::Histogram,
        timestamp_latency: prometheus::Histogram,
        network: String,
    ) -> Self {
        Self {
            incoming,
            outgoing,
            count,
            wallclock_latency,
            timestamp_latency,
            network,
        }
    }

    pub fn spawn(mut self) -> Instrumented<JoinHandle<()>> {
        let span = info_span!("LatencyMetricsTask", network = self.network.as_str());

        tokio::spawn(async move {
            let mut last_timestamp = 0;
            let mut wallclock_latency = self.wallclock_latency.start_timer();

            loop {
                // get the next event from the channel
                let incoming = self.incoming.recv().await;
                if incoming.is_none() {
                    break;
                }
                let incoming = incoming.unwrap();

                // calculate the blockchain-reported latency in seconds
                let timestamp = incoming.timestamp();
                let event_latency = timestamp.saturating_sub(last_timestamp);
                last_timestamp = timestamp;

                // update our metrics
                self.count.inc();
                self.timestamp_latency.observe(event_latency as f64);
                wallclock_latency.observe_duration();

                // send the next event out
                if self.outgoing.send(incoming).await.is_err() {
                    break;
                }

                // restart the timer
                wallclock_latency = self.wallclock_latency.start_timer();
            }
        })
        .instrument(span)
    }
}
