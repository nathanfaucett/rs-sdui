use std::{any::Any, fmt};

use a2ui_author::{Action, ActionEvent, Event, FunctionCall};

use crate::{Effect, Handler, IntoActionResult, MaybeSend, MaybeSync};

#[cfg(not(target_arch = "wasm32"))]
type BoxHandler = Box<dyn Any + Send>;
#[cfg(target_arch = "wasm32")]
type BoxHandler = Box<dyn Any>;

/// A standard event and its Rust-only state transition.
pub struct EventHandler {
    event: Event,
    handler: BoxHandler,
}

impl EventHandler {
    #[must_use]
    pub const fn event(&self) -> &Event {
        &self.event
    }

    fn new<S: 'static>(event: Event, handler: Handler<S>) -> Self {
        Self {
            event,
            handler: Box::new(handler),
        }
    }

    /// Recovers the typed transition for the application that owns this view.
    ///
    /// # Errors
    ///
    /// Returns the standard event when its handler targets another state type.
    pub fn into_handler<S: 'static>(self) -> Result<(Event, Handler<S>), Event> {
        let event = self.event.clone();
        self.handler
            .downcast::<Handler<S>>()
            .map(|handler| (self.event, *handler))
            .map_err(|_error| event)
    }
}

impl fmt::Debug for EventHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventHandler")
            .field("event", &self.event)
            .finish_non_exhaustive()
    }
}

/// A standard A2UI action or a standard event with a Rust-only transition.
#[derive(Debug)]
pub enum ViewAction {
    Standard(Action),
    Handled(EventHandler),
}

impl From<Action> for ViewAction {
    fn from(value: Action) -> Self {
        Self::Standard(value)
    }
}

impl From<Event> for ViewAction {
    fn from(value: Event) -> Self {
        Self::Standard(Action::Event(value))
    }
}

impl From<FunctionCall> for ViewAction {
    fn from(value: FunctionCall) -> Self {
        Self::Standard(Action::FunctionCall(value))
    }
}

impl From<EventHandler> for ViewAction {
    fn from(value: EventHandler) -> Self {
        Self::Handled(value)
    }
}

/// Adds typed state transitions to standard A2UI events.
pub trait EventExt {
    /// Declares a synchronous transition beside the event that triggers it.
    fn on<S, F, R>(self, handler: F) -> EventHandler
    where
        S: MaybeSend + MaybeSync + 'static,
        F: Fn(&S, ActionEvent) -> R + MaybeSend + MaybeSync + 'static,
        R: IntoActionResult<S> + MaybeSend;

    /// Declares an asynchronous I/O effect beside the event that triggers it.
    fn on_effect<S, F>(self, handler: F) -> EventHandler
    where
        S: MaybeSend + MaybeSync + 'static,
        F: Fn(&S, ActionEvent) -> Effect<S> + MaybeSend + MaybeSync + 'static;
}

impl EventExt for Event {
    fn on<S, F, R>(self, handler: F) -> EventHandler
    where
        S: MaybeSend + MaybeSync + 'static,
        F: Fn(&S, ActionEvent) -> R + MaybeSend + MaybeSync + 'static,
        R: IntoActionResult<S> + MaybeSend,
    {
        EventHandler::new(self, Handler::action(handler))
    }

    fn on_effect<S, F>(self, handler: F) -> EventHandler
    where
        S: MaybeSend + MaybeSync + 'static,
        F: Fn(&S, ActionEvent) -> Effect<S> + MaybeSend + MaybeSync + 'static,
    {
        EventHandler::new(self, Handler::effect(handler))
    }
}
