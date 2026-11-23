use a2ui_author::{A2uiPath, DataModel, Event};
use a2ui_basic::v1::{self, Button, ButtonVariant, Column, Text};
use a2ui_view::{Element, EventExt};
use serde::Serialize;
use serde_json::json;

#[derive(A2uiPath, Serialize)]
struct Trip {
    name: String,
}

#[derive(A2uiPath, Serialize)]
struct Booking {
    trip: Trip,
    price: u32,
}

fn confirm_button() -> impl Into<Element> + 'static {
    Button::new(
        Text::new("Confirm booking"),
        Event::new("confirm_booking")
            .context("price", Booking::paths().price())
            .on(|(): &(), _| ()),
    )
    .variant(ButtonVariant::Primary)
}

#[test]
fn generated_components_render_nested_standard_a2ui_messages() {
    let state = Booking {
        trip: Trip {
            name: "Berlin to San Francisco".to_owned(),
        },
        price: 455,
    };
    let message = v1::surface("booking")
        .root(Column::new((
            Text::new(Booking::paths().trip().name()),
            confirm_button(),
        )))
        .data_model(
            DataModel::new()
                .with("trip", json!({"name": state.trip.name}))
                .with("price", state.price),
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
    let call = v1::functions::format_currency("USD", Booking::paths().price()).arg("decimals", 2);

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
