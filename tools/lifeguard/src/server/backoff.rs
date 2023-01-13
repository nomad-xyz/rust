use std::{collections::HashMap, ops::Sub};

use crate::k8s::LifeguardPod;
use chrono::prelude::*;
use chrono::Duration;

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
    timespan: u32,
    max_restarts: u32,
    db: HashMap<String, Vec<DateTime<Utc>>>,
}

impl RestartBackoff {
    pub fn new(max_restarts: u32) -> Result<Self, Box<dyn std::error::Error>> {
        if max_restarts > 0 {
            Ok(Self {
                timespan: 5000,
                max_restarts,
                db: HashMap::new(),
            })
        } else {
            Err(Box::new(&BackoffError::ZeroMaxRestarts))
        }
    }

    fn clean_and_calc(&mut self, pod: &LifeguardPod) -> u32 {
        let s = pod.to_string();
        let now = Utc::now();
        let latest_relevant = now - Duration::seconds(self.timespan.into());

        if let Some(timestamps) = self.db.get_mut(&s) {
            timestamps.retain_mut(|x| *x >= latest_relevant);
            return timestamps.len() as u32;
        }

        return 0;
    }

    fn add(&mut self, pod: &LifeguardPod) {
        let s = pod.to_string();
        let now = Utc::now();
        if let Some(vecc) = self.db.get_mut(&s) {
            vecc.push(now);
        } else {
            self.db.insert(s, vec![now]);
        }
    }

    pub fn inc(&mut self, pod: &LifeguardPod) -> bool {
        // let now: DateTime<Utc> = Utc::now();
        let relevant = self.clean_and_calc(pod);
        if relevant >= self.max_restarts {
            return false;
        } else {
            self.add(pod);
            return true;
        }

        // let s = pod.to_string();
        // if let Some(x) = self.db.get_mut(&s) {
        //     if *x < self.max_restarts {
        //         *x += 1;
        //         return true;
        //     } else {
        //         return false;
        //     }
        // } else {
        //     self.db.insert(s, 1);

        //     return true;
        // }
    }
}
