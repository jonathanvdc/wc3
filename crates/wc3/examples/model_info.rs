//! Inspect model metadata without rendering or loading its dependencies.
mod support;
use clap::Parser;
use std::path::PathBuf;
use support::{load, Result};
use wc3::model::formats::mdlx::chunks::Chunk;
use wc3::model::formats::mdlx::{visit_document, Dialect, Document};

#[derive(Parser)]
#[command(about = "Inspect model metadata, geometry, and animations")]
struct Args {
    /// Input MDX or MDL model.
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let model = load(&args.input)?;
    visit_document!(&model, |typed| dump(typed));
    Ok(())
}

fn dump<'data, V: Dialect>(model: &Document<'data, V>)
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
    let (geosets, vertices, faces) =
        model
            .geosets()
            .fold((0, 0, 0), |(count, vertices, faces), geoset| {
                (
                    count + 1,
                    vertices + geoset.vertices().len(),
                    faces + geoset.face_indices().len(),
                )
            });
    println!("Geosets: {}\nVertices: {}\nFace indices: {}\nMaterials: {}\nTextures: {}\nBones: {}\nNodes: {}", geosets, vertices, faces, model.materials().count(), model.textures().count(), model.bones().count(), model.nodes().len());
    println!(
        "Particle emitters: {}\nParticle emitters 2: {}\nRibbon emitters: {}\nPopcorn emitters: {}",
        model.particle_emitters().count(),
        model.particle_emitters2().count(),
        model.ribbon_emitters().count(),
        model
            .try_popcorn_emitters()
            .map_or(0, |items| items.count())
    );
    for chunk in &model.chunks {
        println!("Chunk: {}", String::from_utf8_lossy(&chunk.tag()));
    }
    for (index, sequence) in model.sequences().enumerate() {
        println!(
            "Sequence {index}: {} [{}..{} ms], duration {} ms",
            sequence.name.text(),
            sequence.interval[0],
            sequence.interval[1],
            sequence.interval[1].saturating_sub(sequence.interval[0])
        );
    }
}
