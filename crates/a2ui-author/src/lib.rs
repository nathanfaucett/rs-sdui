//! Authoring types for the official A2UI v1.0 JSON protocol.
//!
//! This crate does not process surfaces, evaluate expressions, or select a
//! transport. Use an official A2UI Core SDK for those tasks.

pub mod component;
pub mod dynamic;
pub mod message;

pub use component::*;
pub use dynamic::*;
pub use message::*;
