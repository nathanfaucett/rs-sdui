use a2ui_author::{AgentToRendererMessage, CreateSurface, DataModel, Metadata};

use crate::Element;

/// A standalone A2UI surface built from one declarative root element.
#[derive(Clone, Debug, PartialEq)]
pub struct Surface(CreateSurface);

impl Surface {
    #[must_use]
    pub fn new(surface_id: impl Into<String>, catalog_id: impl Into<String>) -> Self {
        Self(CreateSurface::new(surface_id, catalog_id))
    }

    #[must_use]
    pub fn root(mut self, root: impl Into<Element>) -> Self {
        self.0.components = Some(root.into().into_components());
        self
    }

    #[must_use]
    pub fn data_model(mut self, data_model: impl Into<DataModel>) -> Self {
        self.0 = self.0.data_model(data_model);
        self
    }

    #[must_use]
    pub fn send_data_model(mut self, send: bool) -> Self {
        self.0 = self.0.send_data_model(send);
        self
    }

    #[must_use]
    pub fn metadata(mut self, metadata: Metadata) -> Self {
        self.0 = self.0.metadata(metadata);
        self
    }

    #[must_use]
    pub fn into_message(self) -> AgentToRendererMessage {
        self.0.into_message()
    }
}
