use futures::executor::block_on;
use sdui::{ActionEvent, ActionMessage, ProtocolVersion};
use sdui_browser_example::{SURFACE_ID, application};
use serde_json::{Map, Value, json};

fn main() {
    block_on(async {
        let mut app = application();
        let mut messages = app.start().expect("initial view must be valid");
        for (source_component_id, name, context) in [
            ("root_3_1", "add_todo", json!({"title": "Learn A2UI"})),
            (
                "root_3_1",
                "add_todo",
                json!({"title": "Build a todo list"}),
            ),
            ("root_4_0_0_2_1", "toggle", json!({"id": 1})),
            (
                "root_4_1_0_2_0",
                "save",
                json!({"id": 2, "title": "Ship the todo list"}),
            ),
            ("root_6_1", "show_active", json!({})),
        ] {
            messages.extend(
                app.handle(action(source_component_id, name, context))
                    .await
                    .unwrap_or_else(|error| panic!("{name} must succeed: {error}")),
            );
        }
        let json = serde_json::to_string(&messages).expect("A2UI messages must serialize");
        println!("{json}");
    });
}

fn action(source_component_id: &str, name: &str, context: Value) -> ActionMessage {
    ActionMessage {
        version: ProtocolVersion::V1_0,
        action: ActionEvent {
            name: name.to_owned(),
            user_message: None,
            surface_id: SURFACE_ID.to_owned(),
            source_component_id: source_component_id.to_owned(),
            timestamp: "2026-11-22T00:00:00Z".to_owned(),
            context: match context {
                Value::Object(context) => context,
                _ => Map::new(),
            },
            metadata: None,
        },
    }
}
