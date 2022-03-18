//! Configuration

use ethers::core::types::H256;

use crate::kathy::ChatGenerator;

use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatGenConfig {
    Static {
        recipient: H256,
        message: String,
    },
    OrderedList {
        messages: Vec<String>,
    },
    Random {
        length: usize,
    },
    #[serde(other)]
    Default,
}

impl Default for ChatGenConfig {
    fn default() -> Self {
        Self::Default
    }
}

impl From<ChatGenConfig> for ChatGenerator {
    fn from(conf: ChatGenConfig) -> ChatGenerator {
        match conf {
            ChatGenConfig::Static { recipient, message } => {
                ChatGenerator::Static { recipient, message }
            }
            ChatGenConfig::OrderedList { messages } => ChatGenerator::OrderedList {
                messages,
                counter: 0,
            },
            ChatGenConfig::Random { length } => ChatGenerator::Random { length },
            ChatGenConfig::Default => ChatGenerator::Default,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KathySettingsBlock {
    pub interval: u64,
    pub chat: ChatGenConfig,
}

// TODO: add kathy settings to configuration
impl AgentSettingsBlock for KathySettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        _secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().kathy.interval;
        Self {
            interval,
            chat: Default::default(),
        }
    }
}
decl_settings!(Kathy, KathySettingsBlock,);
