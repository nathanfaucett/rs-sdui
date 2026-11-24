//! Server-driven UI applications and transport-independent authoring for A2UI v1.0.
//!
//! Rendering, transport, and renderer-side expression evaluation stay outside
//! this crate. Function validation and RPC are available through `function`.

extern crate self as sdui;

mod app;
pub mod basic;
#[cfg(feature = "function")]
pub mod function;
mod protocol;
mod view;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use app::{application::Application, error::ApplicationError};
pub use protocol::{
    component::{CheckRule, ChildList, ChildTemplate, Component, Condition},
    dynamic::{
        Action, DataBinding, DynamicBoolean, DynamicNumber, DynamicString, DynamicStringList,
        DynamicValue, Event, FunctionCall, number,
    },
    message::{
        ActionEvent, ActionMessage, AgentFunctionResponseMessage, AgentToRendererMessage,
        CallAgentFunction, CallAgentFunctionMessage, CallRendererFunction,
        CallRendererFunctionMessage, CreateSurface, CreateSurfaceMessage, DataModel, DeleteSurface,
        DeleteSurfaceMessage, ErrorMessage, FunctionError, FunctionResponse, FunctionResult,
        GenericError, Metadata, ProtocolVersion, RendererError, RendererFunctionResponseMessage,
        RendererToAgentMessage, UpdateComponents, UpdateComponentsMessage, UpdateDataModel,
        UpdateDataModelMessage, ValidationError, ValidationErrorCode,
    },
    state::{A2uiData, CollectionPath, StatePath},
};
pub use sdui_derive::A2uiData;
pub use view::{
    action::{ActionResult, BoxError, IntoActionResult},
    effect::Effect,
    element::{Children, Element, IntoChildren},
    event::{EventExt, EventHandler, ViewAction},
    platform::{MaybeSend, MaybeSync},
};
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub use wasm::local::WasmApplication;
