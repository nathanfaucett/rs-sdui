use a2ui_author::{ActionEvent, ActionMessage, ProtocolVersion};
use a2ui_browser_example::{SURFACE_ID, application};
use futures::executor::block_on;
use serde_json::{Map, Value, json};

fn main() {
    block_on(async {
        let mut app = application();
        let mut messages = app.start().expect("initial view must be valid");
        messages.extend(
            app.handle(action("submit_name", json!({"name": "Ada"})))
                .await
                .expect("submit must succeed"),
        );
        messages.extend(
            app.handle(action("reset_form", json!({})))
                .await
                .expect("reset must succeed"),
        );
        let json = serde_json::to_string(&messages).expect("A2UI messages must serialize");
        println!("{json}");
    });
}

fn action(name: &str, context: Value) -> ActionMessage {
    ActionMessage {
        version: ProtocolVersion::V1_0,
        action: ActionEvent {
            name: name.to_owned(),
            user_message: None,
            surface_id: SURFACE_ID.to_owned(),
            source_component_id: "root".to_owned(),
            timestamp: "2026-11-22T00:00:00Z".to_owned(),
            context: match context {
                Value::Object(context) => context,
                _ => Map::new(),
            },
            metadata: None,
        },
    }
}
