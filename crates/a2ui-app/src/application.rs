use std::{collections::BTreeMap, fmt};

use a2ui_author::{
    ActionMessage, AgentToRendererMessage, CreateSurface, DataModel, DeleteSurface,
    UpdateComponents, UpdateDataModel,
};
use a2ui_view::{Element, Handler, MaybeSend, MaybeSync};
use serde::Serialize;

use crate::{
    ApplicationError, data,
    view::{changed_components, validate},
};

#[cfg(not(target_arch = "wasm32"))]
type View<S> = Box<dyn Fn(&S) -> Element + Send + Sync>;
#[cfg(target_arch = "wasm32")]
type View<S> = Box<dyn Fn(&S) -> Element>;

struct Current<S> {
    surface: CreateSurface,
    handlers: BTreeMap<(String, String), Handler<S>>,
}

/// Owns serializable state and a pure declarative view for one A2UI surface.
///
/// State must serialize to a JSON object. It becomes the root A2UI data model.
/// Completed actions emit only changed data paths. Bound paths are also refreshed
/// because A2UI inputs can change the renderer-local model without sending a message.
/// Component updates use structural diffs.
pub struct Application<S> {
    surface_id: String,
    catalog_id: String,
    state: S,
    view: View<S>,
    current: Option<Current<S>>,
}

impl<S: fmt::Debug> fmt::Debug for Application<S> {
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

impl<S: Serialize + 'static> Application<S> {
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

    /// Returns a complete `createSurface` message for a newly connected renderer.
    ///
    /// This clones the stored surface without rerunning the view or changing lifecycle state.
    ///
    /// # Errors
    ///
    /// Returns an error if the application is stopped or closed.
    pub fn snapshot(&self) -> Result<AgentToRendererMessage, ApplicationError> {
        let current = self.current.as_ref().ok_or(ApplicationError::NotStarted)?;
        Ok(current.surface.clone().into_message())
    }

    /// Runs the handler matching the action's source component and event name.
    ///
    /// Surface and source scoping lets separate components reuse event names.
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
        let current_view = current.as_ref().ok_or(ApplicationError::NotStarted)?;
        if message.action.surface_id != current_view.surface.surface_id {
            return Err(ApplicationError::UnexpectedSurface {
                expected: current_view.surface.surface_id.clone(),
                actual: message.action.surface_id,
            });
        }

        let name = message.action.name.clone();
        let source_component_id = message.action.source_component_id.clone();
        let key = (source_component_id.clone(), name.clone());
        let handler = current_view.handlers.get(&key).ok_or_else(|| {
            ApplicationError::InvalidAction(format!(
                "unknown action {name:?} from component {source_component_id:?}"
            ))
        })?;
        let next_state = handler
            .run(state, message.action)
            .await
            .map_err(|error| ApplicationError::InvalidAction(error.to_string()))?;
        let next = render_current(surface_id, catalog_id, view, &next_state)?;
        validate(&next.surface)?;
        let messages = transition_messages(&current_view.surface, &next.surface);
        *current = Some(next);
        *state = next_state;
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
    for (source_component_id, handler) in parts.handlers {
        let (event, handler) = handler.into_handler::<S>().map_err(|event| {
            ApplicationError::InvalidView(format!(
                "action {:?} has a handler for another state type",
                event.name
            ))
        })?;
        let key = (source_component_id.clone(), event.name.clone());
        if handlers.insert(key, handler).is_some() {
            return Err(ApplicationError::InvalidView(format!(
                "action {:?} has duplicate handlers on component {source_component_id:?}",
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
    let mut messages = data::updates(current, next)
        .into_iter()
        .map(UpdateDataModel::into_message)
        .collect::<Vec<_>>();
    let components = changed_components(current, next);
    if !components.is_empty() {
        messages.push(UpdateComponents::new(&next.surface_id, components).into_message());
    }
    messages
}
