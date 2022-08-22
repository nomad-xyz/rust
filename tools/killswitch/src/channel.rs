use std::fmt::Display;

#[derive(Debug)]
pub(crate) struct Channel {}

impl Channel {}

impl Display for Channel {
    // Describe the readiness of this channel to be killed, including
    // missing or invalid config as appropriate
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}
