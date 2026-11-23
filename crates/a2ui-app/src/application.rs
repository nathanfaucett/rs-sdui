use std::{collections::BTreeMap, fmt};

use a2ui_author::{
    ActionMessage, AgentToRendererMessage, CreateSurface, DataModel, DeleteSurface,
    UpdateComponents, UpdateDataModel,
};
use a2ui_view::{Element, Handler, MaybeSend, MaybeSync};
use serde::Serialize;

use crate::{
    ApplicationError,
    view::{changed_components, data_model, validate},
};

#[cfg(not(target_arch = "wasm32"))]
type View<S> = Box<dyn Fn(&S) -> Element + Send + Sync>;
#[cfg(target_arch = "wasm32")]
type View<S> = Box<dyn Fn(&S) -> Element>;

struct Current<S> {
    surface: CreateSurface,
    handlers: BTreeMap<String, Handler<S>>,
}

/// Owns serializable state and a pure declarative view for one A2UI surface.
///
/// State must serialize to a JSON object. It becomes the root A2UI data model.
/// Every completed action replaces that data model. A2UI inputs can change
/// the renderer-local model without sending a message, so diffing only Rust
/// views could leave stale input values. Component updates use structural diffs.
pub struct Application<S: Clone> {
    surface_id: String,
    catalog_id: String,
    state: S,
    view: View<S>,
    current: Option<Current<S>>,
}

impl<S: Clone + fmt::Debug> fmt::Debug for Application<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Application")
            .field("surface_id", &self.surface_id)
            .field("catalog_id", &self.catalog_id)
            .field("state", &self.state)
            .field(
                "current",
                &self.current.as_ref().map(|current| &current.surface),
            )
            .finish_non_exhaustive()
    }
}

impl<S: Clone + Serialize + 'static> Application<S> {
    /// Creates one mounted catalog surface from owned state and a pure view function.
    #[must_use]
    pub fn new<F, V>(
        surface_id: impl Into<String>,
        catalog_id: impl Into<String>,
        state: S,
        view: F,
    ) -> Self
    where
        F: Fn(&S) -> V + MaybeSend + MaybeSync + 'static,
        V: Into<Element>,
    {
        Self {
            surface_id: surface_id.into(),
            catalog_id: catalog_id.into(),
            state,
            view: Box::new(move |state| view(state).into()),
            current: None,
        }
    }

    #[must_use]
    pub const fn state(&self) -> &S {
        &self.state
    }

    /// Validates and emits the initial `createSurface` message.
    ///
    /// # Errors
    ///
    /// Returns an error if the application is started, state is not a data-model
    /// object, handlers are invalid, or the view is invalid.
    pub fn start(&mut self) -> Result<Vec<AgentToRendererMessage>, ApplicationError> {
        if self.current.is_some() {
            return Err(ApplicationError::AlreadyStarted);
        }
        let current = render_current(&self.surface_id, &self.catalog_id, &self.view, &self.state)?;
        validate(&current.surface)?;
        let message = current.surface.clone().into_message();
        self.current = Some(current);
        Ok(vec![message])
    }

    /// Runs one event handler declared in the current view and emits A2UI updates.
    ///
    /// The returned future drives synchronous updates and async side effects. It
    /// does not spawn work or select an executor. State changes are committed only
    /// after the action and next-view validation succeed.
    ///
    /// # Errors
    ///
    /// Returns an error for a stopped application, a wrong surface, an unknown or
    /// failed action, invalid state, invalid handlers, or an invalid next view.
    #[allow(
        clippy::future_not_send,
        reason = "native handlers are Send when S is Send; WASM handlers are local"
    )]
    pub async fn handle(
        &mut self,
        message: ActionMessage,
    ) -> Result<Vec<AgentToRendererMessage>, ApplicationError> {
        let Self {
            surface_id,
            catalog_id,
            state,
            view,
            current,
        } = self;
        let current = current.as_mut().ok_or(ApplicationError::NotStarted)?;
        if message.action.surface_id != current.surface.surface_id {
            return Err(ApplicationError::UnexpectedSurface {
                expected: current.surface.surface_id.clone(),
                actual: message.action.surface_id,
            });
        }

        let name = message.action.name.clone();
        let handler = current
            .handlers
            .get_mut(&name)
            .ok_or_else(|| ApplicationError::InvalidAction(format!("unknown action {name:?}")))?;
        let mut next_state = state.clone();
        handler
            .run(&mut next_state, message.action)
            .await
            .map_err(|error| ApplicationError::InvalidAction(error.to_string()))?;
        let next = render_current(surface_id, catalog_id, view, &next_state)?;
        validate(&next.surface)?;
        let messages = transition_messages(&current.surface, &next.surface);
        *state = next_state;
        *current = next;
        Ok(messages)
    }

    /// Emits `deleteSurface` and stops the application.
    ///
    /// # Errors
    ///
    /// Returns an error if the application is already stopped.
    pub fn close(&mut self) -> Result<Vec<AgentToRendererMessage>, ApplicationError> {
        let current = self.current.take().ok_or(ApplicationError::NotStarted)?;
        Ok(vec![
            DeleteSurface::new(current.surface.surface_id).into_message(),
        ])
    }
}

fn render_current<S: Serialize + 'static>(
    surface_id: &str,
    catalog_id: &str,
    view: &View<S>,
    state: &S,
) -> Result<Current<S>, ApplicationError> {
    let data_model = serde_json::to_value(state)
        .and_then(serde_json::from_value::<DataModel>)
        .map_err(|error| {
            ApplicationError::InvalidState(format!(
                "state must serialize to a root data-model object: {error}"
            ))
        })?;
    let parts = view(state).into_parts();
    let mut handlers = BTreeMap::new();
    for handler in parts.handlers {
        let (event, handler) = handler.into_handler::<S>().map_err(|event| {
            ApplicationError::InvalidView(format!(
                "action {:?} has a handler for another state type",
                event.name
            ))
        })?;
        if handlers.insert(event.name.clone(), handler).is_some() {
            return Err(ApplicationError::InvalidView(format!(
                "action {:?} has duplicate handlers",
                event.name
            )));
        }
    }
    let surface = CreateSurface::new(surface_id, catalog_id)
        .components(parts.components)
        .data_model(data_model);
    Ok(Current { surface, handlers })
}

fn transition_messages(
    current: &CreateSurface,
    next: &CreateSurface,
) -> Vec<AgentToRendererMessage> {
    let mut messages =
        vec![UpdateDataModel::root(&next.surface_id, data_model(next)).into_message()];
    let components = changed_components(current, next);
    if !components.is_empty() {
        messages.push(UpdateComponents::new(&next.surface_id, components).into_message());
    }
    messages
}
