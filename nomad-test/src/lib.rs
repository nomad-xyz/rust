//! Nomad. OPTimistic Interchain Communication
//!
//! This crate contains mocks and utilities for testing Nomad agents.

#![forbid(unsafe_code)]
#![cfg_attr(test, warn(missing_docs))]
#![warn(unused_extern_crates)]
#![forbid(where_clauses_object_safety)]

/// Mock contracts
pub mod mocks;
pub use mocks::MockError;

/// Testing utilities
pub mod test_utils;
