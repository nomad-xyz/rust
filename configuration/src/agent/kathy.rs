//! Kathy public configuration

use crate::decl_config;
use ethers::types::H256;

decl_config!(Kathy {
    /// Chat generator config
    #[serde(default)]
    chat: ChatGenConfig,
});

/// Kathy chat generator configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatGenConfig {
    /// Static messages
    Static {
        /// Recipient
        recipient: H256,
        /// Message
        message: String,
    },
    /// Ordered list of messages
    OrderedList {
        /// Messages
        messages: Vec<String>,
    },
    /// Random messages
    Random {
        /// Message length
        length: usize,
    },
    /// Default
    #[serde(other)]
    Default,
}

impl Default for ChatGenConfig {
    fn default() -> Self {
        Self::Default
    }
}
