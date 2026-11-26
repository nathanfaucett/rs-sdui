use std::collections::{BTreeMap, HashMap};

use a2ui::{
    AgentCapability, DataModel, ProtocolVersion, RendererCapabilities, RendererCapability,
    RendererDataModel,
};
use a2ui_a2a::{
    A2UI_EXTENSION_URI, agent_extension, read_renderer_capabilities, read_renderer_data_model,
    renderer_metadata,
};
use serde_json::json;

#[test]
fn agent_card_and_message_metadata_use_official_scopes() {
    let extension = agent_extension(&AgentCapability {
        supported_catalog_ids: Some(vec!["https://example.com/catalog.json".into()]),
        accepts_inline_catalogs: Some(true),
    })
    .unwrap();
    assert_eq!(extension.uri, A2UI_EXTENSION_URI);
    assert_eq!(extension.required, Some(false));
    assert_eq!(
        extension.params,
        Some(HashMap::from_iter([
            (
                "supportedCatalogIds".into(),
                json!(["https://example.com/catalog.json"]),
            ),
            ("acceptsInlineCatalogs".into(), json!(true)),
        ]))
    );

    let capabilities = RendererCapabilities {
        v1_0: RendererCapability {
            supported_catalog_ids: vec!["https://example.com/catalog.json".into()],
            inline_catalogs: None,
        },
    };
    let data_model = RendererDataModel {
        version: ProtocolVersion::V1_0,
        surfaces: BTreeMap::from([("main".into(), DataModel::new().with("count", 2))]),
    };
    let metadata = renderer_metadata(&capabilities, Some(&data_model)).unwrap();

    assert_eq!(
        read_renderer_capabilities(&metadata).unwrap(),
        Some(capabilities.clone())
    );

    let mut with_other_version = serde_json::to_value(&capabilities).unwrap();
    with_other_version
        .as_object_mut()
        .unwrap()
        .insert("2.0".into(), json!({"supportedCatalogIds": []}));
    let metadata_with_other_version: HashMap<String, serde_json::Value> =
        HashMap::from_iter([("a2uiRendererCapabilities".into(), with_other_version)]);
    assert_eq!(
        read_renderer_capabilities(&metadata_with_other_version).unwrap(),
        Some(capabilities)
    );
    assert_eq!(
        read_renderer_data_model(&metadata).unwrap(),
        Some(data_model)
    );
    assert_eq!(metadata.len(), 2);
    assert_eq!(read_renderer_capabilities(&HashMap::new()).unwrap(), None);
}
