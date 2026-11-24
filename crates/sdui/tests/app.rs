#[path = "app/application.rs"]
mod application;
#[cfg(feature = "function")]
#[path = "app/functions.rs"]
mod functions;
