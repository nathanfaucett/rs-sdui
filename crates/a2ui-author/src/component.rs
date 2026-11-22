#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::derive_partial_eq_without_eq,
    reason = "fluent authoring methods return new values"
)]

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{DataBinding, FunctionCall};

/// One entry in an A2UI flat component list.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Component(pub Map<String, Value>);

impl Component {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut properties = Map::new();
        properties.insert("id".to_owned(), Value::String(id.into()));
        properties.insert("component".to_owned(), Value::String(name.into()));
        Self(properties)
    }

    pub fn property(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.0.insert(name.into(), value.into());
        self
    }

    pub fn catalog_id(self, catalog_id: impl Into<String>) -> Self {
        self.property("@catalogId", Value::String(catalog_id.into()))
    }

    pub fn id(&self) -> Option<&str> {
        self.0.get("id").and_then(Value::as_str)
    }

    pub fn name(&self) -> Option<&str> {
        self.0.get("component").and_then(Value::as_str)
    }
}

impl From<Component> for Value {
    fn from(value: Component) -> Self {
        Self::Object(value.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildTemplate {
    pub component_id: String,
    pub path: String,
}

impl ChildTemplate {
    pub fn new(path: impl Into<String>, component_id: impl Into<String>) -> Self {
        Self {
            component_id: component_id.into(),
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChildList {
    Static(Vec<String>),
    Template(ChildTemplate),
}

impl<T: Into<String>> From<Vec<T>> for ChildList {
    fn from(value: Vec<T>) -> Self {
        Self::Static(value.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<String>, const N: usize> From<[T; N]> for ChildList {
    fn from(value: [T; N]) -> Self {
        Self::Static(value.into_iter().map(Into::into).collect())
    }
}

impl From<ChildTemplate> for ChildList {
    fn from(value: ChildTemplate) -> Self {
        Self::Template(value)
    }
}

impl From<ChildList> for Value {
    fn from(value: ChildList) -> Self {
        serde_json::to_value(value).expect("A2UI child lists must be JSON serializable")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    Binding(DataBinding),
    Call(FunctionCall),
}

impl From<DataBinding> for Condition {
    fn from(value: DataBinding) -> Self {
        Self::Binding(value)
    }
}

impl From<FunctionCall> for Condition {
    fn from(value: FunctionCall) -> Self {
        Self::Call(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckRule {
    pub condition: Condition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl CheckRule {
    pub fn new(condition: impl Into<Condition>) -> Self {
        Self {
            condition: condition.into(),
            message: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

impl From<CheckRule> for Value {
    fn from(value: CheckRule) -> Self {
        serde_json::to_value(value).expect("A2UI check rules must be JSON serializable")
    }
}
