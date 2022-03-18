use ethers::types::H256;
use std::collections::HashSet;

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessorConfig {
    pub interval: u64,
    pub allowed: Option<HashSet<H256>>,
    pub denied: Option<HashSet<H256>>,
    pub index_only: Option<String>,
    pub s3: Option<S3Config>,
}
