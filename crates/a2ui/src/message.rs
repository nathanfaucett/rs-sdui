#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::derive_partial_eq_without_eq,
    reason = "fluent authoring methods return new values"
)]

use std::{error::Error, fmt};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de::Error as _, ser::SerializeStruct,
};
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunctionError {
    pub code: String,
    pub message: String,
}

impl FunctionError {
    pub const INVALID_FUNCTION_CALL: &'static str = "INVALID_FUNCTION_CALL";

    fn invalid_function_call() -> Self {
        Self {
            code: Self::INVALID_FUNCTION_CALL.to_owned(),
            message: "callFunction.catalogId is required".to_owned(),
        }
    }
}

impl fmt::Display for FunctionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for FunctionError {}

fn require_catalog_id(call_function: FunctionCall) -> Result<FunctionCall, FunctionError> {
    if call_function.catalog_id.is_none() {
        Err(FunctionError::invalid_function_call())
    } else {
        Ok(call_function)
    }
}

fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionResult {
    Value(Value),
    Error(FunctionError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionResponse {
    function_call_id: String,
    result: FunctionResult,
}

impl FunctionResponse {
    pub fn success(function_call_id: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            function_call_id: function_call_id.into(),
            result: FunctionResult::Value(value.into()),
        }
    }

    pub fn failure(function_call_id: impl Into<String>, error: FunctionError) -> Self {
        Self {
            function_call_id: function_call_id.into(),
            result: FunctionResult::Error(error),
        }
    }

    pub fn function_call_id(&self) -> &str {
        &self.function_call_id
    }

    pub const fn result(&self) -> &FunctionResult {
        &self.result
    }
}

impl Serialize for FunctionResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FunctionResponse", 2)?;
        state.serialize_field("functionCallId", &self.function_call_id)?;
        match &self.result {
            FunctionResult::Value(value) => state.serialize_field("value", value)?,
            FunctionResult::Error(error) => state.serialize_field("error", error)?,
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for FunctionResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Wire {
            function_call_id: String,
            #[serde(default, deserialize_with = "deserialize_present")]
            value: Option<Value>,
            #[serde(default, deserialize_with = "deserialize_present")]
            error: Option<FunctionError>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let result = match (wire.value, wire.error) {
            (Some(value), None) => FunctionResult::Value(value),
            (None, Some(error)) => FunctionResult::Error(error),
            _ => {
                return Err(D::Error::custom(
                    "FunctionResponse requires exactly one of value or error",
                ));
            }
        };
        Ok(Self {
            function_call_id: wire.function_call_id,
            result,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRendererFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    surface_id: Option<String>,
    function_call_id: String,
    call_function: FunctionCall,
}

impl CallRendererFunction {
    /// Creates an agent-to-renderer call with a required resolved catalog ID.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_FUNCTION_CALL` when `call_function.catalog_id` is absent.
    pub fn new(
        function_call_id: impl Into<String>,
        call_function: FunctionCall,
    ) -> Result<Self, FunctionError> {
        Ok(Self {
            surface_id: None,
            function_call_id: function_call_id.into(),
            call_function: require_catalog_id(call_function)?,
        })
    }

    pub fn surface_id_ref(&self) -> Option<&str> {
        self.surface_id.as_deref()
    }

    pub fn function_call_id(&self) -> &str {
        &self.function_call_id
    }

    pub const fn call_function(&self) -> &FunctionCall {
        &self.call_function
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

impl<'de> Deserialize<'de> for CallRendererFunction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Wire {
            surface_id: Option<String>,
            function_call_id: String,
            call_function: FunctionCall,
        }

        let wire = Wire::deserialize(deserializer)?;
        let mut call =
            Self::new(wire.function_call_id, wire.call_function).map_err(D::Error::custom)?;
        call.surface_id = wire.surface_id;
        Ok(call)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CallRendererFunctionMessage {
    pub version: ProtocolVersion,
    pub call_renderer_function: CallRendererFunction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallAgentFunction {
    surface_id: String,
    function_call_id: String,
    call_function: FunctionCall,
}

impl CallAgentFunction {
    /// Creates a renderer-to-agent call with a required resolved catalog ID.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_FUNCTION_CALL` when `call_function.catalog_id` is absent.
    pub fn new(
        surface_id: impl Into<String>,
        function_call_id: impl Into<String>,
        call_function: FunctionCall,
    ) -> Result<Self, FunctionError> {
        Ok(Self {
            surface_id: surface_id.into(),
            function_call_id: function_call_id.into(),
            call_function: require_catalog_id(call_function)?,
        })
    }

    pub fn surface_id(&self) -> &str {
        &self.surface_id
    }

    pub fn function_call_id(&self) -> &str {
        &self.function_call_id
    }

    pub const fn call_function(&self) -> &FunctionCall {
        &self.call_function
    }
}

impl<'de> Deserialize<'de> for CallAgentFunction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Wire {
            surface_id: String,
            function_call_id: String,
            call_function: FunctionCall,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.surface_id, wire.function_call_id, wire.call_function)
            .map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CallAgentFunctionMessage {
    pub version: ProtocolVersion,
    pub call_agent_function: CallAgentFunction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

    #[test]
    fn function_response_success_has_exact_json_and_round_trips() {
        let expected = json!({
            "functionCallId": "call-1",
            "value": {"approved": true}
        });
        let response = FunctionResponse::success("call-1", json!({"approved": true}));

        assert_eq!(response.function_call_id(), "call-1");
        assert_eq!(
            response.result(),
            &FunctionResult::Value(json!({"approved": true}))
        );
        assert_eq!(
            serde_json::to_value(&response).expect("response must serialize"),
            expected
        );
        let decoded: FunctionResponse =
            serde_json::from_value(expected.clone()).expect("response must deserialize");
        assert_eq!(decoded, response);
        assert_eq!(
            serde_json::to_value(response.into_agent_message()).expect("message must serialize"),
            json!({"version": "1.0", "agentFunctionResponse": expected})
        );
    }

    #[test]
    fn function_response_error_has_exact_json_and_round_trips() {
        let expected = json!({
            "functionCallId": "call-2",
            "error": {"code": "NOT_FOUND", "message": "No booking"}
        });
        let error = FunctionError {
            code: "NOT_FOUND".to_owned(),
            message: "No booking".to_owned(),
        };
        let response = FunctionResponse::failure("call-2", error.clone());

        assert_eq!(response.result(), &FunctionResult::Error(error));
        assert_eq!(
            serde_json::to_value(&response).expect("response must serialize"),
            expected
        );
        let decoded: FunctionResponse =
            serde_json::from_value(expected.clone()).expect("response must deserialize");
        assert_eq!(decoded, response);
        assert_eq!(
            serde_json::to_value(response.into_renderer_message()).expect("message must serialize"),
            json!({"version": "1.0", "rendererFunctionResponse": expected})
        );
    }

    #[test]
    fn function_response_rejects_missing_id_both_neither_and_unknown_fields() {
        let invalid = [
            json!({"value": 1}),
            json!({"functionCallId": "call-1"}),
            json!({
                "functionCallId": "call-1",
                "value": 1,
                "error": {"code": "FAILED", "message": "Failed"}
            }),
            json!({"functionCallId": "call-1", "value": 1, "unknown": true}),
            json!({
                "functionCallId": "call-1",
                "error": {"code": "FAILED", "message": "Failed", "unknown": true}
            }),
        ];

        for input in invalid {
            assert!(
                serde_json::from_value::<FunctionResponse>(input.clone()).is_err(),
                "input must be rejected: {input}"
            );
        }
    }

    #[test]
    fn function_response_accepts_null_as_its_value() {
        let expected = json!({"functionCallId": "call-1", "value": null});
        let response: FunctionResponse =
            serde_json::from_value(expected.clone()).expect("null value must deserialize");

        assert_eq!(response.result(), &FunctionResult::Value(Value::Null));
        assert_eq!(
            serde_json::to_value(response).expect("response must serialize"),
            expected
        );
    }

    #[test]
    fn renderer_function_requires_a_catalog_id() {
        let error = CallRendererFunction::new("call-1", FunctionCall::new("render"))
            .expect_err("call without a catalog must fail");
        assert_eq!(error.code, FunctionError::INVALID_FUNCTION_CALL);
        assert_eq!(
            error.to_string(),
            "INVALID_FUNCTION_CALL: callFunction.catalogId is required"
        );

        let input = json!({
            "functionCallId": "call-1",
            "callFunction": {"@call": "render"}
        });
        let _ = serde_json::from_value::<CallRendererFunction>(input)
            .expect_err("a missing catalog ID must fail");
    }

    #[test]
    fn renderer_function_has_exact_json_and_round_trips() {
        let expected = json!({
            "surfaceId": "booking",
            "functionCallId": "call-1",
            "callFunction": {
                "@call": "render",
                "catalogId": "travel:ui",
                "args": {"visible": true}
            }
        });
        let call = CallRendererFunction::new(
            "call-1",
            FunctionCall::new("render")
                .catalog_id("travel:ui")
                .arg("visible", true),
        )
        .expect("valid call must construct")
        .surface_id("booking");

        assert_eq!(call.surface_id_ref(), Some("booking"));
        assert_eq!(call.function_call_id(), "call-1");
        assert_eq!(
            call.call_function().catalog_id.as_deref(),
            Some("travel:ui")
        );
        assert_eq!(
            serde_json::to_value(&call).expect("call must serialize"),
            expected
        );
        let decoded: CallRendererFunction =
            serde_json::from_value(expected.clone()).expect("call must deserialize");
        assert_eq!(decoded, call);
        assert_eq!(
            serde_json::to_value(call.into_message()).expect("message must serialize"),
            json!({"version": "1.0", "callRendererFunction": expected})
        );
    }

    #[test]
    fn renderer_function_rejects_unknown_fields() {
        let input = json!({
            "functionCallId": "call-1",
            "callFunction": {"@call": "render", "catalogId": "travel:ui"},
            "unknown": true
        });
        let _ = serde_json::from_value::<CallRendererFunction>(input)
            .expect_err("unknown renderer-call fields must fail");
    }

    #[test]
    fn agent_function_requires_a_catalog_id() {
        let error = CallAgentFunction::new("booking", "call-1", FunctionCall::new("submit"))
            .expect_err("call without a catalog must fail");
        assert_eq!(error.code, FunctionError::INVALID_FUNCTION_CALL);

        let input = json!({
            "surfaceId": "booking",
            "functionCallId": "call-1",
            "callFunction": {"@call": "submit"}
        });
        let _ = serde_json::from_value::<CallAgentFunction>(input)
            .expect_err("a missing catalog ID must fail");
    }

    #[test]
    fn agent_function_has_exact_json_and_round_trips() {
        let expected = json!({
            "surfaceId": "booking",
            "functionCallId": "call-1",
            "callFunction": {
                "@call": "submit",
                "catalogId": "travel:actions",
                "args": {"bookingId": "b-1"}
            }
        });
        let call = CallAgentFunction::new(
            "booking",
            "call-1",
            FunctionCall::new("submit")
                .catalog_id("travel:actions")
                .arg("bookingId", "b-1"),
        )
        .expect("valid call must construct");

        assert_eq!(call.surface_id(), "booking");
        assert_eq!(call.function_call_id(), "call-1");
        assert_eq!(
            call.call_function().catalog_id.as_deref(),
            Some("travel:actions")
        );
        assert_eq!(
            serde_json::to_value(&call).expect("call must serialize"),
            expected
        );
        let decoded: CallAgentFunction =
            serde_json::from_value(expected.clone()).expect("call must deserialize");
        assert_eq!(decoded, call);
        let message = RendererToAgentMessage::CallAgentFunction(CallAgentFunctionMessage {
            version: ProtocolVersion::V1_0,
            call_agent_function: decoded,
        });
        assert_eq!(
            serde_json::to_value(message).expect("message must serialize"),
            json!({"version": "1.0", "callAgentFunction": expected})
        );
    }

    #[test]
    fn agent_function_rejects_unknown_fields() {
        let input = json!({
            "surfaceId": "booking",
            "functionCallId": "call-1",
            "callFunction": {"@call": "submit", "catalogId": "travel:actions"},
            "unknown": true
        });
        let _ = serde_json::from_value::<CallAgentFunction>(input)
            .expect_err("unknown agent-call fields must fail");
    }

    #[test]
    fn function_envelopes_reject_unknown_fields() {
        let agent_messages = [
            json!({
                "version": "1.0",
                "agentFunctionResponse": {"functionCallId": "call-1", "value": null},
                "unknown": true
            }),
            json!({
                "version": "1.0",
                "callRendererFunction": {
                    "functionCallId": "call-1",
                    "callFunction": {"@call": "render", "catalogId": "test:functions"}
                },
                "unknown": true
            }),
        ];
        for input in agent_messages {
            let _ = serde_json::from_value::<AgentToRendererMessage>(input)
                .expect_err("unknown agent-envelope fields must fail");
        }

        let renderer_messages = [
            json!({
                "version": "1.0",
                "rendererFunctionResponse": {"functionCallId": "call-1", "value": null},
                "unknown": true
            }),
            json!({
                "version": "1.0",
                "callAgentFunction": {
                    "surfaceId": "main",
                    "functionCallId": "call-1",
                    "callFunction": {"@call": "submit", "catalogId": "test:functions"}
                },
                "unknown": true
            }),
        ];
        for input in renderer_messages {
            let _ = serde_json::from_value::<RendererToAgentMessage>(input)
                .expect_err("unknown renderer-envelope fields must fail");
        }
    }
}
