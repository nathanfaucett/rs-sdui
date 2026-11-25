use a2ui::{Component, DynamicString};
use sdui::{Element, IntoChildren, ViewAction};

pub const CATALOG_ID: &str = "test:catalog";

pub fn text(value: impl Into<DynamicString>) -> Element {
    let value: DynamicString = value.into();
    Element::new(Component::new("", "Text")).property("text", value)
}

pub fn button(child: impl Into<Element>, action: impl Into<ViewAction>) -> Element {
    Element::new(Component::new("", "Button"))
        .child("child", child)
        .action("action", action)
}

pub fn column(children: impl IntoChildren) -> Element {
    Element::new(Component::new("", "Column")).children("children", children)
}
