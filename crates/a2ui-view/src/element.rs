#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "fluent authoring methods return new values"
)]

use serde_json::Value;

use a2ui_author::{Action, ChildList, ChildTemplate, Component};

use crate::{EventHandler, ViewAction};

/// A catalog component and its nested children.
///
/// Flattening assigns the required `root` ID and deterministic child IDs.
#[derive(Debug)]
pub struct Element {
    component: Component,
    relations: Vec<Relation>,
    handlers: Vec<EventHandler>,
}

/// The standard wire components and source-scoped Rust handlers produced by a view.
#[derive(Debug)]
pub struct ElementParts {
    pub components: Vec<Component>,
    pub handlers: Vec<(String, EventHandler)>,
}

#[derive(Debug)]
enum Relation {
    Child(String, Element),
    Children(String, Children),
}

/// Converts one view, a fragment, or an A2UI child-list reference into children.
pub trait IntoChildren {
    fn into_children(self) -> Children;
}

/// Nested elements or a standard A2UI child-list reference.
#[derive(Debug)]
pub enum Children {
    Elements(Vec<Element>),
    List(ChildList),
}

impl Element {
    /// Wraps a wire component. Its ID is assigned when the tree is flattened.
    pub fn new(mut component: Component) -> Self {
        component.0.remove("id");
        Self {
            component,
            relations: Vec::new(),
            handlers: Vec::new(),
        }
    }

    /// Sets a standard component property.
    pub fn property(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.component = self.component.property(name, value);
        self
    }

    /// Sets a standard action and retains its optional Rust-only handler.
    pub fn action(mut self, property: impl Into<String>, action: impl Into<ViewAction>) -> Self {
        match action.into() {
            ViewAction::Standard(action) => {
                self.component = self.component.property(property, Value::from(action));
            }
            ViewAction::Handled(handler) => {
                let event = handler.event().clone();
                self.component = self
                    .component
                    .property(property, Value::from(Action::Event(event)));
                self.handlers.push(handler);
            }
        }
        self
    }

    /// Adds one nested element and writes its ID to `property` during flattening.
    pub fn child(mut self, property: impl Into<String>, child: impl Into<Self>) -> Self {
        self.relations
            .push(Relation::Child(property.into(), child.into()));
        self
    }

    /// Adds nested elements or a standard child-list reference.
    pub fn children(mut self, property: impl Into<String>, children: impl IntoChildren) -> Self {
        self.relations.push(Relation::Children(
            property.into(),
            children.into_children(),
        ));
        self
    }

    /// Produces the standard flat A2UI component list and discards Rust handlers.
    pub fn into_components(self) -> Vec<Component> {
        self.into_parts().components
    }

    /// Splits standard wire data from Rust-only event handlers.
    pub fn into_parts(self) -> ElementParts {
        let mut parts = ElementParts {
            components: Vec::new(),
            handlers: Vec::new(),
        };
        flatten(self, "root", &mut parts);
        parts
    }
}

impl Children {
    pub const fn new() -> Self {
        Self::Elements(Vec::new())
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Into<Element>> FromIterator<E> for Children {
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        Self::Elements(iter.into_iter().map(Into::into).collect())
    }
}

impl From<Element> for Children {
    fn from(value: Element) -> Self {
        Self::Elements(vec![value])
    }
}

impl<T: Into<Element>> IntoChildren for T {
    fn into_children(self) -> Children {
        Children::from(self.into())
    }
}

impl IntoChildren for Children {
    fn into_children(self) -> Children {
        self
    }
}

impl IntoChildren for ChildList {
    fn into_children(self) -> Children {
        Children::List(self)
    }
}

impl IntoChildren for ChildTemplate {
    fn into_children(self) -> Children {
        Children::List(ChildList::Template(self))
    }
}

impl<E: Into<Element>> IntoChildren for Vec<E> {
    fn into_children(self) -> Children {
        Children::Elements(self.into_iter().map(Into::into).collect())
    }
}

impl<E: Into<Element>, const N: usize> IntoChildren for [E; N] {
    fn into_children(self) -> Children {
        Children::Elements(self.into_iter().map(Into::into).collect())
    }
}

impl IntoChildren for () {
    fn into_children(self) -> Children {
        Children::new()
    }
}

macro_rules! tuple_children {
    ($($type:ident),+ $(,)?) => {
        impl<$($type: Into<Element>),+> IntoChildren for ($($type,)+) {
            #[allow(non_snake_case, reason = "tuple element names are type parameters")]
            fn into_children(self) -> Children {
                let ($($type,)+) = self;
                Children::Elements(vec![$($type.into()),+])
            }
        }
    };
}

tuple_children!(A);
tuple_children!(A, B);
tuple_children!(A, B, C);
tuple_children!(A, B, C, D);
tuple_children!(A, B, C, D, E);
tuple_children!(A, B, C, D, E, F);
tuple_children!(A, B, C, D, E, F, G);
tuple_children!(A, B, C, D, E, F, G, H);
tuple_children!(A, B, C, D, E, F, G, H, I);
tuple_children!(A, B, C, D, E, F, G, H, I, J);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K, L);

fn flatten(element: Element, path: &str, output: &mut ElementParts) {
    let Element {
        mut component,
        relations,
        handlers,
    } = element;
    output.handlers.extend(
        handlers
            .into_iter()
            .map(|handler| (path.to_owned(), handler)),
    );
    component = component.property("id", path);

    let mut descendants = Vec::new();
    for relation in relations {
        match relation {
            Relation::Child(property, child) => {
                let index = descendants.len();
                let child_path = format!("{path}_{index}");
                component = component.property(property, child_path.clone());
                descendants.push((child, child_path));
            }
            Relation::Children(property, Children::Elements(children)) => {
                let mut ids = Vec::with_capacity(children.len());
                for child in children {
                    let index = descendants.len();
                    let child_path = format!("{path}_{index}");
                    ids.push(child_path.clone());
                    descendants.push((child, child_path));
                }
                component = component.property(property, Value::from(ChildList::Static(ids)));
            }
            Relation::Children(property, Children::List(children)) => {
                component = component.property(property, Value::from(children));
            }
        }
    }

    output.components.push(component);
    for (child, child_path) in descendants {
        flatten(child, &child_path, output);
    }
}
