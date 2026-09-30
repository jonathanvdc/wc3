//! Check model conversion without starting a GPU renderer.
use std::error::Error;
use std::fs;
use wc3_bevy::Wc3Model;

fn main() -> Result<(), Box<dyn Error>> {
    for path in std::env::args().skip(1) {
        let model = Wc3Model::decode(&fs::read(&path)?)?;
        println!(
            "{path}: version {}, {} geosets, {} materials, {} bones, {} sequences",
            model.source_version,
            model.model.geosets().len(),
            model.model.materials().len(),
            model.model.bones().len(),
            model.model.sequences().len(),
        );
    }
    Ok(())
}
