use std::collections::{BTreeMap, BTreeSet};

use a2ui::{Component, CreateSurface};

use crate::ApplicationError;

pub fn validate(view: &CreateSurface) -> Result<(), ApplicationError> {
    let components = view.components.as_deref().unwrap_or_default();
    let mut ids = BTreeSet::new();

    for (index, component) in components.iter().enumerate() {
        let id = component.id().ok_or_else(|| {
            ApplicationError::InvalidView(format!("component {index} needs a string id"))
        })?;
        let name = component.name().ok_or_else(|| {
            ApplicationError::InvalidView(format!("component {id:?} needs a string component type"))
        })?;

        if !is_identifier(id) {
            return Err(ApplicationError::InvalidView(format!(
                "component id {id:?} does not follow UAX #31"
            )));
        }
        if !is_identifier(name) || name == "Surface" {
            return Err(ApplicationError::InvalidView(format!(
                "component type {name:?} is not a valid catalog component name"
            )));
        }
        if !ids.insert(id) {
            return Err(ApplicationError::InvalidView(format!(
                "component id {id:?} is duplicated"
            )));
        }

        match (view.catalog_id.is_some(), component.0.get("@catalogId")) {
            (false, None) => {
                return Err(ApplicationError::InvalidView(format!(
                    "component {id:?} needs @catalogId because the surface has no default catalog"
                )));
            }
            (_, Some(value)) if !value.is_string() => {
                return Err(ApplicationError::InvalidView(format!(
                    "component {id:?} has a non-string @catalogId"
                )));
            }
            _ => {}
        }
    }

    if !ids.contains("root") {
        return Err(ApplicationError::InvalidView(
            "the view needs exactly one component with id \"root\"".to_owned(),
        ));
    }
    Ok(())
}

pub fn changed_components(previous: &CreateSurface, next: &CreateSurface) -> Vec<Component> {
    let previous = previous
        .components
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|component| component.id().map(|id| (id, component)))
        .collect::<BTreeMap<_, _>>();

    next.components
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter(|component| {
            component
                .id()
                .and_then(|id| previous.get(id))
                .is_none_or(|previous| *previous != *component)
        })
        .cloned()
        .collect()
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || unicode_ident::is_xid_start(first))
        && characters.all(|character| character == '_' || unicode_ident::is_xid_continue(character))
}
