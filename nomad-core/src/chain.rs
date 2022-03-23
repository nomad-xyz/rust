#![allow(missing_docs)]

use color_eyre::eyre::Result;
use nomad_types::NomadIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Address(pub bytes::Bytes);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balance(pub num::BigInt);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLocator {
    pub name: String,
    pub domain: u32,
    pub address: NomadIdentifier,
}
impl std::fmt::Display for ContractLocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}[@{}]+contract:0x{:x}",
            self.name, self.domain, *self.address
        )
    }
}

#[async_trait::async_trait]
pub trait Chain {
    /// Query the balance on a chain
    async fn query_balance(&self, addr: Address) -> Result<Balance>;
}
