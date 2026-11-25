#[path = "app/application.rs"]
mod application;
#[cfg(feature = "function")]
#[path = "app/functions.rs"]
mod functions;
#[path = "app/support/mod.rs"]
mod support;
