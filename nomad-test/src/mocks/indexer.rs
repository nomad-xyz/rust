#![allow(non_snake_case)]

use crate::MockError;
use async_trait::async_trait;
use color_eyre::Result;
use mockall::*;

use nomad_core::*;

mock! {
    pub Indexer {
        pub fn _get_block_number(&self) -> Result<u32, MockError> {}

        pub fn _fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>, MockError> {}

        pub fn _fetch_sorted_messages(&self, from: u32, to: u32) -> Result<Vec<RawCommittedMessage>, MockError> {}
    }
}

impl std::fmt::Debug for MockIndexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockIndexer")
    }
}

#[async_trait]
impl CommonIndexer for MockIndexer {
    type Error = MockError;

    async fn get_block_number(&self) -> Result<u32, Self::Error> {
        self._get_block_number()
    }

    async fn fetch_sorted_updates(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<SignedUpdateWithMeta>, Self::Error> {
        self._fetch_sorted_updates(from, to)
    }
}

#[async_trait]
impl HomeIndexer for MockIndexer {
    async fn fetch_sorted_messages(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<RawCommittedMessage>, <Self as CommonIndexer>::Error> {
        self._fetch_sorted_messages(from, to)
    }
}
