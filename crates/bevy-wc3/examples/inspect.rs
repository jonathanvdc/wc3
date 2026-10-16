//! Check model conversion without starting a GPU renderer.
#[path = "support/model.rs"]
mod semantic;
use std::env::args;
use std::error::Error;
use std::fs::read;

fn main() -> Result<(), Box<dyn Error>> {
    for path in args().skip(1) {
        let model = semantic::decode(&read(&path)?)?;
        println!(
            "{path}: {} meshes, {} materials, {} nodes, {} clips",
            model.meshes().count(),
            model.materials().count(),
            model.nodes().count(),
            model.clips().count(),
        );
    }
    Ok(())
}
