use ethers_signers::Signer;
use nomad_core::Signers;
use nomad_xyz_configuration::{agent::SignerConf, FromEnv};

#[tokio::test]
async fn signer_auths() {
    if let Some(signer_conf) = SignerConf::from_env("TEST_KMS", None) {
        let signer = Signers::try_from_signer_conf(&signer_conf).await.unwrap();
        let message = "hello world";
        signer.sign_message(message).await.unwrap();
    }
}
