use std::{
    collections::{HashMap, VecDeque},
    fmt,
    sync::Arc,
};

use a2a::A2AError;
use a2ui::AgentToRendererMessage;
use chrono::{DateTime, Utc};
use sdui::{Application, function::RendererMessageProcessor};
use serde::Serialize;
use tokio::sync::Mutex;

pub type SessionFactory<S> =
    Arc<dyn Fn() -> (Application<S>, RendererMessageProcessor) + Send + Sync + 'static>;
pub type SessionHandle<S> = Arc<Mutex<Session<S>>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SessionIdentity {
    pub principal: String,
    pub tenant: Option<String>,
}

impl SessionIdentity {
    #[must_use]
    pub fn new(principal: &str, tenant: Option<&str>) -> Self {
        Self {
            principal: principal.to_owned(),
            tenant: tenant.map(str::to_owned),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SessionKey {
    pub principal: String,
    pub tenant: Option<String>,
    pub context_id: String,
}

impl SessionKey {
    pub fn new(principal: Option<&str>, tenant: Option<&str>, context_id: &str) -> Self {
        Self {
            principal: principal.unwrap_or("anonymous").to_owned(),
            tenant: tenant.map(str::to_owned),
            context_id: context_id.to_owned(),
        }
    }
}

pub struct Session<S> {
    pub application: Application<S>,
    pub processor: RendererMessageProcessor,
    pub initial_messages: Option<Vec<AgentToRendererMessage>>,
    pub outbox: VecDeque<AgentToRendererMessage>,
    pub last_timestamp: Option<DateTime<Utc>>,
}

impl<S: Serialize + Send + Sync + 'static> Session<S> {
    /// Creates and starts a new session from the application factory.
    ///
    /// # Errors
    /// Returns an error if the initial application view is invalid.
    pub fn from_factory(factory: &SessionFactory<S>) -> Result<Self, A2AError> {
        let (application, processor) = factory();
        Self::restore(application, processor, None)
    }

    /// Starts an unstarted application containing restored state.
    ///
    /// The processor must be fresh. The initial surface is authoritative, the RPC outbox
    /// is empty, and pending function calls are never restored.
    ///
    /// # Errors
    /// Returns an error if the application is already started or its initial view is invalid.
    pub fn restore(
        mut application: Application<S>,
        processor: RendererMessageProcessor,
        last_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Self, A2AError> {
        let initial_messages = application.start().map_err(|error| {
            A2AError::internal(format!("failed to start A2UI application: {error}"))
        })?;
        Ok(Self {
            application,
            processor,
            initial_messages: Some(initial_messages),
            outbox: VecDeque::new(),
            last_timestamp,
        })
    }

    pub const fn state(&self) -> &S {
        self.application.state()
    }

    pub const fn last_timestamp(&self) -> Option<DateTime<Utc>> {
        self.last_timestamp
    }
}

#[async_trait::async_trait]
pub trait SessionStore<S>: Send + Sync + 'static {
    /// Returns a live session or restores one with `Session::restore`.
    async fn get_or_create(
        &self,
        key: &SessionKey,
        factory: &SessionFactory<S>,
    ) -> Result<SessionHandle<S>, A2AError>;

    /// Atomically saves application state and its last timestamp.
    ///
    /// Do not persist the initial messages, RPC outbox, or pending function calls.
    async fn save(&self, key: &SessionKey, session: &Session<S>) -> Result<(), A2AError>;
}

impl<S> fmt::Debug for Session<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session").finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct InMemorySessionStore<S> {
    sessions: Arc<Mutex<HashMap<SessionKey, SessionHandle<S>>>>,
}

impl<S> Default for InMemorySessionStore<S> {
    fn default() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S> fmt::Debug for InMemorySessionStore<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemorySessionStore")
            .finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl<S: Serialize + Send + Sync + 'static> SessionStore<S> for InMemorySessionStore<S> {
    async fn get_or_create(
        &self,
        key: &SessionKey,
        factory: &SessionFactory<S>,
    ) -> Result<SessionHandle<S>, A2AError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(key) {
            return Ok(Arc::clone(session));
        }
        let session = Arc::new(Mutex::new(Session::from_factory(factory)?));
        sessions.insert(key.clone(), Arc::clone(&session));
        drop(sessions);
        Ok(session)
    }

    async fn save(&self, _key: &SessionKey, _session: &Session<S>) -> Result<(), A2AError> {
        Ok(())
    }
}
