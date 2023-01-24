use crate::k8s::LifeguardPod;
use crate::metrics::metrics::Metrics;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::prelude::*;
use chrono::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, instrument};

/// Structure that controls 2 types of backoffs.
/// * Hard limit is protecting from restarting a pod
///     more than a specific amount of times `max_restarts`
///     per some amount of time `hard_limit`. Typically,
///     the limit would be 5 restarts per day
/// * Soft limit prevents from multiple restarts in a row.
///     `soft_limit` usually is some little time between attempts,
///     like 1 minute.
#[derive(Debug)]
pub struct RestartBackoff {
    /// Minimum time between each restart
    soft_limit: Duration,
    /// Duration which considers max_restarts
    hard_limit: Duration,
    /// Max restarts per hard limit
    max_restarts: u32,
    metrics: Arc<Metrics>,
    /// Map that holds timestamps of previous restarts for every different agent
    db: Arc<Mutex<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl RestartBackoff {
    pub fn new(
        max_restarts: u32,
        soft_limit: Option<Duration>,
        hard_limit: Option<Duration>,
        metrics: Arc<Metrics>,
    ) -> Self {
        if let Some(soft_limit) = soft_limit {
            if soft_limit < Duration::seconds(3) {
                panic!("Soft limit cannot be less than 5 seconds");
            }
        }

        if let Some(hard_limit) = hard_limit {
            if hard_limit < Duration::minutes(1) {
                panic!("Hard limit cannot be less than 1 minute");
            }
        }
        if max_restarts <= 0 {
            panic!("Max restarts cannot be less than 1");
        }

        Self {
            soft_limit: soft_limit.unwrap_or(Duration::seconds(30)),
            hard_limit: hard_limit.unwrap_or(Duration::days(1)),
            max_restarts,
            metrics,
            db: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Increment internal "counter" of retries that obey hard limit
    #[instrument]
    async fn add(&self, pod: &LifeguardPod) {
        let s = pod.to_string();
        let now = Utc::now();
        let mut db = self.db.lock().await;
        if let Some(dates) = db.get_mut(&s) {
            dates.push(now);
        } else {
            db.insert(s, vec![now]);
        }

        debug!(pod = ?pod, timestamp = ?now, "Added timestamp");
    }

    /*
    Returns None if gtg, else Some(time), which is the next earliest
     */

    /// Attempts to increment the internal "counter". If successful - returns `None`,
    /// though if hits soft or hard backoff limit - returns `Some(DateTime<Utc>)`,
    /// where internal value is the date-time of next earliest possible attempt.
    #[instrument]
    pub async fn inc(&self, pod: &LifeguardPod) -> Option<DateTime<Utc>> {
        let s = pod.to_string();
        let now = Utc::now();
        let latest_relevant = now - self.hard_limit;

        if let Some(timestamps) = self.db.lock().await.get_mut(&s) {
            timestamps.retain_mut(|x| *x >= latest_relevant);

            debug!(pod = ?pod, timestamps = timestamps.len(), "Found previous timestamps");

            // Soft limit check
            if timestamps.len() > 0 {
                let latest_timestamp = timestamps.iter().max();
                if let Some(latest_timestamp) = latest_timestamp {
                    let next_attempt = *latest_timestamp + self.soft_limit;
                    if next_attempt > now {
                        error!(pod = ?pod, next_attempt = ?next_attempt, "Hit soft limit in backoff");
                        self.metrics.backoffs_inc(
                            "soft",
                            &pod.network.to_string(),
                            &pod.agent.to_string(),
                        );
                        return Some(next_attempt);
                    }
                }
            }

            // Hard limit check
            if timestamps.len() > self.max_restarts as usize {
                let oldest_timestamp = timestamps.iter().min();

                if let Some(oldest_timestamp) = oldest_timestamp {
                    let next_attempt = *oldest_timestamp + self.hard_limit;
                    error!(pod = ?pod, next_attempt = ?next_attempt, "Hit hard limit in backoff");
                    // metric backoff soft limit
                    self.metrics.backoffs_inc(
                        "hard",
                        &pod.network.to_string(),
                        &pod.agent.to_string(),
                    );
                    return Some(next_attempt);
                }
            }
        }

        self.add(pod).await;
        None
    }
}
