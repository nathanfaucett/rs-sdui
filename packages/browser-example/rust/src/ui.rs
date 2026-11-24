use a2ui_app::Application;
use a2ui_author::{A2uiData, ActionEvent, CheckRule, Event};
use a2ui_basic::v1::{
    self, Button, ButtonVariant, Card, Column, Row, Text, TextField, TextVariant,
};
use a2ui_view::{ActionResult, BoxError, Effect, Element, EventExt};

use crate::load::{LoadedTodo, load_examples};
use serde::Serialize;
use serde_json::{Value, json};

pub const SURFACE_ID: &str = "main";

#[derive(A2uiData, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

impl Filter {
    const fn includes(self, todo: &Todo) -> bool {
        match self {
            Self::All => true,
            Self::Active => !todo.completed,
            Self::Completed => todo.completed,
        }
    }
}

#[derive(A2uiData, Clone, Debug, PartialEq, Eq, Serialize)]
struct Todo {
    id: u64,
    title: String,
    completed: bool,
}

#[derive(A2uiData, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoApp {
    draft: String,
    todos: Vec<Todo>,
    filter: Filter,
    next_id: u64,
}

impl Default for TodoApp {
    fn default() -> Self {
        Self {
            draft: String::new(),
            todos: Vec::new(),
            filter: Filter::All,
            next_id: 1,
        }
    }
}

impl TodoApp {
    fn add(&self, action: &ActionEvent) -> ActionResult<Self> {
        let title = action_title(action)?;
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or("todo ID space exhausted")?;
        let mut next = self.clone();
        next.todos.push(Todo {
            id: self.next_id,
            title,
            completed: false,
        });
        next.next_id = next_id;
        next.draft.clear();
        Ok(next)
    }

    fn with_loaded(&self, examples: Vec<LoadedTodo>) -> ActionResult<Self> {
        let mut next = self.clone();
        for example in examples {
            let title = example.title.trim();
            if title.is_empty() {
                return Err("loaded tasks need non-empty titles".into());
            }
            let id = next.next_id;
            next.next_id = next
                .next_id
                .checked_add(1)
                .ok_or("todo ID space exhausted")?;
            next.todos.push(Todo {
                id,
                title: title.to_owned(),
                completed: example.completed,
            });
        }
        next.filter = Filter::All;
        Ok(next)
    }

    fn rename(&self, action: &ActionEvent) -> ActionResult<Self> {
        let id = action_id(action)?;
        let title = action_title(action)?;
        let mut next = self.clone();
        let todo = next
            .todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or_else(|| format!("todo {id} no longer exists"))?;
        todo.title = title;
        Ok(next)
    }

    fn toggle(&self, action: &ActionEvent) -> ActionResult<Self> {
        let id = action_id(action)?;
        let mut next = self.clone();
        let todo = next
            .todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or_else(|| format!("todo {id} no longer exists"))?;
        todo.completed = !todo.completed;
        Ok(next)
    }

    fn remove(&self, action: &ActionEvent) -> ActionResult<Self> {
        let id = action_id(action)?;
        if !self.todos.iter().any(|todo| todo.id == id) {
            return Err(format!("todo {id} no longer exists").into());
        }
        let mut next = self.clone();
        next.todos.retain(|todo| todo.id != id);
        Ok(next)
    }

    fn with_filter(&self, filter: Filter) -> Self {
        Self {
            filter,
            ..self.clone()
        }
    }

    fn clear_completed(&self) -> Self {
        let mut next = self.clone();
        next.todos.retain(|todo| !todo.completed);
        next
    }

    fn remaining(&self) -> usize {
        self.todos.iter().filter(|todo| !todo.completed).count()
    }

    fn view(&self) -> Element {
        let remaining = self.remaining();
        let summary = if remaining == 1 {
            "1 task remaining".to_owned()
        } else {
            format!("{remaining} tasks remaining")
        };
        let completed = self.todos.len() - remaining;

        Column::new((
            Text::new("A2UI Todo List"),
            Text::new(v1::functions::format_string(
                "Declarative Rust state. Standard A2UI updates. Filter: ${/filter}",
            ))
            .variant(TextVariant::Caption),
            Button::new(
                Text::new("A2UI documentation"),
                v1::functions::open_url("https://a2ui.org/"),
            )
            .variant(ButtonVariant::Borderless),
            composer(),
            self.todo_list(),
            Text::new(summary).variant(TextVariant::Caption),
            self.filters(),
            Button::new(
                Text::new(format!("Clear completed ({completed})")),
                Event::new("clear_completed").on(|state: &Self, _| state.clear_completed()),
            )
            .variant(ButtonVariant::Borderless),
        ))
        .into()
    }

    fn todo_list(&self) -> Element {
        let items = self
            .todos
            .iter()
            .enumerate()
            .filter(|(_, todo)| self.filter.includes(todo))
            .map(|(index, todo)| todo_card(index, todo))
            .collect::<Vec<_>>();
        if items.is_empty() {
            let message = match self.filter {
                Filter::All => "No tasks yet. Add one above or load the examples.",
                Filter::Active => "No active tasks.",
                Filter::Completed => "No completed tasks.",
            };
            let empty = Text::new(message).variant(TextVariant::Caption);
            if self.todos.is_empty() && self.filter == Filter::All {
                Column::new((empty, load_examples_button())).into()
            } else {
                empty.into()
            }
        } else {
            Column::new(items).into()
        }
    }

    fn filters(&self) -> Row {
        Row::new((
            self.filter_button(Filter::All, "All", "show_all"),
            self.filter_button(Filter::Active, "Active", "show_active"),
            self.filter_button(Filter::Completed, "Completed", "show_completed"),
        ))
    }

    fn filter_button(&self, filter: Filter, label: &'static str, event: &'static str) -> Button {
        let variant = if self.filter == filter {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Default
        };
        Button::new(
            Text::new(label),
            Event::new(event).on(move |state: &Self, _| state.with_filter(filter)),
        )
        .variant(variant)
    }
}

fn composer() -> Row {
    let draft = TodoApp::paths().draft();
    let title_rule = non_empty_title(draft.clone());
    Row::new((
        TextField::new("New task", draft.clone())
            .placeholder("What needs to be done?")
            .checks([title_rule.clone()]),
        Button::new(
            Text::new("Add task"),
            Event::new("add_todo")
                .context("title", draft)
                .on(|state: &TodoApp, action| state.add(&action)),
        )
        .variant(ButtonVariant::Primary)
        .checks([title_rule]),
    ))
}

fn load_examples_button() -> Button {
    Button::new(
        Text::new("Load example tasks"),
        Event::new("load_examples").on_effect(|state: &TodoApp, _| {
            let state = state.clone();
            Effect::new(async move {
                let examples = load_examples().await?;
                state.with_loaded(examples)
            })
        }),
    )
}

fn todo_card(index: usize, todo: &Todo) -> Card {
    let id = todo.id;
    let title_path = TodoApp::paths().todos().index(index).title();
    let title_rule = non_empty_title(title_path.clone());
    let toggle_label = if todo.completed { "Reopen" } else { "Complete" };
    let status = if todo.completed {
        "Completed"
    } else {
        "Active"
    };

    Card::new(Column::new((
        Text::new(status).variant(TextVariant::Caption),
        TextField::new(format!("Edit {}", todo.title), title_path.clone())
            .checks([title_rule.clone()]),
        Row::new((
            Element::from(
                Button::new(
                    Text::new("Save"),
                    Event::new("save")
                        .context("id", Value::from(id))
                        .context("title", title_path)
                        .on(|state: &TodoApp, action| state.rename(&action)),
                )
                .checks([title_rule]),
            )
            .property(
                "@accessibility",
                json!({"label": format!("Save {}", todo.title)}),
            ),
            Element::from(Button::new(
                Text::new(toggle_label),
                Event::new("toggle")
                    .context("id", Value::from(id))
                    .on(|state: &TodoApp, action| state.toggle(&action)),
            ))
            .property(
                "@accessibility",
                json!({"label": format!("{toggle_label} {}", todo.title)}),
            ),
            Element::from(
                Button::new(
                    Text::new("Delete"),
                    Event::new("delete")
                        .context("id", Value::from(id))
                        .on(|state: &TodoApp, action| state.remove(&action)),
                )
                .variant(ButtonVariant::Borderless),
            )
            .property(
                "@accessibility",
                json!({"label": format!("Delete {}", todo.title)}),
            ),
        )),
    )))
}

fn non_empty_title(path: impl Into<a2ui_author::DynamicString>) -> CheckRule {
    CheckRule::new(v1::functions::regex("\\S", path)).message("Enter a non-whitespace title")
}

fn action_id(action: &ActionEvent) -> Result<u64, BoxError> {
    action
        .context
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| "a todo action needs an unsigned integer context.id".into())
}

fn action_title(action: &ActionEvent) -> Result<String, BoxError> {
    action
        .context
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "a todo needs a non-empty string context.title".into())
}

#[must_use]
pub fn application() -> Application<TodoApp> {
    Application::new(
        SURFACE_ID,
        v1::CATALOG_ID,
        TodoApp::default(),
        TodoApp::view,
    )
}

#[cfg(test)]
mod tests {
    use a2ui_author::{AgentToRendererMessage, ProtocolVersion};
    use futures::executor::block_on;
    use serde_json::{Map, json};

    use super::*;

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

    fn action(source_component_id: &str, name: &str, context: Value) -> a2ui_author::ActionMessage {
        a2ui_author::ActionMessage {
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
}
