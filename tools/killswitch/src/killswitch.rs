#![allow(dead_code)] // TODO: Remove me

use crate::{errors::Error, settings::Settings, Args, Result};
use nomad_core::SignedFailureNotification;
use std::collections::{HashMap, HashSet};

/// Main `KillSwitch` struct
#[derive(Debug)]
pub(crate) struct KillSwitch {
    /// Set of replicas by network we're disconnecting
    /// or errors encountered attempting to configure replicas
    replicas: HashMap<String, Result<HashSet<String>>>,
}

impl KillSwitch {
    /// Get replicas for network, returning errors for a missing network or an empty replica set
    fn get_replicas(network: &String, settings: &Settings) -> Result<HashSet<String>> {
        let core_contracts = settings.config.core().get(network);
        if core_contracts.is_none() {
            return Err(Error::MissingNetwork(format!(
                "Network {} was not found in core",
                network
            )));
        }
        let core_contracts = core_contracts.unwrap();
        let replicas = core_contracts
            .replicas()
            .map(Clone::clone)
            .collect::<HashSet<String>>();
        if replicas.is_empty() {
            return Err(Error::MissingReplicas(format!(
                "No replicas found for {} in core",
                network
            )));
        }
        Ok(replicas)
    }

    /// Create a `SignedFailureNotification`
    async fn create_signed_failure(&self) -> Result<SignedFailureNotification> {
        unimplemented!()
    }

    /// Build a new `KillSwitch`, configuring best effort and storing, not returning errors
    pub(crate) async fn new(args: Args, settings: Settings) -> Self {
        let networks = if args.all {
            Vec::from_iter(settings.config.networks.clone())
        } else {
            vec![args
                .all_inbound
                .expect("Should not happen. Clap requires this to be present")]
        };

        let replicas = networks
            .iter()
            .map(|network| (network.clone(), Self::get_replicas(network, &settings)))
            .collect::<HashMap<String, Result<HashSet<String>>>>();

        // Set up connection managers

        Self { replicas }
    }

    /// Run `KillSwitch` against configuration
    pub(crate) async fn run(&self) {
        unimplemented!()
    }
}
