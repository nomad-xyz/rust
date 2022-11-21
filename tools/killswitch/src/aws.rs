use crate::{errors::Error, Result};
use rusoto_core::credential::{ProfileProvider, ProvideAwsCredentials};

/// An AWS credentials helper that checks for locally-stored
/// credentials and validates them (as existing)
pub(crate) struct MasterCredentials {
    /// Internal `ProfileProvider` we use to handle local credentials
    provider: ProfileProvider,
}

impl MasterCredentials {
    /// New `MasterCredentials` with locally stored credentials
    pub(crate) fn new_from_credentials() -> Result<Self> {
        Ok(MasterCredentials {
            provider: ProfileProvider::new().map_err(Error::CredentialsError)?,
        })
    }

    /// Ensure that the underlying provider can return valid credentials
    pub(crate) async fn validate_credentials(&self) -> Result<()> {
        let _ = self
            .provider
            .credentials()
            .await
            .map_err(Error::CredentialsError)?;
        Ok(())
    }
}
