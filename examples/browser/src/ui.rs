use a2ui_author::{AgentToRendererMessage, DataBinding, DataModel, Event};
use a2ui_basic::v1::{self, Button, ButtonVariant, Column, Text, TextField};
use serde_json::json;
use wasm_bindgen::prelude::wasm_bindgen;

#[must_use]
pub fn surface_messages() -> Vec<AgentToRendererMessage> {
    vec![
        v1::surface("main")
            .components([
                Column::new("root", ["heading", "name_field", "submit"]).into(),
                Text::new("heading", "A2UI from Rust").into(),
                TextField::new("name_field", "Name", DataBinding::new("/form/name"))
                    .placeholder("Enter your name")
                    .into(),
                Text::new("submit_label", "Submit").into(),
                Button::new(
                    "submit",
                    "submit_label",
                    Event::new("submit_name").context("name", DataBinding::new("/form/name")),
                )
                .variant(ButtonVariant::Primary)
                .into(),
            ])
            .data_model(DataModel::new().with("form", json!({"name": "A2UI"})))
            .into_message(),
    ]
}

/// Returns one standard A2UI createSurface message in a JSON array.
///
/// # Panics
///
/// Panics only if the in-memory authoring values cannot serialize.
#[must_use]
#[wasm_bindgen]
pub fn messages() -> String {
    serde_json::to_string(&surface_messages()).expect("A2UI messages must serialize")
}
