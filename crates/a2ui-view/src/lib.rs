//! Declarative A2UI view trees and typed event callbacks.

pub mod action;
pub mod effect;
pub mod element;
pub mod event;
mod platform;
pub mod surface;

pub use action::*;
pub use effect::Effect;
pub use element::*;
pub use event::*;
pub use platform::{MaybeSend, MaybeSync};
pub use surface::Surface;
