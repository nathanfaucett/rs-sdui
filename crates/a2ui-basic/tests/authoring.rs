use a2ui_author::{DataBinding, DataModel, Event};
use a2ui_basic::v1::{self, Button, ButtonVariant, Column, Text};
use serde_json::json;

#[test]
fn generated_builders_emit_standard_a2ui_messages() {
    let message = v1::surface("booking")
        .components([
            Column::new("root", ["title", "submit"]).into(),
            Text::new("title", DataBinding::new("/trip/name")).into(),
            Text::new("submit_label", "Confirm booking").into(),
            Button::new(
                "submit",
                "submit_label",
                Event::new("confirm_booking").context("price", DataBinding::new("/price")),
            )
            .variant(ButtonVariant::Primary)
            .into(),
        ])
        .data_model(
            DataModel::new()
                .with("trip", json!({"name": "Berlin to San Francisco"}))
                .with("price", 455),
        )
        .into_message();

    assert_eq!(
        serde_json::to_value(message).expect("message must serialize"),
        json!({
            "version": "1.0",
            "createSurface": {
                "surfaceId": "booking",
                "catalogId": v1::CATALOG_ID,
                "components": [
                    {"id": "root", "component": "Column", "children": ["title", "submit"]},
                    {"id": "title", "component": "Text", "text": {"@path": "/trip/name"}},
                    {"id": "submit_label", "component": "Text", "text": "Confirm booking"},
                    {
                        "id": "submit",
                        "component": "Button",
                        "child": "submit_label",
                        "action": {
                            "event": {
                                "name": "confirm_booking",
                                "context": {"price": {"@path": "/price"}}
                            }
                        },
                        "variant": "primary"
                    }
                ],
                "dataModel": {
                    "trip": {"name": "Berlin to San Francisco"},
                    "price": 455
                }
            }
        })
    );
}

#[test]
fn generated_functions_emit_standard_function_calls() {
    let call = v1::functions::format_currency("USD", DataBinding::new("/price")).arg("decimals", 2);

    assert_eq!(
        serde_json::to_value(call).expect("function call must serialize"),
        json!({
            "@call": "formatCurrency",
            "catalogId": v1::CATALOG_ID,
            "args": {
                "currency": "USD",
                "value": {"@path": "/price"},
                "decimals": 2
            }
        })
    );
}
