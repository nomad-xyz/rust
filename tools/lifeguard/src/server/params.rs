use std::{
    fmt::{self, Display},
    str::FromStr,
};

use regex::Regex;

/// Enum that is used to store the option of which agent caller wants to restart.
/// It is also used to capture and filter the parameter in a warp handler
#[derive(Debug, Clone)]
pub enum RestartableAgent {
    Updater,
    Relayer,
    Processor,
}

/// `FromStr` trait implementation, that is mainly used to filter the parameter in a warp handler
impl FromStr for RestartableAgent {
    type Err = ();
    fn from_str(s: &str) -> Result<RestartableAgent, ()> {
        match s {
            "updater" => Ok(RestartableAgent::Updater),
            "relayer" => Ok(RestartableAgent::Relayer),
            "processor" => Ok(RestartableAgent::Processor),
            _ => Err(()),
        }
    }
}

impl Display for RestartableAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestartableAgent::Updater => write!(f, "updater"),
            RestartableAgent::Relayer => write!(f, "relayer"),
            RestartableAgent::Processor => write!(f, "processor"),
        }
    }
}

/// Start that is used to store the name of the network that corresponds to an agent that caller wants to restart.
/// It is also used to capture and filter the parameter in a warp handler
#[derive(Debug, Clone)]
pub struct Network(String);

/// `FromStr` trait implementation, that is mainly used to filter the parameter in a warp handler
impl FromStr for Network {
    type Err = ();
    fn from_str(s: &str) -> Result<Network, ()> {
        if Regex::new(r"^[a-z0-9]+$").unwrap().is_match(s) {
            Ok(Network(s.to_string()))
        } else {
            Err(())
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
