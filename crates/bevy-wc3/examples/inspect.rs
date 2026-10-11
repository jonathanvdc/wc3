//! Check model conversion without starting a GPU renderer.
use bevy_wc3::Wc3Model;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    for path in std::env::args().skip(1) {
        let model = Wc3Model::decode(&fs::read(&path)?)?;
        println!(
            "{path}: version {}, {} geosets, {} materials, {} bones, {} sequences",
            model.source_version,
            model.model.geosets().count(),
            model.model.materials().count(),
            model.model.bones().count(),
            model.model.sequences().count(),
        );
    }
    Ok(())
}
