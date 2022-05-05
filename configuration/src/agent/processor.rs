//! Processor public configuration

use crate::{decl_config, S3Config};
use ethers::types::H256;
use std::collections::HashSet;

decl_config!(Processor {
    /// Allow list
    allowed: Option<HashSet<H256>>,
    /// Deny list
    denied: Option<HashSet<H256>>,
    /// Index only mode
    subsidized_remotes: Vec<String>,
    /// Whether to upload proofs to s3
    #[serde(default, skip_serializing_if = "Option::is_none")]
    s3: Option<S3Config>,
});
