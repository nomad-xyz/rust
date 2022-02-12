use std::collections::{HashMap, HashSet};

use crate::{
    agent::AgentConfig,
    common::{NomadIdentifier, NumberOrNumberString},
    contracts::CoreContracts,
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Governor {
    pub address: NomadIdentifier,
    pub domain: NumberOrNumberString,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Governance {
    pub recovery_manager: NomadIdentifier,
    pub recovery_timelock: NumberOrNumberString,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreNetwork {
    pub name: String,
    pub domain: NumberOrNumberString,
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
