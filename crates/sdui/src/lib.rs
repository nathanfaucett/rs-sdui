//! Server-driven UI applications and transport-independent authoring for A2UI v1.0.
//!
//! Rendering, transport, and renderer-side expression evaluation stay outside
//! this crate. Function validation and RPC are available through `function`.

mod app;
#[cfg(feature = "function")]
pub mod function;
mod state;
mod view;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use app::{application::Application, error::ApplicationError};
pub use sdui_derive::A2uiData;
pub use state::{A2uiData, CollectionPath, StatePath};
pub use view::{
    action::{ActionResult, BoxError, IntoActionResult},
    effect::Effect,
    element::{Children, Element, IntoChildren, SurfaceExt},
    event::{EventExt, EventHandler, ViewAction},
    platform::{MaybeSend, MaybeSync},
};
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub use wasm::local::WasmApplication;
