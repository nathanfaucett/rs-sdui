use crate::support::{agent_call, custom_catalogs, function_call};
use futures::executor::block_on;
use sdui::function::{Dispatch, RendererMessageProcessor};
use sdui::{
    AgentToRendererMessage, FunctionError, FunctionResponse, ProtocolVersion,
    RendererFunctionResponseMessage, RendererToAgentMessage,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Args {
    value: String,
}

#[test]
fn processor_executes_agent_functions_and_correlates_renderer_responses() {
    let mut processor = RendererMessageProcessor::new(custom_catalogs());
    processor
        .agent_functions_mut()
        .register("test:functions", "shared", |args: Args, _| async move {
            Ok::<_, FunctionError>(args.value.to_uppercase())
        })
        .expect("registration");

    let dispatched = block_on(processor.process(RendererToAgentMessage::CallAgentFunction(
        agent_call("shared", "agent-call"),
    )));
    let Dispatch::Handled(messages) = dispatched else {
        panic!("agent call must be handled");
    };
    assert_eq!(messages.len(), 1);
    assert_eq!(
        serde_json::to_value(&messages[0]).expect("response JSON"),
        json!({
            "version": "1.0",
            "agentFunctionResponse": {
                "functionCallId": "agent-call",
                "value": "HELLO"
            }
        })
    );

    let (_, pending) = processor
        .renderer_function_calls()
        .call(
            "renderer-call",
            None,
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("renderer call");
    let response = RendererFunctionResponseMessage {
        version: ProtocolVersion::V1_0,
        renderer_function_response: FunctionResponse::success("renderer-call", "done"),
    };
    assert_eq!(
        block_on(processor.process(RendererToAgentMessage::RendererFunctionResponse(response))),
        Dispatch::Handled(Vec::<AgentToRendererMessage>::new())
    );
    assert_eq!(block_on(pending).expect("renderer value"), json!("done"));
}

#[test]
fn processor_leaves_unknown_responses_unhandled() {
    let processor = RendererMessageProcessor::new(custom_catalogs());
    let message =
        RendererToAgentMessage::RendererFunctionResponse(RendererFunctionResponseMessage {
            version: ProtocolVersion::V1_0,
            renderer_function_response: FunctionResponse::success("unknown", Value::Null),
        });
    assert_eq!(
        block_on(processor.process(message.clone())),
        Dispatch::Unhandled(message)
    );
}

#[test]
fn processor_forwards_actions_and_errors_unchanged() {
    let processor = RendererMessageProcessor::new(custom_catalogs());
    let action: RendererToAgentMessage = serde_json::from_value(json!({
        "version": "1.0",
        "action": {
            "name": "save",
            "surfaceId": "surface",
            "sourceComponentId": "button",
            "timestamp": "2026-11-23T00:00:00Z",
            "context": {"id": 42}
        }
    }))
    .expect("action");
    assert_eq!(
        block_on(processor.process(action.clone())),
        Dispatch::Unhandled(action)
    );

    let error: RendererToAgentMessage = serde_json::from_value(json!({
        "version": "1.0",
        "error": {
            "code": "INTEGRITY_ERROR",
            "message": "surface error",
            "surfaceId": "surface"
        }
    }))
    .expect("error");
    assert_eq!(
        block_on(processor.process(error.clone())),
        Dispatch::Unhandled(error)
    );
}
