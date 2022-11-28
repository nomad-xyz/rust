use crate::{errors::Error, Result};
use rusoto_core::{credential::ProfileProvider, Client, HttpClient, Region};
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use serde_yaml;
use std::{collections::HashMap, default::Default, env, fs, str::FromStr};
use tokio::io::AsyncReadExt;

/// A model for our remote secrets file
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Secrets {
    /// Equivalent to `CONFIG_URL`
    pub config_url: String,
    /// Equivalent to `CONFIG_PATH`. Included for testing only
    pub config_path: Option<String>,
    /// Equivalent to the set of `<NETWORK>_CONNECTION_URL`
    pub connection_urls: HashMap<String, String>,
    /// Equivalent to the set of `<NETWORK>_TXSIGNER_ID`
    pub txsigner_ids: HashMap<String, String>,
    /// Equivalent to the set of `<NETWORK>_ATTESTATION_SIGNER_ID`
    pub attestation_signer_ids: HashMap<String, String>,
}

impl Secrets {
    /// Create a `Secrets` by fetching yaml from an S3 bucket
    pub(crate) async fn fetch(
        profile: &str,
        region: &str,
        bucket: &str,
        key: &str,
    ) -> Result<Self> {
        let credentials_provider =
            ProfileProvider::with_default_credentials(profile).map_err(Error::CredentialsError)?;
        let client = Client::new_with(credentials_provider, HttpClient::new().unwrap());
        let s3_client = S3Client::new_with_client(client, Region::from_str(region).unwrap());
        Self::fetch_with_client(s3_client, bucket, key).await
    }

    /// Create a `Secrets` by fetching yaml from an S3 bucket given an `S3Client`
    pub(crate) async fn fetch_with_client(
        client: S3Client,
        bucket: &str,
        key: &str,
    ) -> Result<Self> {
        let mut yaml = String::new();
        let mut request = GetObjectRequest::default();
        request.bucket = bucket.into();
        request.key = key.into();
        let response = client
            .get_object(request)
            .await
            .map_err(Error::RusotoGetObject)?;
        response
            .body
            .unwrap()
            .into_async_read()
            .read_to_string(&mut yaml)
            .await
            .map_err(Error::BadIO)?;
        Ok(serde_yaml::from_slice::<Self>(yaml.as_bytes()).map_err(Error::YamlBadDeser)?)
    }

    /// Set `Secrets` as environment variables so they can be picked up by `Settings`
    pub(crate) fn set_environment(&self) {
        // We've included `CONFIG_PATH` for testing and `CONFIG_URL` takes precedence
        // so force precedence here.
        if let Some(ref path) = self.config_path {
            env::set_var("CONFIG_PATH", path);
        } else {
            env::set_var("CONFIG_URL", &self.config_url);
        }
        // Set everything else
        for (k, v) in self.connection_urls.iter() {
            env::set_var(k, v);
        }
        for (k, v) in self.txsigner_ids.iter() {
            env::set_var(k, v);
        }
        for (k, v) in self.attestation_signer_ids.iter() {
            env::set_var(k, v);
        }
        // Set constant values that don't need to be in the secrets file
        env::set_var("DEFAULT_RPCSTYLE", "ethereum");
        env::set_var("DEFAULT_SUBMITTER_TYPE", "local");
    }

    /// Create a `Secrets` by loading a local file. Included for testing only
    pub(crate) async fn load(path: &str) -> Result<Self> {
        let secrets = fs::read_to_string(path).unwrap();
        Ok(serde_yaml::from_slice::<Self>(secrets.as_bytes()).map_err(Error::YamlBadDeser)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rusoto_mock::{
        MockCredentialsProvider, MockRequestDispatcher, MultipleMockRequestDispatcher,
    };
    use std::fs;

    fn mock_s3_client() -> S3Client {
        let secrets_response =
            fs::read_to_string("../../fixtures/killswitch_secrets.testing.yaml").unwrap();
        let request_dispatcher = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::default().with_body(&secrets_response),
        ]);
        S3Client::new_with(
            request_dispatcher,
            MockCredentialsProvider,
            Region::default(),
        )
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_fetches_secrets_from_s3() {
        let s3_client = mock_s3_client();
        let secrets = Secrets::fetch_with_client(s3_client, "any-bucket", "any-key.yaml").await;
        assert!(secrets.is_ok());
    }
}
