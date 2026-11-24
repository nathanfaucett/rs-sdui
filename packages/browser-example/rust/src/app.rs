use a2ui::Application;
use a2ui::basic::v1;

use crate::state::TodoApp;

pub const SURFACE_ID: &str = "main";

#[must_use]
pub fn application() -> Application<TodoApp> {
    Application::new(
        SURFACE_ID,
        v1::CATALOG_ID,
        TodoApp::default(),
        TodoApp::view,
    )
}
