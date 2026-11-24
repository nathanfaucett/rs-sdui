#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::derive_partial_eq_without_eq,
    reason = "fluent authoring methods return new values"
)]

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

/// An RFC 6901 data-model reference. Relative paths are valid inside list templates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataBinding {
    #[serde(rename = "@path")]
    pub path: String,
}

impl DataBinding {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl From<DataBinding> for Value {
    fn from(value: DataBinding) -> Self {
        to_value(value)
    }
}

/// A standard A2UI function call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunctionCall {
    #[serde(rename = "@call")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_id: Option<String>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub args: Map<String, Value>,
}

impl FunctionCall {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            catalog_id: None,
            args: Map::new(),
        }
    }

    pub fn catalog_id(mut self, catalog_id: impl Into<String>) -> Self {
        self.catalog_id = Some(catalog_id.into());
        self
    }

    pub fn arg(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.args.insert(name.into(), value.into());
        self
    }
}

impl From<FunctionCall> for Value {
    fn from(value: FunctionCall) -> Self {
        to_value(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicString {
    Literal(String),
    Binding(DataBinding),
    Call(FunctionCall),
}

impl From<String> for DynamicString {
    fn from(value: String) -> Self {
        Self::Literal(value)
    }
}

impl From<&str> for DynamicString {
    fn from(value: &str) -> Self {
        Self::Literal(value.to_owned())
    }
}

impl From<DataBinding> for DynamicString {
    fn from(value: DataBinding) -> Self {
        Self::Binding(value)
    }
}

impl From<FunctionCall> for DynamicString {
    fn from(value: FunctionCall) -> Self {
        Self::Call(value)
    }
}

impl From<DynamicString> for Value {
    fn from(value: DynamicString) -> Self {
        to_value(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicNumber {
    Literal(Number),
    Binding(DataBinding),
    Call(FunctionCall),
}

/// Creates a JSON number. A2UI uses JSON, so non-finite numbers are invalid.
///
/// # Panics
///
/// Panics when `value` is NaN or infinite.
pub fn number(value: f64) -> Number {
    Number::from_f64(value).expect("A2UI numbers must be finite")
}

macro_rules! integer_number {
    ($($type:ty),* $(,)?) => {
        $(
            impl From<$type> for DynamicNumber {
                fn from(value: $type) -> Self {
                    Self::Literal(Number::from(value))
                }
            }
        )*
    };
}

integer_number!(i8, i16, i32, i64, u8, u16, u32, u64);

impl From<f64> for DynamicNumber {
    fn from(value: f64) -> Self {
        Self::Literal(number(value))
    }
}

impl From<f32> for DynamicNumber {
    fn from(value: f32) -> Self {
        Self::Literal(number(f64::from(value)))
    }
}

impl From<DataBinding> for DynamicNumber {
    fn from(value: DataBinding) -> Self {
        Self::Binding(value)
    }
}

impl From<FunctionCall> for DynamicNumber {
    fn from(value: FunctionCall) -> Self {
        Self::Call(value)
    }
}

impl From<DynamicNumber> for Value {
    fn from(value: DynamicNumber) -> Self {
        to_value(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicBoolean {
    Literal(bool),
    Binding(DataBinding),
    Call(FunctionCall),
}

impl From<bool> for DynamicBoolean {
    fn from(value: bool) -> Self {
        Self::Literal(value)
    }
}

impl From<DataBinding> for DynamicBoolean {
    fn from(value: DataBinding) -> Self {
        Self::Binding(value)
    }
}

impl From<FunctionCall> for DynamicBoolean {
    fn from(value: FunctionCall) -> Self {
        Self::Call(value)
    }
}

impl From<DynamicBoolean> for Value {
    fn from(value: DynamicBoolean) -> Self {
        to_value(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicStringList {
    Literal(Vec<String>),
    Binding(DataBinding),
    Call(FunctionCall),
}

impl<T: Into<String>> From<Vec<T>> for DynamicStringList {
    fn from(value: Vec<T>) -> Self {
        Self::Literal(value.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<String>, const N: usize> From<[T; N]> for DynamicStringList {
    fn from(value: [T; N]) -> Self {
        Self::Literal(value.into_iter().map(Into::into).collect())
    }
}

impl From<DataBinding> for DynamicStringList {
    fn from(value: DataBinding) -> Self {
        Self::Binding(value)
    }
}

impl From<FunctionCall> for DynamicStringList {
    fn from(value: FunctionCall) -> Self {
        Self::Call(value)
    }
}

impl From<DynamicStringList> for Value {
    fn from(value: DynamicStringList) -> Self {
        to_value(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicValue {
    Binding(DataBinding),
    Call(FunctionCall),
    Literal(Value),
}

impl DynamicValue {
    pub fn literal(value: impl Into<Value>) -> Self {
        Self::Literal(value.into())
    }
}

impl From<Value> for DynamicValue {
    fn from(value: Value) -> Self {
        Self::Literal(value)
    }
}

impl From<String> for DynamicValue {
    fn from(value: String) -> Self {
        Self::Literal(Value::String(value))
    }
}

impl From<&str> for DynamicValue {
    fn from(value: &str) -> Self {
        Self::Literal(Value::String(value.to_owned()))
    }
}

impl From<bool> for DynamicValue {
    fn from(value: bool) -> Self {
        Self::Literal(Value::Bool(value))
    }
}

impl From<DataBinding> for DynamicValue {
    fn from(value: DataBinding) -> Self {
        Self::Binding(value)
    }
}

impl From<FunctionCall> for DynamicValue {
    fn from(value: FunctionCall) -> Self {
        Self::Call(value)
    }
}

impl From<DynamicValue> for Value {
    fn from(value: DynamicValue) -> Self {
        to_value(value)
    }
}

/// An agent-bound event used by a component action.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_message: Option<DynamicString>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub context: Map<String, Value>,
}

impl Event {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            user_message: None,
            context: Map::new(),
        }
    }

    pub fn user_message(mut self, message: impl Into<DynamicString>) -> Self {
        self.user_message = Some(message.into());
        self
    }

    pub fn context(mut self, name: impl Into<String>, value: impl Into<DynamicValue>) -> Self {
        self.context.insert(name.into(), value.into().into());
        self
    }
}

/// A standard component interaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Event(Event),
    FunctionCall(FunctionCall),
}

impl From<Event> for Action {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

impl From<FunctionCall> for Action {
    fn from(value: FunctionCall) -> Self {
        Self::FunctionCall(value)
    }
}

impl From<Action> for Value {
    fn from(value: Action) -> Self {
        to_value(value)
    }
}

fn to_value(value: impl Serialize) -> Value {
    serde_json::to_value(value).expect("A2UI authoring values must be JSON serializable")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn dynamic_function_call_does_not_require_a_catalog_id() {
        let expected = json!({
            "@call": "formatDate",
            "args": {"value": "2026-11-23"}
        });
        let dynamic: DynamicString =
            serde_json::from_value(expected.clone()).expect("dynamic call must deserialize");

        assert_eq!(
            dynamic,
            DynamicString::Call(FunctionCall::new("formatDate").arg("value", "2026-11-23"))
        );
        assert_eq!(
            serde_json::to_value(dynamic).expect("dynamic call must serialize"),
            expected
        );
    }

    #[test]
    fn function_call_rejects_unknown_fields() {
        let input = json!({"@call": "formatDate", "unknown": true});
        let _ = serde_json::from_value::<FunctionCall>(input)
            .expect_err("unknown function-call fields must fail");
    }
}
