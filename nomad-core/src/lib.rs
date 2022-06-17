//! Nomad. OPTimistic Interchain Communication
//!
//! This crate contains core primitives, traits, and types for Nomad
//! implementations.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![forbid(unsafe_code)]
#![forbid(where_clauses_object_safety)]

pub use accumulator;

/// AWS global state and init
pub mod aws;

/// DB related utilities
pub mod db;

/// Model instantatiations of the on-chain structures
pub mod models {
    /// A simple Home chain Nomad implementation
    mod home;

    /// A simple Replica chain Nomad implementation
    mod replica;

    pub use self::{home::*, replica::*};
}

/// Async Traits for Homes & Replicas for use in applications
mod traits;
pub use traits::*;

mod signer;
pub use signer::*;

/// Utilities to match contract values
pub mod utils;

/// Testing utilities
pub mod test_utils;

/// Core nomad system data structures
mod types;
pub use types::*;

/// Test functions that output json files for Solidity tests
#[cfg(feature = "output")]
pub mod test_output;

mod chain;
pub use chain::*;

pub use nomad_types::NomadIdentifier;

use ethers::core::types::{SignatureError, H256};

/// Enum for validity of a list (of updates or messages)
#[derive(Debug)]
pub enum ListValidity {
    /// Empty list
    Empty,
    /// Valid list
    Valid,
    /// Invalid list
    Invalid,
}

/// Error types for Nomad
#[derive(Debug, thiserror::Error)]
pub enum NomadError {
    /// Signature Error pasthrough
    #[error(transparent)]
    SignatureError(#[from] SignatureError),
    /// Update does not build off the current root
    #[error("Update has wrong current root. Expected: {expected}. Got: {actual}.")]
    WrongCurrentRoot {
        /// The provided root
        actual: H256,
        /// The current root
        expected: H256,
    },
    /// Update specifies a new root that is not in the queue. This is an
    /// improper update and is slashable
    #[error("Update has unknown new root: {0}")]
    UnknownNewRoot(H256),
    /// IO error from Read/Write usage
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
