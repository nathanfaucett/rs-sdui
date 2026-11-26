//! Official A2UI v1.0 extension mapping for A2A v1.

#[cfg(feature = "client")]
mod client;
mod data_part;
mod metadata;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
mod session;
mod task;

#[cfg(feature = "client")]
pub use client::A2uiClient;
pub use data_part::{
    A2UI_MIME_TYPE, BindingError, DecodedMessages, MessageError, agent_data_part,
    decode_agent_messages, decode_renderer_messages, renderer_data_part,
};
pub use metadata::{
    A2UI_EXTENSION_URI, agent_extension, read_renderer_capabilities, read_renderer_data_model,
    renderer_metadata,
};
#[cfg(feature = "server")]
pub use server::{InMemoryTaskStoreFactory, SduiExecutor, SduiRequestHandler, TaskStoreFactory};
#[cfg(feature = "server")]
pub use session::{
    InMemorySessionStore, Session, SessionFactory, SessionHandle, SessionIdentity, SessionKey,
    SessionStore,
};
pub use task::{decode_task, decode_tasks};
