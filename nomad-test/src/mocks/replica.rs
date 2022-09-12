#![allow(non_snake_case)]

use async_trait::async_trait;
use mockall::*;

use ethers::core::types::H256;

use nomad_core::{accumulator::NomadProof, *};

use super::MockError;

mock! {
    pub ReplicaContract {
        // Replica
        pub fn _local_domain(&self) -> u32 {}

        pub fn _remote_domain(&self) -> Result<u32, MockError> {}

        pub fn _prove(&self, proof: &NomadProof) -> Result<TxOutcome, MockError> {}

        pub fn _process(&self, message: &NomadMessage) -> Result<TxOutcome, MockError> {}

        pub fn _prove_and_process(
            &self,
            message: &NomadMessage,
            proof: &NomadProof,
        ) -> Result<TxOutcome, MockError> {}

        // Common
        pub fn _name(&self) -> &str {}

        pub fn _status(&self, txid: H256) -> Result<Option<TxOutcome>, MockError> {}

        pub fn _updater(&self) -> Result<H256, MockError> {}

        pub fn _state(&self) -> Result<State, MockError> {}

        pub fn _committed_root(&self) -> Result<H256, MockError> {}
        pub fn _update(&self, update: &SignedUpdate) -> Result<TxOutcome, MockError> {}

        pub fn _double_update(
            &self,
            double: &DoubleUpdate,
        ) -> Result<TxOutcome, MockError> {}

        pub fn _message_status(&self, leaf: H256) -> Result<MessageStatus, MockError> {}

        pub fn _acceptable_root(&self, root: H256) -> Result<bool, MockError> {}
    }
}

impl std::fmt::Debug for MockReplicaContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockReplicaContract")
    }
}

impl std::fmt::Display for MockReplicaContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockReplica")
    }
}

#[async_trait]
impl Replica for MockReplicaContract {
    fn local_domain(&self) -> u32 {
        self._local_domain()
    }

    async fn remote_domain(&self) -> Result<u32, <Self as Common>::Error> {
        self._remote_domain()
    }

    async fn prove(&self, proof: &NomadProof) -> Result<TxOutcome, <Self as Common>::Error> {
        self._prove(proof)
    }

    async fn process(&self, message: &NomadMessage) -> Result<TxOutcome, <Self as Common>::Error> {
        self._process(message)
    }

    async fn prove_and_process(
        &self,
        message: &NomadMessage,
        proof: &NomadProof,
    ) -> Result<TxOutcome, <Self as Common>::Error> {
        self._prove_and_process(message, proof)
    }

    async fn message_status(&self, leaf: H256) -> Result<MessageStatus, <Self as Common>::Error> {
        self._message_status(leaf)
    }

    async fn acceptable_root(&self, root: H256) -> Result<bool, <Self as Common>::Error> {
        self._acceptable_root(root)
    }
}

#[async_trait]
impl Common for MockReplicaContract {
    type Error = MockError;

    fn name(&self) -> &str {
        self._name()
    }

    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, Self::Error> {
        self._status(txid)
    }

    async fn updater(&self) -> Result<H256, Self::Error> {
        self._updater()
    }

    async fn state(&self) -> Result<State, Self::Error> {
        self._state()
    }

    async fn committed_root(&self) -> Result<H256, Self::Error> {
        self._committed_root()
    }

    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, Self::Error> {
        self._update(update)
    }

    async fn double_update(&self, double: &DoubleUpdate) -> Result<TxOutcome, Self::Error> {
        self._double_update(double)
    }
}
