#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::derive_partial_eq_without_eq,
    reason = "fluent authoring methods return new values"
)]

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{Component, FunctionCall};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolVersion {
    #[default]
    #[serde(rename = "1.0")]
    V1_0,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DataModel(pub Map<String, Value>);

impl DataModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.0.insert(name.into(), value.into());
        self
    }
}

impl From<Map<String, Value>> for DataModel {
    fn from(value: Map<String, Value>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub extensions: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurface {
    pub surface_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_data_model: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_model: Option<DataModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

impl CreateSurface {
    pub fn new(surface_id: impl Into<String>, catalog_id: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.into(),
            catalog_id: Some(catalog_id.into()),
            send_data_model: None,
            components: None,
            data_model: None,
            metadata: None,
        }
    }

    pub fn without_catalog(surface_id: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.into(),
            catalog_id: None,
            send_data_model: None,
            components: None,
            data_model: None,
            metadata: None,
        }
    }

    pub fn components(mut self, components: impl IntoIterator<Item = Component>) -> Self {
        self.components = Some(components.into_iter().collect());
        self
    }

    pub fn data_model(mut self, data_model: impl Into<DataModel>) -> Self {
        self.data_model = Some(data_model.into());
        self
    }

    pub const fn send_data_model(mut self, send: bool) -> Self {
        self.send_data_model = Some(send);
        self
    }

    pub fn metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub const fn into_message(self) -> AgentToRendererMessage {
        AgentToRendererMessage::CreateSurface(CreateSurfaceMessage {
            version: ProtocolVersion::V1_0,
            create_surface: self,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurfaceMessage {
    pub version: ProtocolVersion,
    pub create_surface: CreateSurface,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponents {
    pub surface_id: String,
    pub components: Vec<Component>,
}

impl UpdateComponents {
    pub fn new(
        surface_id: impl Into<String>,
        components: impl IntoIterator<Item = Component>,
    ) -> Self {
        Self {
            surface_id: surface_id.into(),
            components: components.into_iter().collect(),
        }
    }

    pub const fn into_message(self) -> AgentToRendererMessage {
        AgentToRendererMessage::UpdateComponents(UpdateComponentsMessage {
            version: ProtocolVersion::V1_0,
            update_components: self,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentsMessage {
    pub version: ProtocolVersion,
    pub update_components: UpdateComponents,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModel {
    pub surface_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub value: Value,
}

impl UpdateDataModel {
    pub fn new(
        surface_id: impl Into<String>,
        path: impl Into<String>,
        value: impl Into<Value>,
    ) -> Self {
        Self {
            surface_id: surface_id.into(),
            path: Some(path.into()),
            value: value.into(),
        }
    }

    pub fn root(surface_id: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            surface_id: surface_id.into(),
            path: None,
            value: value.into(),
        }
    }

    pub const fn into_message(self) -> AgentToRendererMessage {
        AgentToRendererMessage::UpdateDataModel(UpdateDataModelMessage {
            version: ProtocolVersion::V1_0,
            update_data_model: self,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModelMessage {
    pub version: ProtocolVersion,
    pub update_data_model: UpdateDataModel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    pub surface_id: String,
}

impl DeleteSurface {
    pub fn new(surface_id: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.into(),
        }
    }

    pub const fn into_message(self) -> AgentToRendererMessage {
        AgentToRendererMessage::DeleteSurface(DeleteSurfaceMessage {
            version: ProtocolVersion::V1_0,
            delete_surface: self,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurfaceMessage {
    pub version: ProtocolVersion,
    pub delete_surface: DeleteSurface,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponse {
    pub function_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FunctionError>,
}

impl FunctionResponse {
    pub fn success(function_call_id: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            function_call_id: function_call_id.into(),
            value: Some(value.into()),
            error: None,
        }
    }

    pub fn failure(function_call_id: impl Into<String>, error: FunctionError) -> Self {
        Self {
            function_call_id: function_call_id.into(),
            value: None,
            error: Some(error),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRendererFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_id: Option<String>,
    pub function_call_id: String,
    pub call_function: FunctionCall,
}

impl CallRendererFunction {
    pub fn new(function_call_id: impl Into<String>, call_function: FunctionCall) -> Self {
        Self {
            surface_id: None,
            function_call_id: function_call_id.into(),
            call_function,
        }
    }

    pub fn surface_id(mut self, surface_id: impl Into<String>) -> Self {
        self.surface_id = Some(surface_id.into());
        self
    }

    pub const fn into_message(self) -> AgentToRendererMessage {
        AgentToRendererMessage::CallRendererFunction(CallRendererFunctionMessage {
            version: ProtocolVersion::V1_0,
            call_renderer_function: self,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRendererFunctionMessage {
    pub version: ProtocolVersion,
    pub call_renderer_function: CallRendererFunction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentFunctionResponseMessage {
    pub version: ProtocolVersion,
    pub agent_function_response: FunctionResponse,
}

impl FunctionResponse {
    pub const fn into_agent_message(self) -> AgentToRendererMessage {
        AgentToRendererMessage::AgentFunctionResponse(AgentFunctionResponseMessage {
            version: ProtocolVersion::V1_0,
            agent_function_response: self,
        })
    }

    pub const fn into_renderer_message(self) -> RendererToAgentMessage {
        RendererToAgentMessage::RendererFunctionResponse(RendererFunctionResponseMessage {
            version: ProtocolVersion::V1_0,
            renderer_function_response: self,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentToRendererMessage {
    CreateSurface(CreateSurfaceMessage),
    UpdateComponents(UpdateComponentsMessage),
    UpdateDataModel(UpdateDataModelMessage),
    DeleteSurface(DeleteSurfaceMessage),
    CallRendererFunction(CallRendererFunctionMessage),
    AgentFunctionResponse(AgentFunctionResponseMessage),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEvent {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_message: Option<String>,
    pub surface_id: String,
    pub source_component_id: String,
    pub timestamp: String,
    #[serde(default)]
    pub context: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionMessage {
    pub version: ProtocolVersion,
    pub action: ActionEvent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallAgentFunction {
    pub surface_id: String,
    pub function_call_id: String,
    pub call_function: FunctionCall,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallAgentFunctionMessage {
    pub version: ProtocolVersion,
    pub call_agent_function: CallAgentFunction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RendererFunctionResponseMessage {
    pub version: ProtocolVersion,
    pub renderer_function_response: FunctionResponse,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub version: ProtocolVersion,
    pub error: RendererError,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RendererError {
    Validation(ValidationError),
    Generic(GenericError),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    pub code: ValidationErrorCode,
    pub surface_id: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationErrorCode {
    ValidationFailed,
    UnallowedParent,
    UnallowedChild,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call_id: Option<String>,
    #[serde(flatten)]
    pub additional: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RendererToAgentMessage {
    Action(ActionMessage),
    CallAgentFunction(CallAgentFunctionMessage),
    RendererFunctionResponse(RendererFunctionResponseMessage),
    Error(ErrorMessage),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{DataBinding, Event};

    #[test]
    fn serializes_the_official_inline_create_surface_shape() {
        let message = CreateSurface::new("booking", "travel:flights")
            .components([
                Component::new("root", "Column").property("children", json!(["flight"])),
                Component::new("flight", "Text")
                    .property("text", DataBinding::new("/flights/0/summary")),
            ])
            .data_model(DataModel::new().with(
                "flights",
                json!([{ "summary": "LH458 · BER -> SFO · $680" }]),
            ))
            .into_message();

        assert_eq!(
            serde_json::to_value(message).expect("message must serialize"),
            json!({
                "version": "1.0",
                "createSurface": {
                    "surfaceId": "booking",
                    "catalogId": "travel:flights",
                    "components": [
                        {"id": "root", "component": "Column", "children": ["flight"]},
                        {"id": "flight", "component": "Text", "text": {"@path": "/flights/0/summary"}}
                    ],
                    "dataModel": {"flights": [{"summary": "LH458 · BER -> SFO · $680"}]}
                }
            })
        );
    }

    #[test]
    fn deserializes_the_official_action_shape() {
        let input = json!({
            "version": "1.0",
            "action": {
                "name": "confirm_booking",
                "surfaceId": "flight-booking-surface",
                "sourceComponentId": "submit_btn",
                "timestamp": "2026-10-12T08:30:00Z",
                "context": {"outboundPax": 2, "returnPax": 2}
            }
        });

        let message: RendererToAgentMessage =
            serde_json::from_value(input.clone()).expect("official action must deserialize");
        assert_eq!(
            serde_json::to_value(message).expect("action must serialize"),
            input
        );
    }

    #[test]
    fn event_context_uses_data_bindings() {
        let event = Event::new("confirm_booking").context("price", DataBinding::new("/price"));
        assert_eq!(
            serde_json::to_value(event).expect("event must serialize"),
            json!({"name": "confirm_booking", "context": {"price": {"@path": "/price"}}})
        );
    }
}
