use std::collections::HashMap;

use crate::k8s::LifeguardPod;
use chrono::prelude::*;
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

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

pub struct RestartBackoff {
    // Minimum time between each restart
    soft_duration: Duration,
    // Duration which considers max_restarts
    duration: Duration,
    max_restarts: u32,
    db: Arc<Mutex<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl RestartBackoff {
    pub fn new(max_restarts: u32) -> Result<Self, Box<dyn std::error::Error>> {
        if max_restarts > 0 {
            Ok(Self {
                soft_duration: Duration::seconds(20),
                duration: Duration::days(1),
                max_restarts,
                db: Arc::new(Mutex::new(HashMap::new())),
            })
        } else {
            Err(Box::new(&BackoffError::ZeroMaxRestarts))
        }
    }

    async fn add(&self, pod: &LifeguardPod) {
        let s = pod.to_string();
        let now = Utc::now();
        let mut db = self.db.lock().await;
        if let Some(dates) = db.get_mut(&s) {
            dates.push(now);
        } else {
            db.insert(s, vec![now]);
        }
    }

    /*
    Returns None if gtg, else Some(time), which is the next earliest
     */
    pub async fn inc(&self, pod: &LifeguardPod) -> Option<DateTime<Utc>> {
        let s = pod.to_string();
        let now = Utc::now();
        let latest_relevant = now - self.duration;

        if let Some(timestamps) = self.db.lock().await.get_mut(&s) {
            timestamps.retain_mut(|x| *x >= latest_relevant);

            if timestamps.len() > 0 {
                let latest = timestamps.iter().max();
                if let Some(latest) = latest {
                    let earliest_next_request = *latest + self.soft_duration;
                    if earliest_next_request > now {
                        return Some(earliest_next_request);
                    }
                }
            }

            if timestamps.len() > self.max_restarts as usize {
                let earliest = timestamps.iter().min();

                if let Some(earliest) = earliest {
                    return Some(*earliest + self.duration);
                }
            }
        }

        self.add(pod).await;
        None
    }
}
