use a2ui::{
    ActionEvent, ActionMessage, CallAgentFunction, CallAgentFunctionMessage, CallRendererFunction,
    Component, CreateSurface, DeleteSurface, ErrorMessage, FunctionCall, FunctionResponse,
    GenericError, ProtocolVersion, RendererError, RendererToAgentMessage, UpdateComponents,
    UpdateDataModel,
};
use a2ui_a2a::{
    agent_data_part, decode_agent_messages, decode_renderer_messages, renderer_data_part,
};

#[test]
fn every_a2ui_message_variant_round_trips_through_data_parts() {
    let agent_messages = vec![
        CreateSurface::new("main", "catalog").into_message(),
        UpdateComponents::new("main", [Component::new("root", "Text")]).into_message(),
        UpdateDataModel::new("main", "/count", 1).into_message(),
        DeleteSurface::new("main").into_message(),
        CallRendererFunction::new(
            "call-renderer",
            FunctionCall::new("shared").catalog_id("catalog"),
        )
        .unwrap()
        .surface_id("main")
        .into_message(),
        FunctionResponse::success("call-agent", true).into_agent_message(),
    ];
    let agent_part = agent_data_part(&agent_messages).unwrap();
    let decoded_agent = decode_agent_messages(&agent_part).unwrap();
    assert!(decoded_agent.errors.is_empty());
    assert_eq!(decoded_agent.messages, agent_messages);

    let renderer_messages = vec![
        RendererToAgentMessage::Action(ActionMessage {
            version: ProtocolVersion::V1_0,
            action: ActionEvent {
                name: "submit".into(),
                user_message: None,
                surface_id: "main".into(),
                source_component_id: "root".into(),
                timestamp: "2026-01-01T00:00:00Z".into(),
                context: serde_json::Map::new(),
                metadata: None,
            },
        }),
        RendererToAgentMessage::CallAgentFunction(CallAgentFunctionMessage {
            version: ProtocolVersion::V1_0,
            call_agent_function: CallAgentFunction::new(
                "main",
                "call-agent",
                FunctionCall::new("shared").catalog_id("catalog"),
            )
            .unwrap(),
        }),
        FunctionResponse::failure(
            "call-renderer",
            a2ui::FunctionError {
                code: "EXECUTION_ERROR".into(),
                message: "failed".into(),
            },
        )
        .into_renderer_message(),
        RendererToAgentMessage::Error(ErrorMessage {
            version: ProtocolVersion::V1_0,
            error: RendererError::Generic(GenericError {
                code: "INTEGRITY_ERROR".into(),
                message: "invalid surface".into(),
                surface_id: Some("main".into()),
                function_call_id: None,
                additional: serde_json::Map::new(),
            }),
        }),
    ];
    let renderer_part = renderer_data_part(&renderer_messages).unwrap();
    let decoded_renderer = decode_renderer_messages(&renderer_part).unwrap();
    assert!(decoded_renderer.errors.is_empty());
    assert_eq!(decoded_renderer.messages, renderer_messages);
}
