use crate::support::{CUSTOM_CATALOG, custom_catalogs};
use sdui::function::{Catalog, CatalogSet, ReturnType};
use serde_json::json;

#[test]
fn parses_multiple_catalogs_and_rejects_duplicate_ids() {
    let custom = Catalog::from_json(CUSTOM_CATALOG).expect("custom catalog");
    let other_json = CUSTOM_CATALOG.replace("test:functions", "test:other");
    let other = Catalog::from_json(&other_json).expect("other catalog");
    let set = CatalogSet::new([custom.clone(), other]).expect("catalog set");
    assert_eq!(set.len(), 2);
    assert!(set.get(custom.id()).is_some());

    let duplicate = CatalogSet::new([custom.clone(), custom]);
    assert_eq!(
        duplicate.expect_err("duplicate IDs must fail").code,
        "INVALID_CATALOG"
    );
}

#[test]
fn all_declared_return_shapes_are_checked() {
    assert!(ReturnType::String.accepts(&json!("x")));
    assert!(ReturnType::Number.accepts(&json!(1.5)));
    assert!(ReturnType::Boolean.accepts(&json!(true)));
    assert!(ReturnType::Array.accepts(&json!([])));
    assert!(ReturnType::Object.accepts(&json!({})));
    assert!(ReturnType::Any.accepts(&json!(null)));
    assert!(ReturnType::ValidationResult.accepts(&json!({"valid": false})));
    assert!(ReturnType::Void.accepts(&json!(null)));
    assert!(!ReturnType::ValidationResult.accepts(&json!({"valid": "no"})));
    assert!(!ReturnType::Void.accepts(&json!(0)));
}

#[test]
fn custom_catalog_compiles_offline() {
    assert_eq!(custom_catalogs().len(), 1);
}
