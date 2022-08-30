//! Nomad-bridge related configuration structs

use std::collections::HashSet;

use nomad_types::deser_nomad_u32;
use nomad_types::{NomadIdentifier, NomadLocator, Proxy};

use crate::network::CustomTokenSpecifier;

/// Deploy-time custom tokens
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct DeployedCustomToken {
    /// Token domain and ID
    pub token: NomadLocator,
    /// Token name
    pub name: String,
    /// Token Symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u8,
    /// Address of the UBC
    pub controller: NomadIdentifier,
    /// Deployed token information
    pub addresses: Proxy,
}

/// EVM Bridge Contracts
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumBridgeDeploymentInfo {
    /// Contract Deploy Height
    #[serde(default, deserialize_with = "deser_nomad_u32")]
    pub deploy_height: u32,
    /// Bridge Route proxy
    pub bridge_router: Proxy,
    /// Token Registry proxy
    pub token_registry: Proxy,
    /// Bridge Token proxy
    pub bridge_token: Proxy,
    /// Eth Helper address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eth_helper: Option<NomadIdentifier>,
    /// Custom Tokens (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customs: Option<HashSet<DeployedCustomToken>>,
}

/// Empty Substrate contracts
#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubstrateBridgeDeployInfo {
    /// Contract Deploy Height
    #[serde(default, deserialize_with = "deser_nomad_u32")]
    pub deploy_height: u32,
}

/// Bridge contract abstraction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum BridgeDeploymentInfo {
    /// EVM Bridge Contracts
    Ethereum(EthereumBridgeDeploymentInfo),
    /// Substrate bridge
    Substrate(SubstrateBridgeDeployInfo),
}

impl Default for BridgeDeploymentInfo {
    fn default() -> Self {
        BridgeDeploymentInfo::Ethereum(Default::default())
    }
}

const fn is_false(b: &bool) -> bool {
    !*b
}

/// EVM Bridge Contracts
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    display_name: String,
    native_token_symbol: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    connections: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    manual_processing: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    connext_enabled: bool,
}

/// Configuration for bridge contracts
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeConfiguration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Location of WETH if any
    pub weth: Option<NomadIdentifier>,
    /// Custom token deployment specifiers
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customs: Option<HashSet<CustomTokenSpecifier>>,
}
