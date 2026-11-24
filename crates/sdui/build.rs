use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    sdui_codegen::generate(&[sdui_codegen::Catalog::new("v1", "catalog.json")])
}
