use std::{error::Error, fmt, pin::Pin};

use a2ui_author::ActionEvent;

use crate::{Effect, MaybeSend};

#[cfg(not(target_arch = "wasm32"))]
pub type BoxError = Box<dyn Error + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type BoxError = Box<dyn Error>;

/// The result of an action. Use `?` for ordinary Rust errors.
pub type ActionResult = Result<(), BoxError>;

/// The result of an asynchronous effect before its state update is applied.
pub type EffectResult<T> = Result<T, BoxError>;

/// Converts an infallible update or a fallible update into an action result.
pub trait IntoActionResult {
    /// Converts the update result.
    ///
    /// # Errors
    ///
    /// Returns the action's error unchanged.
    fn into_action_result(self) -> ActionResult;
}

impl IntoActionResult for () {
    fn into_action_result(self) -> ActionResult {
        Ok(())
    }
}

impl IntoActionResult for ActionResult {
    fn into_action_result(self) -> ActionResult {
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
type BoxActionFuture<'a> = Pin<Box<dyn Future<Output = ActionResult> + Send + 'a>>;
#[cfg(target_arch = "wasm32")]
type BoxActionFuture<'a> = Pin<Box<dyn Future<Output = ActionResult> + 'a>>;

#[cfg(not(target_arch = "wasm32"))]
type Callback<S> = Box<dyn FnMut(&mut S, ActionEvent) -> BoxActionFuture<'_> + Send>;
#[cfg(target_arch = "wasm32")]
type Callback<S> = Box<dyn FnMut(&mut S, ActionEvent) -> BoxActionFuture<'_>>;

/// A typed state transition attached to a declarative event.
pub struct Handler<S> {
    callback: Callback<S>,
}

impl<S> Handler<S> {
    /// Creates a synchronous state transition.
    pub fn action<F, R>(mut handler: F) -> Self
    where
        F: FnMut(&mut S, ActionEvent) -> R + MaybeSend + 'static,
        R: IntoActionResult + MaybeSend,
    {
        Self {
            callback: Box::new(move |state, action| {
                let result = handler(state, action).into_action_result();
                Box::pin(async move { result })
            }),
        }
    }

    /// Creates an asynchronous effect followed by one state update.
    pub fn effect<F>(mut handler: F) -> Self
    where
        F: FnMut(&mut S, ActionEvent) -> Effect<S> + MaybeSend + 'static,
        S: MaybeSend,
    {
        Self {
            callback: Box::new(move |state, action| {
                let effect = handler(state, action);
                Box::pin(async move { effect.run(state).await })
            }),
        }
    }

    /// Runs the transition without selecting an executor.
    ///
    /// # Errors
    ///
    /// Returns the action or effect error unchanged.
    #[allow(
        clippy::future_not_send,
        reason = "native handlers are Send when S is Send; WASM handlers are local"
    )]
    pub async fn run(&mut self, state: &mut S, action: ActionEvent) -> ActionResult {
        (self.callback)(state, action).await
    }
}

impl<S> fmt::Debug for Handler<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handler").finish_non_exhaustive()
    }
}
