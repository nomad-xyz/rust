#![allow(dead_code)] // TODO: Remove me

use crate::{errors::Error, settings::Settings, Args, Result};
use nomad_core::SignedFailureNotification;
use std::collections::{HashMap, HashSet};

/// Main `KillSwitch` struct
#[derive(Debug)]
pub(crate) struct KillSwitch {
    /// List of homes we're disconnecting replicas for
    homes: Vec<String>,
    /// Set of replicas we're disconnection or errors
    /// encountered attempting to configure replicas
    replicas: HashMap<String, Result<HashSet<String>>>,
}

impl KillSwitch {
    /// Get replicas for home, returning errors for missing a home or an empty replica set
    fn get_replicas(home: &String, settings: &Settings) -> Result<HashSet<String>> {
        let connections = settings
            .config
            .protocol()
            .networks
            .get(home)
            .ok_or_else(|| {
                Error::MissingHome(format!("Home {} was not found in protocol.networks", home))
            })?
            .connections
            .clone();
        if connections.is_empty() {
            Err(Error::MissingReplicas(format!(
                "No replicas found for {} in protocol.networks.connections",
                home
            )))
        } else {
            Ok(connections)
        }
    }

    /// Create a `SignedFailureNotification`
    async fn create_signed_failure(&self) -> Result<SignedFailureNotification> {
        unimplemented!()
    }

    /// Build a new `KillSwitch`, configuring best effort and storing, not returning errors
    pub(crate) async fn new(args: Args, settings: Settings) -> Self {
        let homes = if args.all {
            settings
                .config
                .bridge()
                .keys()
                .map(Clone::clone)
                .collect::<Vec<String>>()
        } else {
            vec![args
                .all_inbound
                .expect("Should not happen. Clap requires this to be present")]
        };

        let replicas = homes
            .iter()
            .map(|home| (home.clone(), Self::get_replicas(home, &settings)))
            .collect::<HashMap<String, Result<HashSet<String>>>>();

        let _chain_setups = replicas
            .iter()
            .filter_map(|(home, replicas)| {
                if let Ok(replicas) = replicas {
                    Some((home.clone(), replicas.clone()))
                } else {
                    None
                }
            })
            .map(|(_home, _replicas)| unimplemented!());

        // ...

        unimplemented!()
    }

    /// Run `KillSwitch` against configuration
    pub(crate) async fn run(&self) {
        unimplemented!()
    }
}
