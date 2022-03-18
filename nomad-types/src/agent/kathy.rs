use ethers::types::H256;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KathyConfig {
    pub interval: u64,
    pub chat: ChatGenConfig,
}

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
