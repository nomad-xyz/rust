use once_cell::sync::OnceCell;
use rusoto_core::{
    credential::{AutoRefreshingProvider, ProvideAwsCredentials},
    Client,
};
use rusoto_kms::KmsClient;

static CLIENT: OnceCell<Client> = OnceCell::new();
static KMS_CLIENT: OnceCell<KmsClient> = OnceCell::new();

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
        let irsa_provider = rusoto_sts::WebIdentityProvider::from_k8s_env();

        // if there are no IRSA credentials this will error
        let creds_res = irsa_provider.credentials().await;

        // if the irsa provider returned creds ok, we'll use the IRSA provider
        let client = if creds_res.is_ok() {
            Client::new_with(
                AutoRefreshingProvider::new(irsa_provider).unwrap(),
                rusoto_core::HttpClient::new().unwrap(),
            )
        } else {
            // if the IRSA provider returned no creds, use the default credentials
            // chain
            Client::shared()
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
        let _ = get_client().await;

        let kms = KmsClient::new_with_client(
            CLIENT.get().expect("just init").clone(),
            Default::default(),
        );
        if KMS_CLIENT.set(kms).is_err() {
            panic!("unable to set KmsClient")
        };
    }
    KMS_CLIENT.get().expect("just init")
}
