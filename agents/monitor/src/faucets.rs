use tokio::sync::mpsc::unbounded_channel;

use crate::{
    pipe::{DispatchPipe, Pipe, ProcessPipe, RelayPipe, UpdatePipe},
    DispatchFaucet, HomeReplicaMap, NetworkMap, ProcessFaucet, RelayFaucet, UpdateFaucet,
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
}
