use a2ui_app::Application;
use a2ui_author::{A2uiData, Event, RendererToAgentMessage};
use a2ui_basic::v1::{self, Button, Column, Text};
use a2ui_function::{CatalogSet, Dispatch, RendererMessageProcessor};
use a2ui_view::{Element, EventExt};
use futures::executor::block_on;
use serde::Serialize;

#[derive(A2uiData, Default, Serialize)]
struct Counter {
    count: i32,
}

fn view(_state: &Counter) -> Element {
    Column::new((
        Text::new(Counter::paths().count()),
        Button::new(
            Text::new("Increment"),
            Event::new("increment").on(|state: &Counter, _| Counter {
                count: state.count + 1,
            }),
        ),
    ))
    .into()
}

#[test]
fn function_processor_forwards_actions_to_the_application() {
    block_on(async {
        let catalogs = CatalogSet::from_json(include_str!("../../a2ui-basic/catalog.json"))
            .expect("the official Basic Catalog must compile offline");
        let processor = RendererMessageProcessor::new(catalogs);
        let message: RendererToAgentMessage = serde_json::from_value(serde_json::json!({
            "version": "1.0",
            "action": {
                "name": "increment",
                "surfaceId": "main",
                "sourceComponentId": "root_1",
                "timestamp": "2026-11-23T00:00:00Z",
                "context": {}
            }
        }))
        .expect("the standard action must deserialize");

        let Dispatch::Unhandled(RendererToAgentMessage::Action(action)) =
            processor.process(message).await
        else {
            panic!("an ordinary action must remain unhandled by function RPC");
        };

        let mut app = Application::new("main", v1::CATALOG_ID, Counter::default(), view);
        app.start().expect("the application must start");
        let updates = app.handle(action).await.expect("the action must succeed");
        assert_eq!(app.state().count, 1);
        assert_eq!(updates.len(), 1);
        assert_eq!(
            serde_json::to_value(&updates[0]).expect("update JSON"),
            serde_json::json!({
                "version": "1.0",
                "updateDataModel": {
                    "surfaceId": "main",
                    "path": "/count",
                    "value": 1
                }
            })
        );
    });
}
