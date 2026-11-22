use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    a2ui_codegen::generate(&[a2ui_codegen::Catalog::new("v1", "catalog.json")])
}
