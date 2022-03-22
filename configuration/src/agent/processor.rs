//! Processor public configuration

use crate::decl_config;
use ethers::types::H256;
use std::collections::HashSet;

decl_config!(Processor {
    /// Allow list
    allowed: Option<HashSet<H256>>,
    /// Deny list
    denied: Option<HashSet<H256>>,
    /// Index only mode
    index_only: bool,
    /// S3 config
    s3: Option<S3Config>,
});

/// S3 Configuration
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    /// Bucket
    pub bucket: String,
    /// Region
    pub region: String,
}
