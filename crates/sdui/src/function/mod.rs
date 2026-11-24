//! Offline A2UI v1.0 function validation and bidirectional function RPC.

mod agent;
mod catalog;
mod error;
mod processor;
mod renderer;

pub use agent::{AgentFunctions, FunctionContext};
pub use catalog::{AllowedCallers, Catalog, CatalogSet, FunctionDefinition, ReturnType};
pub use processor::{Dispatch, RendererMessageProcessor};
pub use renderer::{PendingFunctionCall, RendererFunctionCalls};
