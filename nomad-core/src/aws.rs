use once_cell::sync::OnceCell;
use rusoto_core::{
    credential::{AutoRefreshingProvider, ProvideAwsCredentials},
    Client, HttpClient,
};
use rusoto_kms::KmsClient;
use rusoto_sts::WebIdentityProvider;

static CLIENT: OnceCell<Client> = OnceCell::new();
static KMS_CLIENT: OnceCell<KmsClient> = OnceCell::new();

// Try to get an irsa provider
#[tracing::instrument]
async fn try_irsa_provider() -> Option<AutoRefreshingProvider<WebIdentityProvider>> {
    let irsa_provider = WebIdentityProvider::from_k8s_env();

    // if there are no IRSA credentials this will error
    let result = irsa_provider.credentials().await;

    if result.is_err() {
        tracing::debug!(error = %result.as_ref().unwrap_err(), "Error in irsa provider instantiation");
    }

    result
        .ok()
        .and_then(|_| AutoRefreshingProvider::new(irsa_provider).ok())
}

/// Get a shared AWS client with credentials
///
/// Credential precedence is as follows
/// 1. IRSA
/// 2. IAM
/// 3. environment
/// 4. Conf file
pub async fn get_client() -> &'static Client {
    // init exactly once
    if CLIENT.get().is_none() {
        // try IRSA first
        let client = match try_irsa_provider().await {
            Some(credentials_provider) => {
                let dispatcher = HttpClient::new().unwrap();
                Client::new_with(credentials_provider, dispatcher)
            }
            // if the IRSA provider returned no creds, use the default
            // credentials chain
            None => Client::shared(),
        };

        if CLIENT.set(client).is_err() {
            panic!("unable to set Client")
        };
    }

    CLIENT.get().expect("just init")
}

/// Get a shared KMS client
pub async fn get_kms_client() -> &'static KmsClient {
    if KMS_CLIENT.get().is_none() {
        let client = get_client().await.clone();

        let kms = KmsClient::new_with_client(client, Default::default());

        if KMS_CLIENT.set(kms).is_err() {
            panic!("unable to set KmsClient")
        };
    }
    KMS_CLIENT.get().expect("just init")
}
