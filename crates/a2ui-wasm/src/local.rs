use std::{cell::RefCell, fmt::Display, rc::Rc};

use a2ui_app::Application;
use a2ui_author::{ActionMessage, RendererToAgentMessage};
use serde::Serialize;
use wasm_bindgen::JsValue;

/// Owns one local application behind the borrow rules required by browser calls.
#[derive(Debug)]
pub struct WasmApplication<S> {
    application: Rc<RefCell<Application<S>>>,
}

impl<S> WasmApplication<S> {
    #[must_use]
    pub fn new(application: Application<S>) -> Self {
        Self {
            application: Rc::new(RefCell::new(application)),
        }
    }
}

impl<S> Clone for WasmApplication<S> {
    fn clone(&self) -> Self {
        Self {
            application: Rc::clone(&self.application),
        }
    }
}

impl<S: Serialize + 'static> WasmApplication<S> {
    /// Starts the application and serializes its initial standard A2UI messages.
    ///
    /// # Errors
    ///
    /// Returns an error for concurrent borrows, invalid startup, or JSON serialization.
    pub fn initial_messages(&self) -> Result<String, JsValue> {
        let messages = self
            .application
            .try_borrow_mut()
            .map_err(js_error)?
            .start()
            .map_err(js_error)?;
        serde_json::to_string(&messages).map_err(js_error)
    }

    /// Serializes a complete surface snapshot for a newly connected renderer.
    ///
    /// # Errors
    ///
    /// Returns an error for concurrent borrows, a stopped application, or JSON serialization.
    pub fn snapshot_messages(&self) -> Result<String, JsValue> {
        let message = self
            .application
            .try_borrow()
            .map_err(js_error)?
            .snapshot()
            .map_err(js_error)?;
        serde_json::to_string(&[message]).map_err(js_error)
    }

    /// Decodes one standard action and serializes its standard A2UI updates.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, concurrent actions, failed transitions, or serialization.
    #[allow(
        clippy::await_holding_refcell_ref,
        clippy::future_not_send,
        reason = "local WASM is single-threaded; the borrow rejects re-entrant actions until completion"
    )]
    pub async fn handle_action(&self, input: &str) -> Result<String, JsValue> {
        let action = decode_action(input).map_err(js_error)?;
        let application = Rc::clone(&self.application);
        let messages = application
            .try_borrow_mut()
            .map_err(js_error)?
            .handle(action)
            .await
            .map_err(js_error)?;
        serde_json::to_string(&messages).map_err(js_error)
    }
}

fn decode_action(input: &str) -> Result<ActionMessage, String> {
    let message: RendererToAgentMessage =
        serde_json::from_str(input).map_err(|error| format!("invalid A2UI JSON: {error}"))?;
    match message {
        RendererToAgentMessage::Action(action) => Ok(action),
        _ => Err("the local application accepts standard A2UI action messages only".to_owned()),
    }
}

fn js_error(error: impl Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
