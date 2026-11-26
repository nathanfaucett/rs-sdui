use std::collections::BTreeMap;

use a2ui::{
    AgentCapabilities, AgentCapability, DataModel, ProtocolVersion, RendererCapabilities,
    RendererCapability, RendererDataModel,
};
use serde_json::{Map, json};

#[test]
fn standard_payloads_have_official_json_shapes_and_round_trip() {
    let agent = AgentCapabilities {
        v1_0: AgentCapability {
            supported_catalog_ids: Some(vec!["https://example.com/catalog.json".into()]),
            accepts_inline_catalogs: Some(true),
        },
    };
    let agent_json = json!({
        "1.0": {
            "supportedCatalogIds": ["https://example.com/catalog.json"],
            "acceptsInlineCatalogs": true
        }
    });
    assert_eq!(serde_json::to_value(&agent).unwrap(), agent_json);
    assert_eq!(
        serde_json::from_value::<AgentCapabilities>(agent_json).unwrap(),
        agent
    );

    let renderer = RendererCapabilities {
        v1_0: RendererCapability {
            supported_catalog_ids: vec!["https://example.com/catalog.json".into()],
            inline_catalogs: Some(vec![Map::from_iter([(
                "catalogId".into(),
                json!("https://example.com/custom.json"),
            )])]),
        },
    };
    let renderer_json = json!({
        "1.0": {
            "supportedCatalogIds": ["https://example.com/catalog.json"],
            "inlineCatalogs": [{"catalogId": "https://example.com/custom.json"}]
        }
    });
    assert_eq!(serde_json::to_value(&renderer).unwrap(), renderer_json);
    assert_eq!(
        serde_json::from_value::<RendererCapabilities>(renderer_json).unwrap(),
        renderer
    );

    let data_model = RendererDataModel {
        version: ProtocolVersion::V1_0,
        surfaces: BTreeMap::from([("main".into(), DataModel::new().with("count", 2))]),
    };
    let data_json = json!({
        "version": "1.0",
        "surfaces": {"main": {"count": 2}}
    });
    assert_eq!(serde_json::to_value(&data_model).unwrap(), data_json);
    assert_eq!(
        serde_json::from_value::<RendererDataModel>(data_json).unwrap(),
        data_model
    );
}
