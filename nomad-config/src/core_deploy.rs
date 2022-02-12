use std::collections::{HashMap, HashSet};

use crate::{
    agent::AgentConfig,
    common::{NomadIdentifier, NumberOrDecimalString},
    contracts::CoreContracts,
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Governor {
    pub address: NomadIdentifier,
    pub domain: NumberOrDecimalString,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Governance {
    pub recovery_manager: NomadIdentifier,
    pub recovery_timelock: NumberOrDecimalString,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreNetwork {
    pub name: String,
    pub domain: NumberOrDecimalString,
    pub connections: HashSet<String>,
    pub contracts: CoreContracts,
    pub governance: Governance,
    pub updaters: HashSet<NomadIdentifier>,
    pub watchers: HashSet<NomadIdentifier>,
    pub agents: AgentConfig,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDeploy {
    pub governor: Governor,
    pub networks: HashMap<String, CoreNetwork>,
}
