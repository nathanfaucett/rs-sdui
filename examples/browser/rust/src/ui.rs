use a2ui_app::Application;
use a2ui_author::{DataBinding, Event, RendererToAgentMessage};
use a2ui_basic::v1::{self, Button, ButtonVariant, Column, Row, Text, TextField};
use a2ui_view::{ActionResult, Element, EventExt};
use serde::Serialize;
use serde_json::Value;

pub const SURFACE_ID: &str = "main";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Editing,
    Submitted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileApp {
    #[serde(skip)]
    phase: Phase,
    name: String,
    submissions: u32,
}

impl Default for ProfileApp {
    fn default() -> Self {
        Self {
            phase: Phase::Editing,
            name: "A2UI".to_owned(),
            submissions: 0,
        }
    }
}

impl ProfileApp {
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn submissions(&self) -> u32 {
        self.submissions
    }

    fn status(&self) -> String {
        match self.phase {
            Phase::Editing => "Edit the name, then submit.".to_owned(),
            Phase::Submitted => format!("Welcome, {}!", self.name),
        }
    }

    fn view(&self) -> Element {
        Column::new((
            Text::new("Rust state, standard A2UI"),
            Text::new(self.status()),
            Text::new(format!("Submissions: {}", self.submissions)),
            TextField::new("Name", DataBinding::new("/name")).placeholder("Enter your name"),
            form_actions(DataBinding::new("/name")),
        ))
        .into()
    }
}

fn form_actions(name: DataBinding) -> impl Into<Element> + 'static {
    Row::new((
        Button::new(
            Text::new("Submit"),
            Event::new("submit_name").context("name", name).on(
                |state: &ProfileApp, action| -> ActionResult<ProfileApp> {
                    let name = action
                        .context
                        .get("name")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .ok_or("submit_name needs a non-empty string context.name")?;
                    Ok(ProfileApp {
                        phase: Phase::Submitted,
                        name: name.to_owned(),
                        submissions: state.submissions.saturating_add(1),
                    })
                },
            ),
        )
        .variant(ButtonVariant::Primary),
        Button::new(
            Text::new("Reset"),
            Event::new("reset_form").on(|_: &ProfileApp, _| ProfileApp::default()),
        )
        .variant(ButtonVariant::Borderless),
    ))
}

#[must_use]
pub fn application() -> Application<ProfileApp> {
    Application::new(
        SURFACE_ID,
        v1::CATALOG_ID,
        ProfileApp::default(),
        ProfileApp::view,
    )
}

/// Decodes a standard renderer-to-agent envelope and requires an action.
///
/// # Errors
///
/// Returns an error for invalid JSON, a non-v1.0 envelope, or a non-action message.
pub fn decode_action(input: &str) -> Result<a2ui_author::ActionMessage, String> {
    let message: RendererToAgentMessage =
        serde_json::from_str(input).map_err(|error| format!("invalid A2UI JSON: {error}"))?;
    match message {
        RendererToAgentMessage::Action(action) => Ok(action),
        _ => Err("the local example accepts standard A2UI action messages only".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use a2ui_author::{ActionEvent, AgentToRendererMessage, ProtocolVersion};
    use futures::executor::block_on;
    use serde_json::{Map, json};

    use super::*;

    #[test]
    fn submit_and_reset_form_a_complete_state_loop() {
        block_on(async {
            let mut app = application();
            app.start().expect("initial view must be valid");

            let updates = app
                .handle(action("submit_name", json!({"name": "Ada"})))
                .await
                .expect("submit must succeed");
            assert_eq!(app.state().phase(), Phase::Submitted);
            assert_eq!(app.state().name(), "Ada");
            assert_eq!(app.state().submissions(), 1);
            assert!(updates.iter().any(|message| matches!(
                message,
                AgentToRendererMessage::UpdateDataModel(message)
                    if message.update_data_model.path.as_deref() == Some("/name")
                        && message.update_data_model.value == json!("Ada")
            )));
            assert!(updates.iter().any(|message| matches!(
                message,
                AgentToRendererMessage::UpdateDataModel(message)
                    if message.update_data_model.path.as_deref() == Some("/submissions")
                        && message.update_data_model.value == json!(1)
            )));

            app.handle(action("reset_form", json!({})))
                .await
                .expect("reset must succeed");
            assert_eq!(app.state(), &ProfileApp::default());
        });
    }

    #[test]
    fn empty_names_do_not_change_state() {
        block_on(async {
            let mut app = application();
            app.start().expect("initial view must be valid");
            app.handle(action("submit_name", json!({"name": "  "})))
                .await
                .expect_err("empty names must fail");
            assert_eq!(app.state(), &ProfileApp::default());
        });
    }

    fn action(name: &str, context: Value) -> a2ui_author::ActionMessage {
        a2ui_author::ActionMessage {
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
}
