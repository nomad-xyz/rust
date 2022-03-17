//! Secrets configuration for agents.
//!
//! This struct is serialized from a JSON file or built drawing from a hosted
//! secrets manager backend. This struct is then used to finish building an
//! agents `Settings` block (see settings/mod.rs) along with a `NomadConfig`.
//!
//! Example JSON File Format
//! {
//!     "rpcs": {
//!         "ethereum": {
//!             "type": "http",
//!             "url": ""
//!         },
//!         "moonbeam": {
//!             "type": "http",
//!             "url": ""
//!         },
//!     },
//!     "transactionSigners": {
//!         "ethereum": {
//!             "key": "",
//!             "type": "hexKey"
//!         },
//!         "moonbeam": {
//!             "key": "",
//!             "type": "hexKey"
//!         },
//!     },
//!     "attestationSigner": {
//!         "key": "",
//!         "type": "hexKey"
//!     }
//! }

use crate::{ChainConf, SignerConf};
use serde::Deserialize;
use std::collections::HashMap;
use std::{fs::File, io::BufReader, path::PathBuf};

/// Agent secrets block
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentSecrets {
    /// RPC endpoints
    pub rpcs: HashMap<String, ChainConf>,
    /// Transaction signers
    pub transaction_signers: HashMap<String, SignerConf>,
    /// Attestation signers
    pub attestation_signer: SignerConf,
}

impl AgentSecrets {
    /// Get JSON file and deserialize into AgentSecrets
    pub fn from_json_file(path: PathBuf) -> Self {
        let file = File::open(path).expect("failed to open secrets file");
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).expect("failed to parse json into AgentSecrets")
    }
}
