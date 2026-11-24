#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::significant_drop_tightening,
    reason = "simple RPC accessors; lock guards are scoped before waking callers"
)]

use std::{
    collections::HashMap,
    fmt,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    task::{Context, Poll},
    time::Duration,
};

use a2ui_author::{
    AgentToRendererMessage, CallRendererFunction, FunctionError, FunctionResult,
    RendererFunctionResponseMessage,
};
use futures::channel::oneshot;
use futures_timer::Delay;
use serde_json::Value;

use crate::{
    CatalogSet, FunctionDefinition,
    error::{self, CANCELLED, DISPOSED, DUPLICATE, TIMEOUT},
};

type CallResult = Result<Value, FunctionError>;

struct PendingEntry {
    surface_id: Option<String>,
    definition: FunctionDefinition,
    sender: oneshot::Sender<CallResult>,
}

#[derive(Default)]
struct State {
    disposed: bool,
    pending: HashMap<String, PendingEntry>,
}

/// Owns agent-initiated renderer calls and their correlated pending results.
pub struct RendererFunctionCalls {
    catalogs: CatalogSet,
    state: Arc<Mutex<State>>,
}

impl RendererFunctionCalls {
    pub fn new(catalogs: CatalogSet) -> Self {
        Self {
            catalogs,
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    pub fn catalogs(&self) -> &CatalogSet {
        &self.catalogs
    }

    /// Validates and constructs a call, then returns its message and pending result future.
    pub fn call(
        &self,
        function_call_id: impl Into<String>,
        surface_id: Option<String>,
        call: a2ui_author::FunctionCall,
        timeout: Option<Duration>,
    ) -> Result<(AgentToRendererMessage, PendingFunctionCall), FunctionError> {
        if timeout.is_some_and(|timeout| timeout.is_zero()) {
            return Err(error::invalid_call(
                "Function call timeout must be greater than zero",
            ));
        }
        let definition = self.catalogs.validate_renderer_call(&call)?;
        let function_call_id = function_call_id.into();
        let mut call_envelope = CallRendererFunction::new(function_call_id.clone(), call)?;
        if let Some(surface_id) = surface_id.clone() {
            call_envelope = call_envelope.surface_id(surface_id);
        }
        let message = call_envelope.into_message();
        let (sender, receiver) = oneshot::channel();
        {
            let mut state = self.lock();
            if state.disposed {
                return Err(error::error(
                    DISPOSED,
                    "Renderer function calls are disposed",
                ));
            }
            if state.pending.contains_key(&function_call_id) {
                return Err(error::error(
                    DUPLICATE,
                    format!("Function call ID '{function_call_id}' is already pending"),
                ));
            }
            state.pending.insert(
                function_call_id.clone(),
                PendingEntry {
                    surface_id,
                    definition,
                    sender,
                },
            );
        };

        let pending = PendingFunctionCall {
            function_call_id,
            receiver,
            timeout: timeout.map(|duration| Box::pin(Delay::new(duration))),
            state: Arc::clone(&self.state),
        };
        Ok((message, pending))
    }

    /// Resolves a matching pending call. Unknown IDs remain unhandled.
    pub fn resolve(&self, message: &RendererFunctionResponseMessage) -> bool {
        let response = &message.renderer_function_response;
        let entry = self.lock().pending.remove(response.function_call_id());
        let Some(entry) = entry else {
            return false;
        };
        let result = match response.result() {
            FunctionResult::Value(value) => entry
                .definition
                .validate_return(value)
                .map(|()| value.clone()),
            FunctionResult::Error(error) => Err(error.clone()),
        };
        let _ = entry.sender.send(result);
        true
    }

    /// Cancels one call by ID. Returns false when no such call is pending.
    pub fn cancel(&self, function_call_id: &str) -> bool {
        let entry = self.lock().pending.remove(function_call_id);
        let Some(entry) = entry else {
            return false;
        };
        let _ = entry.sender.send(Err(error::error(
            CANCELLED,
            format!("Function call '{function_call_id}' was cancelled"),
        )));
        true
    }

    /// Cancels all calls associated with a surface and returns their count.
    pub fn cancel_surface(&self, surface_id: &str) -> usize {
        let entries = {
            let mut state = self.lock();
            let ids = state
                .pending
                .iter()
                .filter(|(_, entry)| entry.surface_id.as_deref() == Some(surface_id))
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            ids.iter()
                .filter_map(|id| state.pending.remove(id))
                .collect::<Vec<_>>()
        };
        let count = entries.len();
        for entry in entries {
            let _ = entry.sender.send(Err(error::error(
                CANCELLED,
                format!("Function calls for surface '{surface_id}' were cancelled"),
            )));
        }
        count
    }

    /// Permanently disposes this owner and fails all pending calls with DISPOSED.
    pub fn dispose(&self) {
        let entries = {
            let mut state = self.lock();
            state.disposed = true;
            state
                .pending
                .drain()
                .map(|(_, entry)| entry)
                .collect::<Vec<_>>()
        };
        for entry in entries {
            let _ = entry.sender.send(Err(error::error(
                DISPOSED,
                "Renderer function calls were disposed",
            )));
        }
    }

    pub fn pending_len(&self) -> usize {
        self.lock().pending.len()
    }

    pub fn is_disposed(&self) -> bool {
        self.lock().disposed
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for RendererFunctionCalls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.lock();
        f.debug_struct("RendererFunctionCalls")
            .field("catalogs", &self.catalogs)
            .field("disposed", &state.disposed)
            .field("pending", &state.pending.keys())
            .finish_non_exhaustive()
    }
}

impl Drop for RendererFunctionCalls {
    fn drop(&mut self) {
        self.dispose();
    }
}

/// A single correlated renderer function result.
pub struct PendingFunctionCall {
    function_call_id: String,
    receiver: oneshot::Receiver<CallResult>,
    timeout: Option<Pin<Box<Delay>>>,
    state: Arc<Mutex<State>>,
}

impl PendingFunctionCall {
    pub fn function_call_id(&self) -> &str {
        &self.function_call_id
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Future for PendingFunctionCall {
    type Output = CallResult;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match Pin::new(&mut this.receiver).poll(cx) {
            Poll::Ready(Ok(result)) => return Poll::Ready(result),
            Poll::Ready(Err(_)) => {
                return Poll::Ready(Err(error::error(
                    DISPOSED,
                    "Renderer function call owner was dropped",
                )));
            }
            Poll::Pending => {}
        }
        if let Some(timeout) = &mut this.timeout
            && timeout.as_mut().poll(cx).is_ready()
        {
            this.lock().pending.remove(&this.function_call_id);
            return Poll::Ready(Err(error::error(
                TIMEOUT,
                format!("Function call '{}' timed out", this.function_call_id),
            )));
        }
        Poll::Pending
    }
}

impl fmt::Debug for PendingFunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingFunctionCall")
            .field("function_call_id", &self.function_call_id)
            .field("has_timeout", &self.timeout.is_some())
            .finish_non_exhaustive()
    }
}

impl Drop for PendingFunctionCall {
    fn drop(&mut self) {
        self.lock().pending.remove(&self.function_call_id);
    }
}
