//! Agent configuration (logging, intervals, addresses, etc).
//!
//! All structs defined in this module include public data only. The real agent
//! settings blocks are separate/different from these {Agent}Config blocks and
//! can contain signers. Functionality of these config blocks is minimized to
//! just the data itself.

mod logging;
pub use logging::*;

pub mod kathy;
pub mod processor;
pub mod relayer;
pub mod updater;
pub mod watcher;
