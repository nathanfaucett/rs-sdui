use crate::support::{BASIC_CATALOG_ID, basic_catalogs, custom_catalogs};
use futures::executor::block_on;
use sdui::function::AgentFunctions;
use sdui::{CallAgentFunction, FunctionCall, FunctionError, FunctionResult};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct FormatArgs {
    value: String,
}

fn agent_message(
    catalog_id: &str,
    name: &str,
    args: serde_json::Map<String, Value>,
) -> sdui::CallAgentFunctionMessage {
    let call = FunctionCall {
        name: name.to_owned(),
        catalog_id: Some(catalog_id.to_owned()),
        args,
    };
    sdui::CallAgentFunctionMessage {
        version: sdui::ProtocolVersion::V1_0,
        call_agent_function: CallAgentFunction::new("surface", "call-1", call)
            .expect("explicit catalog ID"),
    }
}

#[test]
fn typed_agent_function_round_trip_uses_the_basic_catalog() {
    let mut functions = AgentFunctions::new(basic_catalogs());
    functions
        .register(
            BASIC_CATALOG_ID,
            "formatString",
            |args: FormatArgs, context| async move {
                assert_eq!(context.surface_id, "surface");
                assert_eq!(context.function_call_id, "call-1");
                assert_eq!(context.catalog_id, BASIC_CATALOG_ID);
                assert_eq!(context.name, "formatString");
                Ok::<_, FunctionError>(args.value.replace("${name}", "Ada"))
            },
        )
        .expect("registration");
    let message = agent_message(
        BASIC_CATALOG_ID,
        "formatString",
        serde_json::Map::from_iter([("value".to_owned(), json!("Hello ${name}"))]),
    );

    let response = block_on(functions.handle(message));
    assert_eq!(
        serde_json::to_value(response).expect("response JSON"),
        json!({
            "version": "1.0",
            "agentFunctionResponse": {
                "functionCallId": "call-1",
                "value": "Hello Ada"
            }
        })
    );
}

#[test]
fn invalid_arguments_unknown_functions_and_handler_type_mismatches_are_standard_errors() {
    let mut functions = AgentFunctions::new(basic_catalogs());
    functions
        .register(
            BASIC_CATALOG_ID,
            "formatString",
            |args: FormatArgs, _| async move { Ok::<_, FunctionError>(args.value) },
        )
        .expect("registration");

    let invalid = block_on(functions.handle(agent_message(
        BASIC_CATALOG_ID,
        "formatString",
        serde_json::Map::from_iter([("value".to_owned(), json!({"secret": "sensitive-argument"}))]),
    )));
    let FunctionResult::Error(error) = invalid.agent_function_response.result() else {
        panic!("invalid arguments must fail");
    };
    assert_eq!(error.code, "INVALID_FUNCTION_CALL");
    assert!(!error.message.contains("sensitive-argument"));

    let unknown = block_on(functions.handle(agent_message(
        BASIC_CATALOG_ID,
        "missingFunction",
        serde_json::Map::new(),
    )));
    let FunctionResult::Error(error) = unknown.agent_function_response.result() else {
        panic!("unknown function must fail");
    };
    assert_eq!(error.code, "UNKNOWN_FUNCTION");
}

#[test]
fn duplicate_and_unknown_registration_fail() {
    let mut functions = AgentFunctions::new(basic_catalogs());
    functions
        .register(
            BASIC_CATALOG_ID,
            "formatString",
            |args: FormatArgs, _| async move { Ok::<_, FunctionError>(args.value) },
        )
        .expect("first registration");
    let duplicate = functions.register(
        BASIC_CATALOG_ID,
        "formatString",
        |args: FormatArgs, _| async move { Ok::<_, FunctionError>(args.value) },
    );
    assert_eq!(
        duplicate.expect_err("duplicate must fail").code,
        "DUPLICATE"
    );
    let unknown = functions.register(
        BASIC_CATALOG_ID,
        "missingFunction",
        |args: FormatArgs, _| async move { Ok::<_, FunctionError>(args.value) },
    );
    assert_eq!(
        unknown.expect_err("unknown must fail").code,
        "UNKNOWN_FUNCTION"
    );
}

#[test]
fn return_type_mismatch_is_an_execution_error() {
    let mut functions = AgentFunctions::new(custom_catalogs());
    functions
        .register("test:functions", "add", |_: Value, _| async move {
            Ok::<_, FunctionError>("not a number")
        })
        .expect("registration");
    let response = block_on(functions.handle(agent_message(
        "test:functions",
        "add",
        serde_json::Map::from_iter([
            ("left".to_owned(), json!(1)),
            ("right".to_owned(), json!(2)),
        ]),
    )));
    let FunctionResult::Error(error) = response.agent_function_response.result() else {
        panic!("mismatched return must fail");
    };
    assert_eq!(error.code, "EXECUTION_ERROR");
    assert_eq!(
        serde_json::to_value(&response.agent_function_response).expect("error JSON"),
        json!({
            "functionCallId": "call-1",
            "error": {
                "code": "EXECUTION_ERROR",
                "message": "Function 'add' returned a value that does not match its declared returnType"
            }
        })
    );
}

#[test]
fn renderer_originated_agent_call_rejects_agent_only_declarations() {
    let functions = AgentFunctions::new(custom_catalogs());
    let response = block_on(functions.handle(agent_message(
        "test:functions",
        "agentTool",
        serde_json::Map::from_iter([("value".to_owned(), json!("x"))]),
    )));
    let FunctionResult::Error(error) = response.agent_function_response.result() else {
        panic!("agentOnly UI invocation must fail");
    };
    assert_eq!(error.code, "INVALID_FUNCTION_CALL");
}
