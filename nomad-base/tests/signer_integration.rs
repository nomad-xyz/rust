use nomad_base::{AttestationSigner, Signer};
use nomad_xyz_configuration::agent::SignerConf;

#[tokio::test]
async fn signer_auths() {
    if let Some(signer_conf) = SignerConf::from_env(Some("TEST_KMS"), None) {
        let signer = AttestationSigner::try_from_signer_conf(&signer_conf)
            .await
            .unwrap();
        let message = "hello world";
        signer.sign_message(message).await.unwrap();
    }
}
