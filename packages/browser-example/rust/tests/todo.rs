use a2ui::{ActionEvent, ActionMessage, AgentToRendererMessage, ProtocolVersion};
use a2ui_browser_example::{Filter, SURFACE_ID, TodoApp, application};
use futures::executor::block_on;
use serde_json::{Map, Value, json};

#[test]
fn initial_json_uses_catalog_functions_checks_and_typed_paths() {
    let mut app = application();
    let messages = app.start().expect("initial view must be valid");
    let initial = serde_json::to_string(&messages).expect("initial messages must serialize");

    assert!(initial.contains(r#""@call":"formatString""#));
    assert!(initial.contains(r#"Filter: ${/filter}""#));
    assert!(initial.contains(r#""@call":"openUrl""#));
    assert!(initial.contains(r#""url":"https://a2ui.org/""#));
    assert_eq!(initial.matches(r#""@checks""#).count(), 2);
    assert!(initial.contains(r#""@call":"regex""#));
    assert!(initial.contains(r#""pattern":"\\S""#));
    assert!(initial.contains(r#""@path":"/draft""#));
}

#[test]
fn add_edit_toggle_and_delete_form_a_complete_state_loop() {
    block_on(async {
        let mut app = application();
        app.start().expect("initial view must be valid");

        let updates = app
            .handle(action(
                "root_3_1",
                "add_todo",
                json!({"title": "  Learn A2UI  "}),
            ))
            .await
            .expect("add must succeed");
        assert_eq!(app.state().todos[0].title, "Learn A2UI");
        assert_eq!(app.state().draft, "");
        assert!(updates.iter().all(|message| !matches!(
            message,
            AgentToRendererMessage::UpdateDataModel(message)
                if message.update_data_model.path.is_none()
        )));

        app.handle(action(
            "root_3_1",
            "add_todo",
            json!({"title": "Build a todo list"}),
        ))
        .await
        .expect("second add must succeed");
        app.handle(action("root_4_0_0_2_1", "toggle", json!({"id": 1})))
            .await
            .expect("toggle must succeed");
        assert!(app.state().todos[0].completed);

        app.handle(action(
            "root_4_1_0_2_0",
            "save",
            json!({"id": 2, "title": "Ship the todo list"}),
        ))
        .await
        .expect("rename must succeed");
        assert_eq!(app.state().todos[1].title, "Ship the todo list");

        app.handle(action("root_4_0_0_2_2", "delete", json!({"id": 1})))
            .await
            .expect("delete must succeed");
        assert_eq!(app.state().todos.len(), 1);
        assert_eq!(app.state().todos[0].id, 2);
        assert_eq!(app.state().next_id, 3);
        app.handle(action("root_4_0_0_2_1", "toggle", json!({"id": 1})))
            .await
            .expect_err("a stale row action must not affect the remaining row");
        assert!(!app.state().todos[0].completed);
    });
}

#[test]
fn async_effects_load_example_tasks() {
    block_on(async {
        let mut app = application();
        app.start().expect("initial view must be valid");

        let updates = app
            .handle(action("root_4_1", "load_examples", json!({})))
            .await
            .expect("async loading must succeed");
        assert_eq!(app.state().todos.len(), 3);
        assert_eq!(app.state().next_id, 4);
        assert_eq!(app.state().todos[0].title, "Read the A2UI specification");
        assert!(app.state().todos[0].completed);
        assert_eq!(app.state().todos[2].title, "Connect an asynchronous action");
        assert!(updates.iter().any(|message| matches!(
            message,
            AgentToRendererMessage::UpdateDataModel(message)
                if message.update_data_model.path.as_deref() == Some("/todos")
        )));
        app.handle(action("root_4_1", "load_examples", json!({})))
            .await
            .expect_err("the one-shot effect must leave with the empty state");
    });
}

#[test]
fn filters_and_clear_completed_preserve_active_tasks() {
    block_on(async {
        let mut app = application();
        app.start().expect("initial view must be valid");
        app.handle(action(
            "root_3_1",
            "add_todo",
            json!({"title": "Active task"}),
        ))
        .await
        .expect("add must succeed");
        app.handle(action(
            "root_3_1",
            "add_todo",
            json!({"title": "Completed task"}),
        ))
        .await
        .expect("add must succeed");
        app.handle(action("root_4_1_0_2_1", "toggle", json!({"id": 2})))
            .await
            .expect("toggle must succeed");

        app.handle(action("root_6_2", "show_completed", json!({})))
            .await
            .expect("filter must succeed");
        assert_eq!(app.state().filter, Filter::Completed);
        app.handle(action("root_7", "clear_completed", json!({})))
            .await
            .expect("clear must succeed");
        assert_eq!(app.state().todos.len(), 1);
        assert_eq!(app.state().todos[0].title, "Active task");
        assert!(!app.state().todos[0].completed);

        app.handle(action("root_6_0", "show_all", json!({})))
            .await
            .expect("all filter must succeed");
        app.handle(action("root_4_0_0_2_1", "toggle", json!({"id": 1})))
            .await
            .expect("toggle must succeed");
        app.handle(action("root_6_1", "show_active", json!({})))
            .await
            .expect("active filter must succeed");
        assert_eq!(app.state().filter, Filter::Active);
    });
}

#[test]
fn empty_titles_roll_back_the_transition() {
    block_on(async {
        let mut app = application();
        app.start().expect("initial view must be valid");
        app.handle(action("root_3_1", "add_todo", json!({"title": "  "})))
            .await
            .expect_err("empty new titles must fail");
        assert_eq!(app.state(), &TodoApp::default());

        app.handle(action("root_3_1", "add_todo", json!({"title": "Valid"})))
            .await
            .expect("add must succeed");
        let previous = app.state().clone();
        app.handle(action(
            "root_4_0_0_2_0",
            "save",
            json!({"id": 1, "title": ""}),
        ))
        .await
        .expect_err("empty edited titles must fail");
        assert_eq!(app.state(), &previous);
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
