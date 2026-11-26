use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{DataModel, ProtocolVersion};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentCapability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_catalog_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepts_inline_catalogs: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCapabilities {
    #[serde(rename = "1.0")]
    pub v1_0: AgentCapability,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RendererCapability {
    pub supported_catalog_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_catalogs: Option<Vec<Map<String, Value>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererCapabilities {
    #[serde(rename = "1.0")]
    pub v1_0: RendererCapability,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RendererDataModel {
    pub version: ProtocolVersion,
    pub surfaces: BTreeMap<String, DataModel>,
}
