use std::collections::{BTreeMap, BTreeSet};

use crate::{CreateSurface, UpdateDataModel};
use serde_json::Value;

pub fn updates(previous: &CreateSurface, next: &CreateSurface) -> Vec<UpdateDataModel> {
    let previous_model = data_model(previous);
    let next_model = data_model(next);
    let mut changes = BTreeMap::new();
    diff(&previous_model, &next_model, None, &mut changes);

    let mut bindings = BTreeSet::new();
    collect_bindings(previous, &mut bindings);
    collect_bindings(next, &mut bindings);
    for path in bindings {
        if let Some(value) = next_model.pointer(&path) {
            insert(&mut changes, Some(path), value.clone());
        } else {
            // A removed binding target needs replacement of an ancestor to remove it.
            insert(&mut changes, None, next_model.clone());
        }
    }

    changes
        .into_iter()
        .map(|(path, value)| {
            path.map_or_else(
                || UpdateDataModel::root(&next.surface_id, next_model.clone()),
                |path| UpdateDataModel::new(&next.surface_id, path, value),
            )
        })
        .collect()
}

fn data_model(view: &CreateSurface) -> Value {
    Value::Object(
        view.data_model
            .as_ref()
            .map_or_else(serde_json::Map::new, |model| model.0.clone()),
    )
}

fn diff(
    previous: &Value,
    next: &Value,
    path: Option<String>,
    changes: &mut BTreeMap<Option<String>, Value>,
) {
    match (previous, next) {
        (Value::Object(previous), Value::Object(next)) => {
            if previous.keys().any(|key| !next.contains_key(key)) {
                // A2UI has no key-deletion operation. Replace the smallest parent object.
                insert(changes, path, Value::Object(next.clone()));
                return;
            }
            for (key, next_value) in next {
                let child_path = Some(child_path(path.as_deref(), key));
                if let Some(previous_value) = previous.get(key) {
                    diff(previous_value, next_value, child_path, changes);
                } else {
                    insert(changes, child_path, next_value.clone());
                }
            }
        }
        _ if previous != next => insert(changes, path, next.clone()),
        _ => {}
    }
}

fn child_path(parent: Option<&str>, key: &str) -> String {
    let key = key.replace('~', "~0").replace('/', "~1");
    parent.map_or_else(|| format!("/{key}"), |parent| format!("{parent}/{key}"))
}

fn insert(changes: &mut BTreeMap<Option<String>, Value>, path: Option<String>, value: Value) {
    if changes
        .keys()
        .any(|ancestor| is_ancestor(ancestor.as_deref(), path.as_deref()))
    {
        return;
    }
    changes.retain(|existing, _| !is_ancestor(path.as_deref(), existing.as_deref()));
    changes.insert(path, value);
}

fn is_ancestor(ancestor: Option<&str>, path: Option<&str>) -> bool {
    match (ancestor, path) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(ancestor), Some(path)) => {
            ancestor == path
                || path
                    .strip_prefix(ancestor)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        }
    }
}

fn collect_bindings(view: &CreateSurface, paths: &mut BTreeSet<String>) {
    for component in view.components.as_deref().unwrap_or_default() {
        for value in component.0.values() {
            collect_value_bindings(value, paths);
        }
    }
}

fn collect_value_bindings(value: &Value, paths: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            if let Some(path) = object.get("@path").and_then(Value::as_str)
                && path.starts_with('/')
            {
                paths.insert(path.to_owned());
            }
            for value in object.values() {
                collect_value_bindings(value, paths);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_value_bindings(value, paths);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::{Component, DataBinding, DataModel};
    use serde_json::json;

    use super::*;

    #[test]
    fn emits_only_the_changed_leaf() {
        let previous = surface(&json!({"profile": {"name": "Ada", "age": 1}, "same": true}));
        let next = surface(&json!({"profile": {"name": "Ada", "age": 2}, "same": true}));

        let updates = updates(&previous, &next);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].path.as_deref(), Some("/profile/age"));
        assert_eq!(updates[0].value, json!(2));
    }

    #[test]
    fn replaces_the_smallest_parent_when_a_key_is_removed() {
        let previous = surface(&json!({"profile": {"name": "Ada", "age": 1}, "same": true}));
        let next = surface(&json!({"profile": {"name": "Ada"}, "same": true}));

        let updates = updates(&previous, &next);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].path.as_deref(), Some("/profile"));
        assert_eq!(updates[0].value, json!({"name": "Ada"}));
    }

    #[test]
    fn refreshes_bound_paths_even_when_rust_state_is_unchanged() {
        let previous = surface(&json!({"profile": {"name": "Ada"}, "same": true}))
            .components([Component::new("root", "TextField")
                .property("value", DataBinding::new("/profile/name"))]);
        let next = previous.clone();

        let updates = updates(&previous, &next);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].path.as_deref(), Some("/profile/name"));
        assert_eq!(updates[0].value, json!("Ada"));
    }

    fn surface(model: &Value) -> CreateSurface {
        CreateSurface::new("main", "test:catalog").data_model(DataModel::from(
            model
                .as_object()
                .cloned()
                .expect("test model must be an object"),
        ))
    }
}
