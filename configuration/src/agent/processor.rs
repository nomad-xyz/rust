//! Processor public configuration

use crate::{decl_config, decl_env_overrides, S3Config};
use ethers::types::H256;
use std::collections::HashSet;

decl_config!(Processor {
    /// Allow list
    allowed: Option<HashSet<H256>>,
    /// Deny list
    denied: Option<HashSet<H256>>,
    /// Remote chains to subsidize processing on
    subsidized_remotes: HashSet<String>,
    /// Whether to upload proofs to s3
    #[serde(default, skip_serializing_if = "Option::is_none")]
    s3: Option<S3Config>,
});

decl_env_overrides!(Processor {self, {
    if let Ok(var) = std::env::var("PROCESSOR_ALLOWED") {
        let allowed = var
            .split(",")
            .map(|v| v.parse::<H256>().expect("invalid PROCESSOR_ALLOWED"))
            .collect::<HashSet<H256>>();
        if allowed.len() < 1 {
            panic!("invalid PROCESSOR_ALLOWED");
        }
        self.allowed = Some(allowed)
    }
    if let Ok(var) = std::env::var("PROCESSOR_DENIED") {
        let denied = var
            .split(",")
            .map(|v| v.parse::<H256>().expect("invalid PROCESSOR_DENIED"))
            .collect::<HashSet<H256>>();
        if denied.len() < 1 {
            panic!("invalid PROCESSOR_DENIED");
        }
        self.denied = Some(denied)
    }
    if let Ok(var) = std::env::var("PROCESSOR_SUBSIDIZED_REMOTES") {
        let subsidized_remotes = var
            .split(",")
            .map(String::from)
            .collect::<HashSet<String>>();
        if subsidized_remotes.len() < 1 {
            panic!("invalid PROCESSOR_SUBSIDIZED_REMOTES");
        }
        self.subsidized_remotes = subsidized_remotes
    }
    if let (Ok(bucket), Ok(region)) = (
        std::env::var("PROCESSOR_S3_BUCKET"),
        std::env::var("PROCESSOR_S3_REGION"),
    ) {
        self.s3 = Some(S3Config { bucket, region })
    }
}});

#[cfg(test)]
mod test {
    use super::*;
    use nomad_test::test_utils;
    use std::{env, str::FromStr};

    #[test]
    #[serial_test::serial]
    fn it_overrides_config_from_env() {
        test_utils::run_test_with_env_sync("../fixtures/env.test-agents", move || {
            let mut config = ProcessorConfig::default();
            config.load_env_overrides();

            let hashes = HashSet::from([
                H256::from_str(
                    "0x1111111111111111111111111111111111111111111111111111111111111111",
                )
                .unwrap(),
                H256::from_str(
                    "0x1111111111111111111111111111111111111111111111111111111111111112",
                )
                .unwrap(),
                H256::from_str(
                    "0x1111111111111111111111111111111111111111111111111111111111111113",
                )
                .unwrap(),
            ]);
            assert_eq!(config.allowed, Some(hashes.clone()));
            assert_eq!(config.denied, Some(hashes));
            assert_eq!(
                config.subsidized_remotes,
                HashSet::from([
                    "chain1".to_string(),
                    "chain3".to_string(),
                    "chain2".to_string(),
                ])
            );
            assert_eq!(
                config.s3,
                Some(S3Config {
                    bucket: "aws-bucket".to_string(),
                    region: "region-1".to_string(),
                })
            );
            assert_eq!(config.interval, 999);
            assert_eq!(config.enabled, true);
        });
    }
}
