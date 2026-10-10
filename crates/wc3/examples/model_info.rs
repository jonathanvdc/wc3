//! Inspect model metadata without rendering or loading its dependencies.
mod support;
use clap::Parser;
use std::path::PathBuf;
use support::{load, Result};
use wc3::model::chunks::Chunk;
use wc3::model::{visit_model, Model, ModelDialect};

#[derive(Parser)]
#[command(about = "Inspect model metadata, geometry, and animations")]
struct Args {
    /// Input MDX or MDL model.
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let model = load(&args.input)?;
    visit_model!(&model, |typed| dump(typed));
    Ok(())
}

fn dump<V: ModelDialect>(model: &Model<V>)
where
    V::Extension: Chunk,
{
    println!("Version: {}", model.version());
    if let Some(info) = model.model_info() {
        println!(
            "Name: {}\nBounds: {:?} .. {:?}\nRadius: {}",
            info.name.text(),
            info.minimum_extent,
            info.maximum_extent,
            info.bounds_radius
        );
    }
    let geosets = model.geosets();
    println!("Geosets: {}\nVertices: {}\nFace indices: {}\nMaterials: {}\nTextures: {}\nBones: {}\nNodes: {}", geosets.len(), geosets.iter().map(|g| g.vertices().len()).sum::<usize>(), geosets.iter().map(|g| g.face_indices().len()).sum::<usize>(), model.materials().len(), model.textures().len(), model.bones().len(), model.nodes().len());
    println!(
        "Particle emitters: {}\nParticle emitters 2: {}\nRibbon emitters: {}\nPopcorn emitters: {}",
        model.particle_emitters().len(),
        model.particle_emitters2().len(),
        model.ribbon_emitters().len(),
        model.try_popcorn_emitters().map_or(0, |items| items.len())
    );
    for chunk in &model.chunks {
        println!("Chunk: {}", String::from_utf8_lossy(&chunk.tag()));
    }
    for (index, sequence) in model.sequences().iter().enumerate() {
        println!(
            "Sequence {index}: {} [{}..{} ms], duration {} ms",
            sequence.name.text(),
            sequence.interval[0],
            sequence.interval[1],
            sequence.interval[1].saturating_sub(sequence.interval[0])
        );
    }
}
