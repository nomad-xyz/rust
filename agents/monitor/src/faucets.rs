use std::collections::HashMap;

use agent_utils::{pipe::Pipe, HomeReplicaMap, NetworkMap};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    DispatchFaucet, DispatchPipe, DispatchSink, ProcessFaucet, ProcessPipe, ProcessSink,
    RelayFaucet, RelayPipe, UpdateFaucet, UpdatePipe,
};

pub(crate) struct Faucets<'a> {
    pub(crate) dispatches: NetworkMap<'a, DispatchFaucet>,
    pub(crate) updates: NetworkMap<'a, UpdateFaucet>,
    pub(crate) relays: HomeReplicaMap<'a, RelayFaucet>,
    pub(crate) processes: HomeReplicaMap<'a, ProcessFaucet>,
}

impl<'a> Faucets<'a> {
    pub(crate) fn swap_dispatch(
        &mut self,
        network: &'a str,
        mut dispatch_faucet: DispatchFaucet,
    ) -> DispatchFaucet {
        self.dispatches
            .get_mut(network)
            .map(|old| {
                std::mem::swap(old, &mut dispatch_faucet);
                dispatch_faucet
            })
            .expect("missing dispatch faucet")
    }

    pub(crate) fn swap_update(
        &mut self,
        network: &'a str,
        mut update_faucet: UpdateFaucet,
    ) -> UpdateFaucet {
        self.updates
            .get_mut(network)
            .map(|old| {
                std::mem::swap(old, &mut update_faucet);
                update_faucet
            })
            .expect("missing dispatch faucet")
    }

    pub(crate) fn swap_relay(
        &mut self,
        network: &'a str,
        replica_of: &'a str,
        mut relay_faucet: RelayFaucet,
    ) -> RelayFaucet {
        self.relays
            .get_mut(network)
            .expect("missing network")
            .get_mut(replica_of)
            .map(|old| {
                std::mem::swap(old, &mut relay_faucet);
                relay_faucet
            })
            .expect("missing faucet")
    }

    pub(crate) fn swap_process(
        &mut self,
        network: &'a str,
        replica_of: &'a str,
        mut process_faucet: ProcessFaucet,
    ) -> ProcessFaucet {
        self.processes
            .get_mut(network)
            .expect("missing network")
            .get_mut(replica_of)
            .map(|old| {
                std::mem::swap(old, &mut process_faucet);
                process_faucet
            })
            .expect("missing faucet")
    }

    pub(crate) fn dispatch_pipe(&mut self, network: &'a str) -> DispatchPipe {
        let (tx, rx) = unbounded_channel();
        let rx = self.swap_dispatch(network, rx);
        Pipe::new(rx, tx, None)
    }

    pub(crate) fn update_pipe(&mut self, network: &'a str) -> UpdatePipe {
        let (tx, rx) = unbounded_channel();
        let rx = self.swap_update(network, rx);
        Pipe::new(rx, tx, None)
    }

    pub(crate) fn relay_pipe(&mut self, network: &'a str, replica_of: &'a str) -> RelayPipe {
        let (tx, rx) = unbounded_channel();
        let rx = self.swap_relay(network, replica_of, rx);
        Pipe::new(rx, tx, None)
    }

    pub(crate) fn process_pipe(&mut self, network: &'a str, replica_of: &'a str) -> ProcessPipe {
        let (tx, rx) = unbounded_channel();
        let rx = self.swap_process(network, replica_of, rx);
        Pipe::new(rx, tx, None)
    }

    // used for e2e
    pub(crate) fn swap_all_dispatches(
        &mut self,
    ) -> (
        HashMap<String, DispatchSink>,
        HashMap<String, DispatchFaucet>,
    ) {
        let mut sinks = HashMap::new();

        let mut dispatches = HashMap::new();

        self.dispatches.keys().for_each(|key| {
            let (sink, faucet) = unbounded_channel();
            sinks.insert(key.to_string(), sink);
            dispatches.insert(*key, faucet);
        });

        std::mem::swap(&mut self.dispatches, &mut dispatches);

        let faucets = dispatches
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        (sinks, faucets)
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn swap_all_processes(
        &mut self,
    ) -> (
        HashMap<String, HashMap<String, ProcessSink>>,
        HashMap<String, HashMap<String, ProcessFaucet>>,
    ) {
        let mut sinks: HashMap<String, HashMap<String, ProcessSink>> = HashMap::new();
        let mut processes: HomeReplicaMap<ProcessFaucet> = HashMap::new();

        self.processes.iter().for_each(|(network, map)| {
            map.keys().for_each(|replica_of| {
                let (sink, faucet) = unbounded_channel();

                processes
                    .entry(network)
                    .or_default()
                    .insert(replica_of, faucet);

                sinks
                    .entry(network.to_string())
                    .or_default()
                    .insert(replica_of.to_string(), sink);
            });
        });

        std::mem::swap(&mut self.processes, &mut processes);

        let faucets = processes
            .into_iter()
            .map(|(network, map)| {
                let map = map
                    .into_iter()
                    .map(|(replica_of, faucet)| (replica_of.to_string(), faucet))
                    .collect();
                (network.to_string(), map)
            })
            .collect();
        (sinks, faucets)
    }
}
