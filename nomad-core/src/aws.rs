use rusoto_core::{
    credential::{AutoRefreshingProvider, ProvideAwsCredentials},
    Client, HttpClient,
};
use rusoto_kms::KmsClient;
use rusoto_sts::WebIdentityProvider;
use tokio::sync::OnceCell;

static CLIENT: OnceCell<Client> = OnceCell::const_new();
static KMS_CLIENT: OnceCell<KmsClient> = OnceCell::const_new();

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
    CLIENT
        .get_or_init(|| async {
            match try_irsa_provider().await {
                Some(credentials_provider) => {
                    let dispatcher = HttpClient::new().unwrap();
                    Client::new_with(credentials_provider, dispatcher)
                }
                // if the IRSA provider returned no creds, use the default
                // credentials chain
                None => Client::shared(),
            }
        })
        .await
}

/// Get a shared KMS client
pub async fn get_kms_client() -> &'static KmsClient {
    KMS_CLIENT
        .get_or_init(|| async {
            let client = get_client().await.clone();

            KmsClient::new_with_client(client, Default::default())
        })
        .await
}
