use ethers_signers::Signer;
use nomad_core::Signers;
use nomad_xyz_configuration::agent::SignerConf;

#[tokio::test]
async fn signer_auths() {
    if let Some(signer_conf) = SignerConf::from_env(Some("TEST_KMS"), None) {
        let kms_client = nomad_core::aws::get_kms_client().await;
        let signer = Signers::try_from_signer_conf(&signer_conf, Some(kms_client))
            .await
            .unwrap();
        let message = "hello world";
        signer.sign_message(message).await.unwrap();
    }
}
