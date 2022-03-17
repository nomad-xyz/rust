//! Common Nomad data structures used across various parts of the stack (configuration, SDK, agents)

mod core;
pub use crate::core::*;

mod error;
pub use error::*;

pub mod agent;
