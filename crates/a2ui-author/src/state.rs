use std::marker::PhantomData;

use serde_json::{Number, Value};

use crate::DataBinding;

/// A type that can produce a typed path into its serialized JSON value.
///
/// Derive this trait together with `Serialize`. Struct derivations provide a
/// `Type::paths()` entry point. Enum derivations act as scalar fields.
pub trait A2uiData {
    type Path;

    #[doc(hidden)]
    fn from_path(path: StatePath) -> Self::Path;
}

/// The serialized path carried by generated field builders.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatePath(String);

impl StatePath {
    #[must_use]
    pub const fn root() -> Self {
        Self(String::new())
    }

    #[must_use]
    pub fn child(&self, name: &str) -> Self {
        let name = name.replace('~', "~0").replace('/', "~1");
        Self(format!("{}/{name}", self.0))
    }

    #[must_use]
    pub fn index(&self, index: usize) -> Self {
        Self(format!("{}/{index}", self.0))
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

/// A typed path into a JSON array.
#[derive(Clone, Debug)]
pub struct CollectionPath<T> {
    path: StatePath,
    marker: PhantomData<fn() -> T>,
}

impl<T: A2uiData> CollectionPath<T> {
    #[must_use]
    pub fn index(self, index: usize) -> T::Path {
        T::from_path(self.path.index(index))
    }
}

impl<T: A2uiData> A2uiData for Vec<T> {
    type Path = CollectionPath<T>;

    fn from_path(path: StatePath) -> Self::Path {
        CollectionPath {
            path,
            marker: PhantomData,
        }
    }
}

impl<T: A2uiData, const N: usize> A2uiData for [T; N] {
    type Path = CollectionPath<T>;

    fn from_path(path: StatePath) -> Self::Path {
        CollectionPath {
            path,
            marker: PhantomData,
        }
    }
}

impl<T: A2uiData> A2uiData for Option<T> {
    type Path = T::Path;

    fn from_path(path: StatePath) -> Self::Path {
        T::from_path(path)
    }
}

impl<T: A2uiData> A2uiData for Box<T> {
    type Path = T::Path;

    fn from_path(path: StatePath) -> Self::Path {
        T::from_path(path)
    }
}

macro_rules! scalar_paths {
    ($($type:ty),* $(,)?) => {
        $(
            impl A2uiData for $type {
                type Path = DataBinding;

                fn from_path(path: StatePath) -> Self::Path {
                    DataBinding::new(path.into_string())
                }
            }
        )*
    };
}

scalar_paths!(
    String, bool, char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64,
    Number, Value,
);
