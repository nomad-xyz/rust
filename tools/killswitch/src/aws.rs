use crate::{errors::Error, Result};
use rusoto_core::credential::{ProfileProvider, ProvideAwsCredentials};

/// An AWS credentials helper that checks for locally-stored credentials
pub(crate) struct AWSCredentials;

impl AWSCredentials {
    /// Check for `AWSCredentials` stored locally
    pub(crate) async fn check_credentials() -> Result<()> {
        ProfileProvider::new()
            .map_err(Error::CredentialsError)?
            .credentials()
            .await
            .map_err(Error::CredentialsError)?;
        Ok(())
    }
}
