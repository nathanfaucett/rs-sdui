use std::{fmt, pin::Pin};

use crate::{ActionResult, MaybeSend};

#[cfg(not(target_arch = "wasm32"))]
type EffectFuture<S> = Pin<Box<dyn Future<Output = ActionResult<S>> + Send>>;
#[cfg(target_arch = "wasm32")]
type EffectFuture<S> = Pin<Box<dyn Future<Output = ActionResult<S>>>>;

/// An owned asynchronous operation that returns the next application state.
///
/// Native effects are `Send`. WASM effects are local.
pub struct Effect<S> {
    future: EffectFuture<S>,
}

impl<S> Effect<S> {
    /// Creates an effect from a future that returns the next state.
    #[must_use]
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = ActionResult<S>> + MaybeSend + 'static,
    {
        Self {
            future: Box::pin(future),
        }
    }

    /// Drives the effect and returns the next state.
    ///
    /// # Errors
    ///
    /// Returns the effect error without producing a state transition.
    pub(crate) async fn run(self) -> ActionResult<S> {
        self.future.await
    }
}

impl<S> fmt::Debug for Effect<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Effect").finish_non_exhaustive()
    }
}
