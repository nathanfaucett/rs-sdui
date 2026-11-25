use std::{error::Error, fmt, pin::Pin};

use a2ui::ActionEvent;

use crate::{Effect, MaybeSend, MaybeSync};

#[cfg(not(target_arch = "wasm32"))]
pub type BoxError = Box<dyn Error + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type BoxError = Box<dyn Error>;

/// The next state or an action error. Use `?` for ordinary Rust errors.
pub type ActionResult<S> = Result<S, BoxError>;

/// Converts an infallible or fallible value into the next state.
pub trait IntoActionResult<S> {
    /// Converts the transition result.
    ///
    /// # Errors
    ///
    /// Returns the action's error unchanged.
    fn into_action_result(self) -> ActionResult<S>;
}

impl<S> IntoActionResult<S> for S {
    fn into_action_result(self) -> ActionResult<S> {
        Ok(self)
    }
}

impl<S> IntoActionResult<S> for ActionResult<S> {
    fn into_action_result(self) -> Self {
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
type BoxActionFuture<'a, S> = Pin<Box<dyn Future<Output = ActionResult<S>> + Send + 'a>>;
#[cfg(target_arch = "wasm32")]
type BoxActionFuture<'a, S> = Pin<Box<dyn Future<Output = ActionResult<S>> + 'a>>;

#[cfg(not(target_arch = "wasm32"))]
type Callback<S> = Box<dyn Fn(&S, ActionEvent) -> BoxActionFuture<'_, S> + Send + Sync>;
#[cfg(target_arch = "wasm32")]
type Callback<S> = Box<dyn Fn(&S, ActionEvent) -> BoxActionFuture<'_, S>>;

/// A pure state transition attached to a declarative event.
pub struct Handler<S> {
    callback: Callback<S>,
}

impl<S> Handler<S> {
    /// Creates a synchronous transition from the current state and action.
    pub(crate) fn action<F, R>(handler: F) -> Self
    where
        F: Fn(&S, ActionEvent) -> R + MaybeSend + MaybeSync + 'static,
        R: IntoActionResult<S> + MaybeSend,
        S: MaybeSend + MaybeSync,
    {
        Self {
            callback: Box::new(move |state, action| {
                let result = handler(state, action).into_action_result();
                Box::pin(async move { result })
            }),
        }
    }

    /// Creates an asynchronous effect that returns the next state.
    pub(crate) fn effect<F>(handler: F) -> Self
    where
        F: Fn(&S, ActionEvent) -> Effect<S> + MaybeSend + MaybeSync + 'static,
        S: MaybeSend + MaybeSync,
    {
        Self {
            callback: Box::new(move |state, action| {
                let effect = handler(state, action);
                Box::pin(async move { effect.run().await })
            }),
        }
    }

    /// Runs the transition without selecting an executor or mutating current state.
    ///
    /// # Errors
    ///
    /// Returns the action or effect error unchanged.
    #[allow(
        clippy::future_not_send,
        reason = "native handlers are Send when S is Send; WASM handlers are local"
    )]
    pub(crate) async fn run(&self, state: &S, action: ActionEvent) -> ActionResult<S> {
        (self.callback)(state, action).await
    }
}

impl<S> fmt::Debug for Handler<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handler").finish_non_exhaustive()
    }
}
