//! Check model conversion without starting a GPU renderer.
use bevy_wc3::Wc3Model;
use std::env::args;
use std::error::Error;
use std::fs::read;

fn main() -> Result<(), Box<dyn Error>> {
    for path in args().skip(1) {
        let model = Wc3Model::decode(&read(&path)?)?;
        println!(
            "{path}: version {}, {} meshes, {} materials, {} nodes, {} clips",
            model.source_version,
            model.model.meshes().count(),
            model.model.materials().count(),
            model.model.nodes().count(),
            model.model.clips().count(),
        );
    }
    Ok(())
}
