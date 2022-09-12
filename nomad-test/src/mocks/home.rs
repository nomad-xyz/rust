#![allow(non_snake_case)]

use super::MockError;
use async_trait::async_trait;
use mockall::*;

use ethers::core::types::{H256, U256};

use nomad_core::*;

mock! {
    pub HomeContract {
        // Home
        pub fn _local_domain(&self) -> u32 {}

        pub fn _home_domain_hash(&self) -> H256 {}

        pub fn _raw_message_by_nonce(
            &self,
            destination: u32,
            nonce: u32,
        ) -> Result<Option<RawCommittedMessage>, MockError> {}

        pub fn _raw_message_by_leaf(
            &self,
            leaf: H256,
        ) -> Result<Option<RawCommittedMessage>, MockError> {}


        pub fn _leaf_by_tree_index(
            &self,
            tree_index: usize,
        ) -> Result<Option<H256>, MockError> {}

        pub fn _nonces(&self, destination: u32) -> Result<u32, MockError> {}

        pub fn _dispatch(&self, message: &Message) -> Result<TxOutcome, MockError> {}

        pub fn _queue_length(&self) -> Result<U256, MockError> {}

        pub fn _queue_contains(&self, root: H256) -> Result<bool, MockError> {}

        pub fn _improper_update(
            &self,
            update: &SignedUpdate,
        ) -> Result<TxOutcome, MockError> {}

        pub fn _produce_update(&self) -> Result<Option<Update>, MockError> {}

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
    }
}

impl std::fmt::Debug for MockHomeContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockHomeContract")
    }
}

impl std::fmt::Display for MockHomeContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockHome")
    }
}

#[async_trait]
impl Home for MockHomeContract {
    fn local_domain(&self) -> u32 {
        self._local_domain()
    }

    fn home_domain_hash(&self) -> H256 {
        self._home_domain_hash()
    }

    async fn nonces(&self, destination: u32) -> Result<u32, <Self as Common>::Error> {
        self._nonces(destination)
    }

    async fn dispatch(&self, message: &Message) -> Result<TxOutcome, <Self as Common>::Error> {
        self._dispatch(message)
    }

    async fn queue_length(&self) -> Result<U256, <Self as Common>::Error> {
        self._queue_length()
    }

    async fn queue_contains(&self, root: H256) -> Result<bool, <Self as Common>::Error> {
        self._queue_contains(root)
    }

    async fn improper_update(
        &self,
        update: &SignedUpdate,
    ) -> Result<TxOutcome, <Self as Common>::Error> {
        self._improper_update(update)
    }

    async fn produce_update(&self) -> Result<Option<Update>, <Self as Common>::Error> {
        self._produce_update()
    }
}

#[async_trait]
impl Common for MockHomeContract {
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
