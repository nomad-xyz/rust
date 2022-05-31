//! Kathy public configuration

use crate::{decl_config, decl_env_overrides};
use ethers::types::H256;

decl_config!(Kathy {
    /// Chat generator config
    #[serde(default)]
    chat: ChatGenConfig,
});

decl_env_overrides!(Kathy {self, {
    if let (Ok(rec), Ok(msg)) = (
        std::env::var("KATHY_CHAT_RECIPIENT"),
        std::env::var("KATHY_CHAT_MESSAGE"),
    ) {
        self.chat = ChatGenConfig::Static {
            recipient: rec.parse::<H256>().expect("invalid KATHY_CHAT_RECIPIENT"),
            message: msg,
        }
    }
    else if let Ok(var) = std::env::var("KATHY_CHAT_MESSAGES") {
        let messages = var.split(",").map(String::from).collect::<Vec<String>>();
        if messages.len() < 1 {
            panic!("invalid KATHY_CHAT_MESSAGES");
        }
        self.chat = ChatGenConfig::OrderedList { messages }
    }
    else if let Ok(var) = std::env::var("KATHY_CHAT_RANDOM") {
        let length = var.parse::<usize>().expect("invalid KATHY_CHAT_RANDOM");
        self.chat = ChatGenConfig::Random { length }
    }
}});

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

#[cfg(test)]
mod test {
    use super::*;
    use nomad_test::test_utils;
    use std::{env, str::FromStr};

    #[test]
    #[serial_test::serial]
    fn it_overrides_config_from_env() {
        test_utils::run_test_with_env_sync("../fixtures/env.test-agents", move || {
            let mut config = KathyConfig::default();
            config.load_env_overrides();
            assert_eq!(
                config.chat,
                ChatGenConfig::OrderedList {
                    messages: vec![
                        "Chat message 1".to_string(),
                        "Chat message 2".to_string(),
                        "Chat message 3".to_string()
                    ]
                }
            );
            assert_eq!(config.interval, 999);
            assert_eq!(config.enabled, true);

            env::remove_var("KATHY_CHAT_MESSAGES");
            env::set_var(
                "KATHY_CHAT_RECIPIENT",
                "0x1111111111111111111111111111111111111111111111111111111111111111",
            );
            env::set_var("KATHY_CHAT_MESSAGE", "Chat message");
            config.load_env_overrides();
            assert_eq!(
                config.chat,
                ChatGenConfig::Static {
                    recipient: H256::from_str(
                        "0x1111111111111111111111111111111111111111111111111111111111111111"
                    )
                    .unwrap(),
                    message: "Chat message".to_string(),
                }
            );

            env::remove_var("KATHY_CHAT_RECIPIENT");
            env::remove_var("KATHY_CHAT_MESSAGE");
            env::set_var("KATHY_CHAT_RANDOM", "99");
            config.load_env_overrides();
            assert_eq!(config.chat, ChatGenConfig::Random { length: 99 });
        });
    }
}
