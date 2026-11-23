use std::iter::once;

use a2ui_app::{Application, ApplicationError};
use a2ui_author::{
    A2uiPath, ActionEvent, ActionMessage, AgentToRendererMessage, Event, ProtocolVersion,
};
use a2ui_basic::v1::{self, Button, Column, Text};
use a2ui_view::{Effect, Element, EventExt};
use futures::{
    executor::block_on,
    io::{AsyncReadExt, Cursor},
};
use serde::Serialize;
use serde_json::{Map, json};

#[derive(A2uiPath, Clone, Debug, Default, Serialize)]
struct Counter {
    count: i32,
}

/// A reusable component: typed props in, a declarative view out.
fn counter_label() -> impl Into<Element> + 'static {
    Text::new(Counter::paths().count())
}

fn counter_view(_state: &Counter) -> Element {
    Column::new((
        counter_label(),
        Button::new(
            Text::new("Increment"),
            Event::new("increment").on(|state: &Counter, _| Counter {
                count: state.count + 1,
            }),
        ),
        Button::new(
            Text::new("Noop"),
            Event::new("noop").on(|state: &Counter, _| Counter { count: state.count }),
        ),
    ))
    .into()
}

fn counter() -> Application<Counter> {
    Application::new("main", v1::CATALOG_ID, Counter::default(), counter_view)
}

#[test]
fn complete_tree_serializes_as_standard_a2ui_json() {
    let messages = counter().start().unwrap();
    assert_eq!(
        serde_json::to_value(messages).unwrap(),
        json!([{
            "version": "1.0",
            "createSurface": {
                "surfaceId": "main",
                "catalogId": v1::CATALOG_ID,
                "components": [
                    {
                        "id": "root",
                        "component": "Column",
                        "children": ["root_0", "root_1", "root_2"]
                    },
                    {
                        "id": "root_0",
                        "component": "Text",
                        "text": {"@path": "/count"}
                    },
                    {
                        "id": "root_1",
                        "component": "Button",
                        "child": "root_1_0",
                        "action": {"event": {"name": "increment"}}
                    },
                    {"id": "root_1_0", "component": "Text", "text": "Increment"},
                    {
                        "id": "root_2",
                        "component": "Button",
                        "child": "root_2_0",
                        "action": {"event": {"name": "noop"}}
                    },
                    {"id": "root_2_0", "component": "Text", "text": "Noop"}
                ],
                "dataModel": {"count": 0}
            }
        }])
    );
}

#[test]
fn basic_catalog_components_render_state_and_actions() {
    block_on(async {
        let mut app = counter();
        assert!(matches!(
            app.start().as_deref(),
            Ok([AgentToRendererMessage::CreateSurface(_)])
        ));

        let updates = app.handle(action("increment", "main")).await.unwrap();
        assert_eq!(app.state().count, 1);
        assert_eq!(updates.len(), 1);
        assert!(matches!(
            &updates[0],
            AgentToRendererMessage::UpdateDataModel(message)
                if message.update_data_model.path.as_deref() == Some("/count")
                    && message.update_data_model.value == json!(1)
        ));

        app.handle(action("unknown", "main")).await.unwrap_err();
        assert_eq!(app.state().count, 1);
        assert_eq!(
            app.handle(action("increment", "other")).await.unwrap_err(),
            ApplicationError::UnexpectedSurface {
                expected: "main".to_owned(),
                actual: "other".to_owned(),
            }
        );

        // A no-op still repairs input changes made only inside the renderer.
        let updates = app.handle(action("noop", "main")).await.unwrap();
        assert_eq!(updates.len(), 1);
        assert!(matches!(
            &updates[0],
            AgentToRendererMessage::UpdateDataModel(message)
                if message.update_data_model.path.as_deref() == Some("/count")
                    && message.update_data_model.value == json!(1)
        ));
    });
}

#[derive(Clone, Serialize)]
struct Detail {
    show_detail: bool,
}

#[test]
fn conditional_children_do_not_recreate_the_surface() {
    block_on(async {
        let mut app = Application::new(
            "main",
            v1::CATALOG_ID,
            Detail { show_detail: true },
            |state: &Detail| {
                let children = once(Text::new("1"))
                    .chain(state.show_detail.then(|| Text::new("Detail")))
                    .collect::<Vec<_>>();
                Column::new((
                    Column::new(children),
                    Button::new(
                        Text::new("Hide detail"),
                        Event::new("hide").on(|_: &Detail, _| Detail { show_detail: false }),
                    ),
                ))
            },
        );
        app.start().unwrap();

        let updates = app.handle(action("hide", "main")).await.unwrap();
        assert!(matches!(
            &updates[1],
            AgentToRendererMessage::UpdateComponents(message)
                if message.update_components.components.len() == 1
                    && message.update_components.components[0].id() == Some("root_0")
        ));
        assert!(
            !updates
                .iter()
                .any(|message| matches!(message, AgentToRendererMessage::DeleteSurface(_)))
        );
    });
}

#[test]
fn async_effects_apply_io_results_to_state() {
    block_on(async {
        let input = Cursor::new(b"42".to_vec());
        let mut app = Application::new(
            "main",
            v1::CATALOG_ID,
            Counter::default(),
            move |_state: &Counter| {
                let input = input.clone();
                Column::new((
                    counter_label(),
                    Button::new(
                        Text::new("Load"),
                        Event::new("load").on_effect(move |_, _| {
                            let mut input = input.clone();
                            Effect::new(async move {
                                let mut value = String::new();
                                input.read_to_string(&mut value).await?;
                                let count = value.parse()?;
                                Ok(Counter { count })
                            })
                        }),
                    ),
                ))
            },
        );
        app.start().unwrap();

        assert_send(app.handle(action("load", "main")))
            .await
            .unwrap();
        assert_eq!(app.state().count, 42);
    });
}

#[test]
fn close_allows_a_clean_restart() {
    block_on(async {
        let mut app = counter();
        app.start().unwrap();
        app.handle(action("increment", "main")).await.unwrap();
        assert!(matches!(
            app.close().as_deref(),
            Ok([AgentToRendererMessage::DeleteSurface(_)])
        ));
        assert_eq!(
            app.handle(action("noop", "main")).await.unwrap_err(),
            ApplicationError::NotStarted
        );
        app.start().unwrap();
        assert_eq!(app.start().unwrap_err(), ApplicationError::AlreadyStarted);
    });
}

#[cfg(not(target_arch = "wasm32"))]
const fn assert_send<T: Send>(value: T) -> T {
    value
}

#[cfg(target_arch = "wasm32")]
const fn assert_send<T>(value: T) -> T {
    value
}

fn action(name: &str, surface_id: &str) -> ActionMessage {
    ActionMessage {
        version: ProtocolVersion::V1_0,
        action: ActionEvent {
            name: name.to_owned(),
            user_message: None,
            surface_id: surface_id.to_owned(),
            source_component_id: "root".to_owned(),
            timestamp: "2026-11-22T00:00:00Z".to_owned(),
            context: Map::new(),
            metadata: None,
        },
    }
}
