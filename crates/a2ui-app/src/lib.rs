//! State-driven application runtime for standard A2UI v1.0 authoring.

pub mod application;
pub mod error;
mod view;

pub use application::Application;
pub use error::ApplicationError;
