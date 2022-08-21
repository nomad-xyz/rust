#![allow(non_snake_case)]

use async_trait::async_trait;
use mockall::*;

use nomad_core::*;

use super::MockError;

mock! {
    pub ConnectionManagerContract {
        pub fn _local_domain(&self) -> u32 {}

        pub fn _is_replica(&self, address: NomadIdentifier) -> Result<bool, MockError> {}

        pub fn _watcher_permission(
            &self,
            address: NomadIdentifier,
            domain: u32,
        ) -> Result<bool, MockError> {}

        pub fn _owner_enroll_replica(
            &self,
            replica: NomadIdentifier,
            domain: u32,
        ) -> Result<TxOutcome, MockError> {}

        pub fn _owner_unenroll_replica(
            &self,
            replica: NomadIdentifier,
        ) -> Result<TxOutcome, MockError> {}

        pub fn _set_home(&self, home: NomadIdentifier) -> Result<TxOutcome, MockError> {}

        pub fn _set_watcher_permission(
            &self,
            watcher: NomadIdentifier,
            domain: u32,
            access: bool,
        ) -> Result<TxOutcome, MockError> {}

        pub fn _unenroll_replica(
            &self,
            signed_failure: &SignedFailureNotification,
        ) -> Result<TxOutcome, MockError> {}
    }
}

impl std::fmt::Debug for MockConnectionManagerContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockConnectionManagerContract")
    }
}

#[async_trait]
impl ConnectionManager for MockConnectionManagerContract {
    type Error = MockError;

    fn local_domain(&self) -> u32 {
        self._local_domain()
    }

    async fn is_replica(&self, address: NomadIdentifier) -> Result<bool, Self::Error> {
        self._is_replica(address)
    }

    async fn watcher_permission(
        &self,
        address: NomadIdentifier,
        domain: u32,
    ) -> Result<bool, Self::Error> {
        self._watcher_permission(address, domain)
    }

    async fn owner_enroll_replica(
        &self,
        replica: NomadIdentifier,
        domain: u32,
    ) -> Result<TxOutcome, Self::Error> {
        self._owner_enroll_replica(replica, domain)
    }

    async fn owner_unenroll_replica(
        &self,
        replica: NomadIdentifier,
    ) -> Result<TxOutcome, Self::Error> {
        self._owner_unenroll_replica(replica)
    }

    async fn set_home(&self, home: NomadIdentifier) -> Result<TxOutcome, Self::Error> {
        self._set_home(home)
    }

    async fn set_watcher_permission(
        &self,
        watcher: NomadIdentifier,
        domain: u32,
        access: bool,
    ) -> Result<TxOutcome, Self::Error> {
        self._set_watcher_permission(watcher, domain, access)
    }

    async fn unenroll_replica(
        &self,
        signed_failure: &SignedFailureNotification,
    ) -> Result<TxOutcome, Self::Error> {
        self._unenroll_replica(signed_failure)
    }
}
