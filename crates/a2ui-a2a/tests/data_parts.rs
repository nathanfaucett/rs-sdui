use std::slice;

use a2a::{Part, PartContent};
use a2ui::{DeleteSurface, FunctionResponse};
use a2ui_a2a::{
    agent_data_part, decode_agent_messages, decode_renderer_messages, renderer_data_part,
};
use serde_json::{Value, json};

#[test]
fn data_parts_use_official_encoding_and_isolate_invalid_entries() {
    let first = DeleteSurface::new("first").into_message();
    let second = DeleteSurface::new("second").into_message();
    let part = agent_data_part(&[first.clone(), second.clone()]).unwrap();

    assert_eq!(
        serde_json::to_value(&part).unwrap(),
        json!({
            "data": [
                {"version": "1.0", "deleteSurface": {"surfaceId": "first"}},
                {"version": "1.0", "deleteSurface": {"surfaceId": "second"}}
            ],
            "metadata": {"mimeType": "application/a2ui+json"}
        })
    );

    let mut with_invalid = Part {
        content: PartContent::Data(Value::Array(vec![
            serde_json::to_value(&first).unwrap(),
            json!({"version": "1.0", "notAnA2uiMessage": {}}),
            serde_json::to_value(&second).unwrap(),
        ])),
        filename: None,
        media_type: None,
        metadata: part.metadata,
    };
    let decoded = decode_agent_messages(&with_invalid).unwrap();
    assert_eq!(decoded.messages, vec![first, second]);
    assert_eq!(decoded.errors.len(), 1);
    assert_eq!(decoded.errors[0].index, 1);

    with_invalid.metadata = None;
    let _error = decode_agent_messages(&with_invalid).unwrap_err();
}

#[test]
fn renderer_data_parts_preserve_function_rpc() {
    let response = FunctionResponse::success("call-1", json!({"ok": true})).into_renderer_message();
    let part = renderer_data_part(slice::from_ref(&response)).unwrap();
    let decoded = decode_renderer_messages(&part).unwrap();

    assert!(decoded.errors.is_empty());
    assert_eq!(decoded.messages, vec![response]);
}
