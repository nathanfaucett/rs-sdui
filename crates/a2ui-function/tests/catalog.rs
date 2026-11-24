mod support;

use a2ui_function::{Catalog, CatalogSet, ReturnType};
use serde_json::json;
use support::{BASIC_CATALOG, CUSTOM_CATALOG, basic_catalogs, custom_catalogs};

#[test]
fn parses_multiple_catalogs_and_rejects_duplicate_ids() {
    let basic = Catalog::from_json(BASIC_CATALOG).expect("basic catalog");
    let custom = Catalog::from_json(CUSTOM_CATALOG).expect("custom catalog");
    let set = CatalogSet::new([basic.clone(), custom]).expect("catalog set");
    assert_eq!(set.len(), 2);
    assert!(set.get(basic.id()).is_some());

    let duplicate = CatalogSet::new([basic.clone(), basic]);
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
fn real_and_custom_catalogs_compile_offline() {
    assert_eq!(basic_catalogs().len(), 1);
    assert_eq!(custom_catalogs().len(), 1);
}
