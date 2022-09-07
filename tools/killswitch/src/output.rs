use crate::errors::Error;
use serde::Serialize;

/// KillSwitch response showing success / failure of configuration
/// and tx submission. Gets serialized to json
#[derive(Serialize)]
pub(crate) struct Output {
    /// The original command `killswitch` was run with
    pub command: String,
    /// The success / failure message
    pub message: Message,
}

/// A wrapper for success / failure messages
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Message {
    /// An error message wrapper
    ErrorMessage(String),
    // other ...
}

impl From<Error> for Message {
    /// Convert `KillSwitchError` to `Message`
    fn from(error: Error) -> Self {
        Message::ErrorMessage(format!("{}", error))
    }
}
