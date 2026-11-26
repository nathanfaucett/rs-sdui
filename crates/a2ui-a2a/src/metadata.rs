use std::{collections::HashMap, hash::BuildHasher};

use a2a::AgentExtension;
use a2ui::{AgentCapability, RendererCapabilities, RendererDataModel};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::BindingError;

pub const A2UI_EXTENSION_URI: &str = "https://a2ui.org/a2a-extension/a2ui/v1.0";
const RENDERER_CAPABILITIES_KEY: &str = "a2uiRendererCapabilities";
const RENDERER_DATA_MODEL_KEY: &str = "a2uiRendererDataModel";

/// Builds the official A2UI Agent Card extension with inner capability parameters.
///
/// # Errors
/// Returns an error if the capability parameters cannot be serialized.
pub fn agent_extension(capability: &AgentCapability) -> Result<AgentExtension, BindingError> {
    let mut params = HashMap::new();
    if let Some(catalog_ids) = &capability.supported_catalog_ids {
        params.insert(
            "supportedCatalogIds".to_owned(),
            serde_json::to_value(catalog_ids)?,
        );
    }
    if let Some(accepts_inline) = capability.accepts_inline_catalogs {
        params.insert(
            "acceptsInlineCatalogs".to_owned(),
            Value::Bool(accepts_inline),
        );
    }
    Ok(AgentExtension {
        uri: A2UI_EXTENSION_URI.to_owned(),
        description: Some("A2UI v1.0".to_owned()),
        required: Some(false),
        params: (!params.is_empty()).then_some(params),
    })
}

/// Builds standard per-message renderer capabilities and optional data-model metadata.
///
/// # Errors
/// Returns an error if either payload cannot be serialized.
pub fn renderer_metadata(
    capabilities: &RendererCapabilities,
    data_model: Option<&RendererDataModel>,
) -> Result<HashMap<String, Value>, BindingError> {
    let mut metadata = HashMap::from_iter([(
        RENDERER_CAPABILITIES_KEY.to_owned(),
        serde_json::to_value(capabilities)?,
    )]);
    if let Some(data_model) = data_model {
        metadata.insert(
            RENDERER_DATA_MODEL_KEY.to_owned(),
            serde_json::to_value(data_model)?,
        );
    }
    Ok(metadata)
}

/// Reads versioned renderer capabilities when present.
///
/// # Errors
/// Returns an error if the metadata value is not a valid renderer capability payload.
pub fn read_renderer_capabilities<S: BuildHasher>(
    metadata: &HashMap<String, Value, S>,
) -> Result<Option<RendererCapabilities>, BindingError> {
    read_metadata(metadata, RENDERER_CAPABILITIES_KEY)
}

/// Reads the renderer data model when present.
///
/// # Errors
/// Returns an error if the metadata value is not a valid renderer data-model payload.
pub fn read_renderer_data_model<S: BuildHasher>(
    metadata: &HashMap<String, Value, S>,
) -> Result<Option<RendererDataModel>, BindingError> {
    read_metadata(metadata, RENDERER_DATA_MODEL_KEY)
}

fn read_metadata<T: DeserializeOwned, S: BuildHasher>(
    metadata: &HashMap<String, Value, S>,
    key: &str,
) -> Result<Option<T>, BindingError> {
    metadata
        .get(key)
        .map(|value| serde_json::from_value(value.clone()).map_err(BindingError::from))
        .transpose()
}
