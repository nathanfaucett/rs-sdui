use std::{cell::RefCell, fmt::Display, rc::Rc};

use a2ui_app::Application;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::{TodoApp, application, decode_action};

thread_local! {
    static APPLICATION: Rc<RefCell<Application<TodoApp>>> = Rc::new(RefCell::new(application()));
}

/// Starts the local Rust application and returns standard A2UI JSON messages.
#[wasm_bindgen]
pub fn initial_messages() -> Result<String, JsValue> {
    APPLICATION.with(|application| {
        let messages = application
            .try_borrow_mut()
            .map_err(js_error)?
            .start()
            .map_err(js_error)?;
        serde_json::to_string(&messages).map_err(js_error)
    })
}

/// Returns the current surface as a JSON array for a newly connected renderer.
#[wasm_bindgen]
pub fn snapshot_messages() -> Result<String, JsValue> {
    APPLICATION.with(|application| {
        let message = application
            .try_borrow()
            .map_err(js_error)?
            .snapshot()
            .map_err(js_error)?;
        serde_json::to_string(&[message]).map_err(js_error)
    })
}

/// Applies one official renderer action and returns standard A2UI JSON updates.
#[wasm_bindgen]
pub async fn handle_action(input: &str) -> Result<String, JsValue> {
    let action = decode_action(input).map_err(js_error)?;
    let application = APPLICATION.with(Rc::clone);
    let messages = application
        .try_borrow_mut()
        .map_err(js_error)?
        .handle(action)
        .await
        .map_err(js_error)?;
    serde_json::to_string(&messages).map_err(js_error)
}

fn js_error(error: impl Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
