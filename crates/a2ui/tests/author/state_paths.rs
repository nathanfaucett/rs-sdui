use a2ui::A2uiData;
use serde::Serialize;

#[derive(A2uiData, Serialize)]
#[serde(rename_all = "camelCase")]
struct Nested {
    display_name: String,
}

#[derive(A2uiData, Serialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    item_id: u64,
}

#[derive(A2uiData, Serialize)]
enum Kind {
    Basic,
}

#[derive(A2uiData, Serialize)]
#[serde(rename_all = "camelCase")]
struct State {
    simple_value: String,
    #[serde(rename = "renamed~/field")]
    escaped: bool,
    #[serde(flatten)]
    nested: Nested,
    items: Vec<Item>,
    optional: Option<Nested>,
    scalars: [String; 1],
    kind: Kind,
    #[serde(skip)]
    skipped: String,
}

#[test]
fn generated_paths_match_the_serialized_data_model() {
    let state = State {
        simple_value: "value".to_owned(),
        escaped: true,
        nested: Nested {
            display_name: "Ada".to_owned(),
        },
        items: vec![Item { item_id: 7 }],
        optional: Some(Nested {
            display_name: "Grace".to_owned(),
        }),
        scalars: ["first".to_owned()],
        kind: Kind::Basic,
        skipped: "not serialized".to_owned(),
    };
    let json = serde_json::to_value(&state).expect("state must serialize");
    assert_eq!(state.skipped, "not serialized");
    assert!(json.get("skipped").is_none());
    let paths = State::paths();
    let bindings = [
        paths.simple_value(),
        paths.escaped(),
        paths.nested().display_name(),
        paths.items().index(0).item_id(),
        paths.optional().display_name(),
        paths.scalars().index(0),
        paths.kind(),
    ];

    for binding in &bindings {
        assert!(
            json.pointer(&binding.path).is_some(),
            "missing serialized path {}",
            binding.path
        );
    }
    assert_eq!(bindings[0].path, "/simpleValue");
    assert_eq!(bindings[1].path, "/renamed~0~1field");
    assert_eq!(bindings[2].path, "/displayName");
    assert_eq!(bindings[3].path, "/items/0/itemId");
    assert_eq!(bindings[4].path, "/optional/displayName");
    assert_eq!(bindings[5].path, "/scalars/0");
    assert_eq!(bindings[6].path, "/kind");
}
