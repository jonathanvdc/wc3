//! Inspect typed Diablo SNO resources without opening their dependencies.
use clap::{Parser, ValueEnum};
use std::{error::Error, fs, path::PathBuf};
use wc3::model::diablo::{d3, d4};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Game {
    D3,
    D4,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Kind {
    Appearance,
    Animation,
    Actor,
    Material,
    AnimationSet,
    ShaderMap,
    Shader,
}
#[derive(Parser)]
#[command(about = "Inspect supported Diablo III or IV SNO model resources")]
struct Args {
    /// Native SNO document, including its 16-byte envelope.
    input: PathBuf,
    /// Resource namespace; the SNO signature alone does not identify the game.
    #[arg(long, value_enum)]
    game: Game,
    /// Native document family for D3. D4 uses its exact root format hash.
    #[arg(long, value_enum, default_value_t = Kind::Appearance)]
    kind: Kind,
    /// Optional D4 external payload body, excluding its storage envelope.
    #[arg(long)]
    payload: Option<PathBuf>,
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let bytes = fs::read(args.input)?;
    match args.game {
        Game::D3 => {
            if args.payload.is_some() {
                return Err("D3 documents do not use --payload".into());
            }
            match args.kind {
                Kind::Appearance => {
                    let document = d3::Appearance::<d3::V260>::read(&bytes)?;
                    let root = &document.root;
                    let geosets = [&root.geo_set0, &root.geo_set1];
                    let objects = geosets
                        .iter()
                        .flat_map(|g| &g.sub_objects)
                        .collect::<Vec<_>>();
                    println!("D3 appearance V260, SNO {}\nBones: {}\nHardpoints: {}\nSubobjects: {}\nVertices: {}\nIndices: {}\nMaterial slots: {}\nLooks: {}", root.sno_id, root.bones.len(), root.hardpoints.len(), objects.len(), objects.iter().map(|o| o.vertices.len()).sum::<usize>(), objects.iter().map(|o| o.indices.len()).sum::<usize>(), root.materials.len(), root.looks.len());
                }
                Kind::Animation => {
                    let document = d3::Animation::<d3::V118>::read(&bytes)?;
                    println!(
                        "D3 animation V118, SNO {}\nAppearance: {}\nPermutations: {}",
                        document.root.sno_id,
                        document.root.appearance,
                        document.root.permutations.len()
                    );
                }
                Kind::Actor => {
                    let document = d3::Actor::<d3::V282>::read(&bytes)?;
                    println!(
                        "D3 actor V282, SNO {}\nAppearance: {}\nAnimation set: {}",
                        document.root.sno_id, document.root.appearance, document.root.anim_set
                    );
                }
                Kind::Material => {
                    let document = d3::Material::<d3::V25>::read(&bytes)?;
                    println!(
                        "D3 material V25, SNO {}\nShader map: {}\nTexture inputs: {}",
                        document.root.sno_id,
                        document.root.material.shader_map,
                        document.root.material.textures.len()
                    );
                }
                Kind::AnimationSet => {
                    let document = d3::AnimationSet::<d3::V24>::read(&bytes)?;
                    println!(
                        "D3 animation set V24, SNO {}\nBase set: {}",
                        document.root.sno_id, document.root.base_anim_set
                    );
                }
                Kind::ShaderMap => {
                    let document = d3::ShaderMapDocument::<d3::V26>::read(&bytes)?;
                    println!("D3 shader map V26, SNO {}", document.root.sno_id);
                }
                Kind::Shader => {
                    let document = d3::Shader::<d3::V150>::read(&bytes)?;
                    println!("D3 shader V150, SNO {}", document.root.sno_id);
                }
            }
        }
        Game::D4 => {
            let payload = args.payload.map(fs::read).transpose()?;
            let document = if let Some(payload) = payload.as_deref() {
                d4::DynamicDocument::read_with_payload(&bytes, payload)?
            } else {
                d4::DynamicDocument::read(&bytes)?
            };
            match document {
                d4::DynamicDocument::Appearance(document) => println!("D4 appearance 297828CC, SNO {}\nChunks: {}\nSkeleton declarations: {}\nMaterial slots: {}\nLooks: {}", document.root.sno_id, document.root.structure.chunks.len(), document.root.structure.bone_data.len(), document.root.appearance_materials.len(), document.root.appearance_looks.len()),
                d4::DynamicDocument::Animation(document) => println!("D4 animation 2088D14A, SNO {}\nPermutations: {}", document.root.sno_id, document.root.permutations.len()),
                d4::DynamicDocument::Actor(document) => println!("D4 actor C23B892A, SNO {}\nAppearance: {}", document.root.sno_id, document.root.appearance),
                d4::DynamicDocument::Material(document) => println!("D4 material BF40B801, SNO {}", document.root.sno_id),
            }
        }
    }
    Ok(())
}
