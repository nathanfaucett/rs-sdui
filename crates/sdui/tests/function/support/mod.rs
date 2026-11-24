#![allow(
    dead_code,
    reason = "each integration test uses a subset of shared helpers"
)]

use sdui::function::CatalogSet;
use sdui::{CallAgentFunctionMessage, FunctionCall};

pub const BASIC_CATALOG: &str = include_str!("../../../catalog.json");
pub const CUSTOM_CATALOG: &str = include_str!("../test_catalog.json");
pub const BASIC_CATALOG_ID: &str =
    "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json";
pub const CUSTOM_CATALOG_ID: &str = "test:functions";

pub fn basic_catalogs() -> CatalogSet {
    CatalogSet::from_json(BASIC_CATALOG).expect("basic catalog")
}

pub fn custom_catalogs() -> CatalogSet {
    CatalogSet::from_json(CUSTOM_CATALOG).expect("custom catalog")
}

pub fn function_call(name: &str, value: serde_json::Value) -> FunctionCall {
    FunctionCall::new(name)
        .catalog_id(CUSTOM_CATALOG_ID)
        .arg("value", value)
}

pub fn agent_call(name: &str, id: &str) -> CallAgentFunctionMessage {
    let call = sdui::CallAgentFunction::new(
        "surface",
        id,
        function_call(name, serde_json::Value::String("hello".to_owned())),
    )
    .expect("explicit catalog ID");
    CallAgentFunctionMessage {
        version: sdui::ProtocolVersion::V1_0,
        call_agent_function: call,
    }
}
