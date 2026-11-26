//! Rust types for standard A2UI v1.0 components, values, and messages.
//!
//! Application state, declarative views, transport, and rendering stay outside
//! this crate.

mod capability;
mod component;
mod dynamic;
mod message;

pub use capability::{
    AgentCapabilities, AgentCapability, RendererCapabilities, RendererCapability, RendererDataModel,
};
pub use component::{CheckRule, ChildList, ChildTemplate, Component, Condition};
pub use dynamic::{
    Action, DataBinding, DynamicBoolean, DynamicNumber, DynamicString, DynamicStringList,
    DynamicValue, Event, FunctionCall, number,
};
pub use message::{
    ActionEvent, ActionMessage, AgentFunctionResponseMessage, AgentToRendererMessage,
    CallAgentFunction, CallAgentFunctionMessage, CallRendererFunction, CallRendererFunctionMessage,
    CreateSurface, CreateSurfaceMessage, DataModel, DeleteSurface, DeleteSurfaceMessage,
    ErrorMessage, FunctionError, FunctionResponse, FunctionResult, GenericError, Metadata,
    ProtocolVersion, RendererError, RendererFunctionResponseMessage, RendererToAgentMessage,
    UpdateComponents, UpdateComponentsMessage, UpdateDataModel, UpdateDataModelMessage,
    ValidationError, ValidationErrorCode,
};
