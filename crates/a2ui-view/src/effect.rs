use std::{fmt, pin::Pin};

use crate::{ActionResult, EffectResult, MaybeSend};

#[cfg(not(target_arch = "wasm32"))]
type Update<S> = Box<dyn FnOnce(&mut S) + Send>;
#[cfg(target_arch = "wasm32")]
type Update<S> = Box<dyn FnOnce(&mut S)>;

#[cfg(not(target_arch = "wasm32"))]
type EffectFuture<S> = Pin<Box<dyn Future<Output = EffectResult<Update<S>>> + Send>>;
#[cfg(target_arch = "wasm32")]
type EffectFuture<S> = Pin<Box<dyn Future<Output = EffectResult<Update<S>>>>>;

/// An owned asynchronous operation followed by one state update.
///
/// Native effects are `Send`. WASM effects are local.
pub struct Effect<S> {
    future: EffectFuture<S>,
}

impl<S> Effect<S> {
    /// Creates an effect from a future that returns a state-update closure.
    #[must_use]
    pub fn new<F, U>(future: F) -> Self
    where
        F: Future<Output = EffectResult<U>> + MaybeSend + 'static,
        U: FnOnce(&mut S) + MaybeSend + 'static,
    {
        Self {
            future: Box::pin(async move {
                let update = future.await?;
                Ok(Box::new(update) as Update<S>)
            }),
        }
    }

    /// Drives the effect and applies its state update.
    ///
    /// # Errors
    ///
    /// Returns the effect error without applying the state update.
    pub async fn run(self, state: &mut S) -> ActionResult {
        let update = self.future.await?;
        update(state);
        Ok(())
    }
}

impl<S> fmt::Debug for Effect<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Effect").finish_non_exhaustive()
    }
}
