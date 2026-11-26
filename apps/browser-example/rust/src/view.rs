use a2ui::{CheckRule, Event};
use sdui::{Element, EventExt};
use serde_json::{Value, json};

use crate::basic::v1::{
    self, Button, ButtonVariant, Card, Column, Row, RowAlign, Text, TextField, TextVariant,
};
use crate::state::{Filter, Todo, TodoApp};

impl TodoApp {
    #[must_use]
    pub fn view(&self) -> Element {
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
                Filter::All => "No tasks yet. Add one above.",
                Filter::Active => "No active tasks.",
                Filter::Completed => "No completed tasks.",
            };
            Text::new(message).variant(TextVariant::Caption).into()
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
            .checks([non_blank_title_if_present(draft.clone())]),
        Button::new(
            Text::new("+"),
            Event::new("add_todo")
                .context("title", draft)
                .on(|state: &TodoApp, action| state.add(&action)),
        )
        .variant(ButtonVariant::Primary)
        .checks([title_rule]),
    ))
    .align(RowAlign::Start)
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
            .checks([non_blank_title_if_present(title_path.clone())]),
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
                .variant(ButtonVariant::Default),
            )
            .property(
                "@accessibility",
                json!({"label": format!("Delete {}", todo.title)}),
            ),
        )),
    )))
}

fn non_empty_title(path: impl Into<a2ui::DynamicString>) -> CheckRule {
    CheckRule::new(v1::functions::regex("\\S", path)).message("Enter a non-whitespace title")
}

fn non_blank_title_if_present(path: a2ui::DataBinding) -> CheckRule {
    CheckRule::new(v1::functions::or(json!([
        a2ui::DynamicBoolean::from(v1::functions::not(v1::functions::required(path.clone(),))),
        a2ui::DynamicBoolean::from(v1::functions::regex("\\S", path))
    ])))
    .message("Enter a non-whitespace title")
}
