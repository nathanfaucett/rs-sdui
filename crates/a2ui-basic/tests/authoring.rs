use a2ui_author::{DataBinding, DataModel, Event};
use a2ui_basic::v1::{self, Button, ButtonVariant, Column, Text};
use a2ui_view::{Element, EventExt};
use serde_json::json;

fn confirm_button() -> impl Into<Element> + 'static {
    Button::new(
        Text::new("Confirm booking"),
        Event::new("confirm_booking")
            .context("price", DataBinding::new("/price"))
            .on(|(): &(), _| ()),
    )
    .variant(ButtonVariant::Primary)
}

#[test]
fn generated_components_render_nested_standard_a2ui_messages() {
    let message = v1::surface("booking")
        .root(Column::new((
            Text::new(DataBinding::new("/trip/name")),
            confirm_button(),
        )))
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
                    {"id": "root", "component": "Column", "children": ["root_0", "root_1"]},
                    {"id": "root_0", "component": "Text", "text": {"@path": "/trip/name"}},
                    {
                        "id": "root_1",
                        "component": "Button",
                        "child": "root_1_0",
                        "action": {
                            "event": {
                                "name": "confirm_booking",
                                "context": {"price": {"@path": "/price"}}
                            }
                        },
                        "variant": "primary"
                    },
                    {"id": "root_1_0", "component": "Text", "text": "Confirm booking"}
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
