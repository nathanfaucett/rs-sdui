//! Build-time generation of typed authoring helpers from A2UI catalogs.

mod generator;

pub use generator::{Catalog, generate, generate_source};
