use std::time::Duration;

use crate::support::{custom_catalogs, function_call};
use futures::executor::block_on;
use sdui::function::RendererFunctionCalls;
use sdui::{FunctionError, FunctionResponse, ProtocolVersion, RendererFunctionResponseMessage};
use serde_json::{Value, json};

fn success(id: &str, value: Value) -> RendererFunctionResponseMessage {
    RendererFunctionResponseMessage {
        version: ProtocolVersion::V1_0,
        renderer_function_response: FunctionResponse::success(id, value),
    }
}

fn failure(id: &str, code: &str) -> RendererFunctionResponseMessage {
    RendererFunctionResponseMessage {
        version: ProtocolVersion::V1_0,
        renderer_function_response: FunctionResponse::failure(
            id,
            FunctionError {
                code: code.to_owned(),
                message: "renderer failed".to_owned(),
            },
        ),
    }
}

#[test]
fn renderer_call_has_strict_shape_and_resolves_success() {
    let calls = RendererFunctionCalls::new(custom_catalogs());
    let (message, pending) = calls
        .call(
            "call-1",
            Some("surface".to_owned()),
            function_call("agentTool", json!("input")),
            None,
        )
        .expect("valid renderer call");
    assert_eq!(
        serde_json::to_value(message).expect("message JSON"),
        json!({
            "version": "1.0",
            "callRendererFunction": {
                "surfaceId": "surface",
                "functionCallId": "call-1",
                "callFunction": {
                    "@call": "agentTool",
                    "catalogId": "test:functions",
                    "args": {"value": "input"}
                }
            }
        })
    );

    assert!(calls.resolve(&success("call-1", json!("output"))));
    assert_eq!(block_on(pending).expect("result"), json!("output"));
    assert_eq!(calls.pending_len(), 0);
}

#[test]
fn renderer_error_and_return_type_mismatch_resolve_the_pending_call() {
    let calls = RendererFunctionCalls::new(custom_catalogs());
    let (_, failed) = calls
        .call("failed", None, function_call("agentTool", json!("x")), None)
        .expect("call");
    assert!(calls.resolve(&failure("failed", "DEVICE_ERROR")));
    let error = block_on(failed).expect_err("renderer error");
    assert_eq!(error.code, "DEVICE_ERROR");
    assert_eq!(error.message, "renderer failed");

    let (_, mismatched) = calls
        .call(
            "mismatched",
            None,
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("call");
    assert!(calls.resolve(&success("mismatched", json!(42))));
    assert_eq!(
        block_on(mismatched).expect_err("return mismatch").code,
        "EXECUTION_ERROR"
    );
}

#[test]
fn duplicate_ids_zero_timeouts_and_renderer_only_calls_are_rejected() {
    let calls = RendererFunctionCalls::new(custom_catalogs());
    let (_, pending) = calls
        .call(
            "duplicate",
            None,
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("first call");
    let duplicate = calls.call(
        "duplicate",
        None,
        function_call("agentTool", json!("x")),
        None,
    );
    assert_eq!(duplicate.expect_err("duplicate ID").code, "DUPLICATE");
    assert_eq!(calls.pending_len(), 1);

    let zero = calls.call(
        "zero",
        None,
        function_call("agentTool", json!("x")),
        Some(Duration::ZERO),
    );
    assert_eq!(
        zero.expect_err("zero timeout").code,
        "INVALID_FUNCTION_CALL"
    );

    let renderer_only = calls.call("local", None, function_call("localOnly", json!("x")), None);
    assert_eq!(
        renderer_only.expect_err("rendererOnly").code,
        "INVALID_FUNCTION_CALL"
    );
    assert!(calls.cancel("duplicate"));
    assert_eq!(block_on(pending).expect_err("cancelled").code, "CANCELLED");
}

#[test]
fn timeout_and_explicit_cancellation_use_standard_codes() {
    let calls = RendererFunctionCalls::new(custom_catalogs());
    let (_, timed_out) = calls
        .call(
            "timeout",
            None,
            function_call("agentTool", json!("x")),
            Some(Duration::from_millis(10)),
        )
        .expect("call");
    assert_eq!(block_on(timed_out).expect_err("timeout").code, "TIMEOUT");
    assert_eq!(calls.pending_len(), 0);

    let (_, cancelled) = calls
        .call("cancel", None, function_call("agentTool", json!("x")), None)
        .expect("call");
    assert!(calls.cancel("cancel"));
    assert!(!calls.cancel("cancel"));
    assert_eq!(
        block_on(cancelled).expect_err("cancelled").code,
        "CANCELLED"
    );
}

#[test]
fn surface_cancellation_only_cancels_matching_calls() {
    let calls = RendererFunctionCalls::new(custom_catalogs());
    let (_, first) = calls
        .call(
            "first",
            Some("one".to_owned()),
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("call");
    let (_, second) = calls
        .call(
            "second",
            Some("one".to_owned()),
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("call");
    let (_, other) = calls
        .call(
            "other",
            Some("two".to_owned()),
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("call");

    assert_eq!(calls.cancel_surface("one"), 2);
    assert_eq!(block_on(first).expect_err("cancelled").code, "CANCELLED");
    assert_eq!(block_on(second).expect_err("cancelled").code, "CANCELLED");
    assert_eq!(calls.pending_len(), 1);
    assert!(calls.resolve(&success("other", json!("ok"))));
    assert_eq!(block_on(other).expect("other result"), json!("ok"));
}

#[test]
fn dispose_and_drop_fail_pending_calls_and_pending_drop_cleans_up() {
    let calls = RendererFunctionCalls::new(custom_catalogs());
    let (_, disposed) = calls
        .call(
            "disposed",
            None,
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("call");
    calls.dispose();
    assert!(calls.is_disposed());
    assert_eq!(block_on(disposed).expect_err("disposed").code, "DISPOSED");
    let after_dispose = calls.call("after", None, function_call("agentTool", json!("x")), None);
    assert_eq!(after_dispose.expect_err("disposed owner").code, "DISPOSED");

    let dropped_owner = {
        let calls = RendererFunctionCalls::new(custom_catalogs());
        let (_, pending) = calls
            .call(
                "drop-owner",
                None,
                function_call("agentTool", json!("x")),
                None,
            )
            .expect("call");
        pending
    };
    assert_eq!(
        block_on(dropped_owner).expect_err("dropped owner").code,
        "DISPOSED"
    );

    let calls = RendererFunctionCalls::new(custom_catalogs());
    let (_, pending) = calls
        .call(
            "drop-pending",
            None,
            function_call("agentTool", json!("x")),
            None,
        )
        .expect("call");
    drop(pending);
    assert_eq!(calls.pending_len(), 0);
    assert!(!calls.resolve(&success("drop-pending", json!("late"))));
}

#[test]
fn unknown_renderer_response_is_not_handled() {
    let calls = RendererFunctionCalls::new(custom_catalogs());
    assert!(!calls.resolve(&success("unknown", json!(null))));
}
