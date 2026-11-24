//! Offline A2UI v1.0 function validation and bidirectional function RPC.

mod error;

pub mod agent;
pub mod catalog;
pub mod processor;
pub mod renderer;

pub use agent::*;
pub use catalog::*;
pub use processor::*;
pub use renderer::*;
