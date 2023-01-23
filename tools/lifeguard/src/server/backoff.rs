use std::collections::HashMap;

use crate::k8s::LifeguardPod;
use chrono::prelude::*;
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, instrument};

#[derive(Debug)]
enum BackoffError {
    ZeroMaxRestarts,
}

impl std::error::Error for BackoffError {}

impl std::fmt::Display for BackoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BackoffError::ZeroMaxRestarts => {
                write!(f, "Max retries for the backoff cannot be equal to zero")
            }
        }
    }
}

#[derive(Debug)]
pub struct RestartBackoff {
    // Minimum time between each restart
    soft_limit: Duration,
    // Duration which considers max_restarts
    hard_limit: Duration,
    max_restarts: u32,
    db: Arc<Mutex<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl RestartBackoff {
    pub fn new(
        max_restarts: u32,
        soft_limit: Option<Duration>,
        hard_limit: Option<Duration>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if max_restarts > 0 {
            Ok(Self {
                soft_limit: soft_limit.unwrap_or(Duration::seconds(30)),
                hard_limit: hard_limit.unwrap_or(Duration::days(1)),
                max_restarts,
                db: Arc::new(Mutex::new(HashMap::new())),
            })
        } else {
            Err(Box::new(&BackoffError::ZeroMaxRestarts))
        }
    }

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

    #[instrument]
    pub async fn inc(&self, pod: &LifeguardPod) -> Option<DateTime<Utc>> {
        let s = pod.to_string();
        let now = Utc::now();
        let latest_relevant = now - self.hard_limit;

        if let Some(timestamps) = self.db.lock().await.get_mut(&s) {
            timestamps.retain_mut(|x| *x >= latest_relevant);

            debug!(pod = ?pod, timestamps = timestamps.len(), "Found previous timestamps");

            if timestamps.len() > 0 {
                let latest_timestamp = timestamps.iter().max();
                if let Some(latest_timestamp) = latest_timestamp {
                    let next_attempt = *latest_timestamp + self.soft_limit;
                    if next_attempt > now {
                        error!(pod = ?pod, next_attempt = ?next_attempt, "Hit soft limit in backoff");
                        return Some(next_attempt);
                    }
                }
            }

            if timestamps.len() > self.max_restarts as usize {
                let oldest_timestamp = timestamps.iter().min();

                if let Some(oldest_timestamp) = oldest_timestamp {
                    let next_attempt = *oldest_timestamp + self.hard_limit;
                    error!(pod = ?pod, next_attempt = ?next_attempt, "Hit hard limit in backoff");
                    return Some(next_attempt);
                }
            }
        }

        self.add(pod).await;
        None
    }
}
