use std::{
    fmt::{self, Display},
    str::FromStr,
};

use regex::Regex;

pub enum RestartableAgent {
    Updater,
    Relayer,
    Processor,
}

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

pub struct Network(String);

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

// impl ToString for Network {
//     fn to_string(&self) -> String {
//         self.0.clone()
//     }
// }

impl Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
