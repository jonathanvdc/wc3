//! List direct declared file references and game-provided replaceable resources.
mod support;
use clap::Parser;
use std::collections::BTreeMap;
use std::path::PathBuf;
use support::{load, paths, Result};
use wc3::model::chunks::ModelChunk;
use wc3::model::{visit_model, Model, ModelVersion};

#[derive(Parser)]
#[command(about = "List direct model file references and replaceable resources")]
struct Args {
    /// Input MDX or MDL model.
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut model = load(&args.input)?;
    visit_model!(&mut model, |typed| dump(typed))
}

fn dump<V: ModelVersion>(model: &mut Model<V>) -> Result<()> {
    let mut files: BTreeMap<String, Vec<String>> = BTreeMap::new();
    paths(model, |location, path| {
        let text = path.text();
        if !text.is_empty() {
            files.entry(text.into_owned()).or_default().push(location);
        }
        Ok(())
    })?;
    for (path, locations) in files {
        println!("{path}\t{}", locations.join(", "));
    }
    for (index, chunk) in model.chunks.iter().enumerate() {
        match chunk {
            ModelChunk::Textures(chunk) => {
                for (i, texture) in chunk.records.iter().enumerate() {
                    if texture.replaceable_id != 0 {
                        println!(
                            "replaceable:{}\tchunk[{index}].textures[{i}]",
                            texture.replaceable_id
                        );
                    }
                }
            }
            ModelChunk::ParticleEmitters2(chunk) => {
                for (i, emitter) in chunk.records.iter().enumerate() {
                    if emitter.replaceable_id != 0 {
                        println!(
                            "replaceable:{}\tchunk[{index}].particle_emitters2[{i}]",
                            emitter.replaceable_id
                        );
                    }
                }
            }
            ModelChunk::PopcornEmitters(chunk) => {
                for (i, emitter) in chunk.records.iter().enumerate() {
                    if emitter.replaceable_id != 0 {
                        println!(
                            "replaceable:{}\tchunk[{index}].popcorn_emitters[{i}]",
                            emitter.replaceable_id
                        );
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
