//! Explicit, resolved modern WoW character recipes for the shared capture tool.
use bevy::prelude::{Assets, Image};
use bevy_wc3::assembly::{Atlas, Models, PixelRect, RgbaImage, TextureLayer, TextureReplacement};
use bevy_wc3::wow::{
    compile_character, CharacterDefinition, EquipmentBinding, GeosetSelection, ModelPart,
};
use bevy_wc3::{CompiledModel, ModelError};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::{read, read_dir};
use std::path::Path;
use wc3::blp::Blp;
use wc3::model::conversion::{ImportOptions, LossPolicy, UnknownPolicy};
use wc3::model::formats::m2::{Bundle, Chunked, FileKey, V274};
use wc3::model::io::m2 as semantic_m2;
use wc3::model::SocketIdentity;
use wc3::model::{ImageResource, ResourceLocator};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    body: u32,
    geosets: BTreeSet<u16>,
    #[serde(default)]
    textures: BTreeMap<u32, Replacement>,
    #[serde(default)]
    parts: Vec<Part>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Part {
    model: u32,
    /// Optional application label; it has no rendering or inventory semantics.
    #[serde(rename = "slot")]
    _label: Option<String>,
    /// Absence selects validated shared-skeleton deformation.
    attachment: Option<u32>,
    geosets: Option<BTreeSet<u16>>,
    #[serde(default)]
    textures: BTreeMap<u32, Replacement>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Replacement {
    Image(u32),
    Atlas(AtlasRecipe),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AtlasRecipe {
    width: u32,
    height: u32,
    #[serde(default)]
    clear: [u8; 4],
    layers: Vec<Layer>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Layer {
    image: u32,
    destination: [u32; 4],
    source: Option<[u32; 4]>,
    #[serde(default = "opaque")]
    opacity: u8,
}
fn opaque() -> u8 {
    255
}
fn rect([x, y, width, height]: [u32; 4]) -> PixelRect {
    PixelRect {
        x,
        y,
        width,
        height,
    }
}
fn replacements(values: BTreeMap<u32, Replacement>) -> BTreeMap<u32, TextureReplacement> {
    values
        .into_iter()
        .map(|(slot, value)| {
            let value = match value {
                Replacement::Image(id) => {
                    TextureReplacement::Image(ImageResource::file_data_id(id))
                }
                Replacement::Atlas(atlas) => TextureReplacement::Atlas {
                    atlas: Atlas {
                        width: atlas.width,
                        height: atlas.height,
                        clear: atlas.clear,
                    },
                    layers: atlas
                        .layers
                        .into_iter()
                        .map(|layer| TextureLayer {
                            resource: ImageResource::file_data_id(layer.image),
                            source: layer.source.map(rect),
                            destination: rect(layer.destination),
                            opacity: layer.opacity,
                            blend: Default::default(),
                        })
                        .collect(),
                },
            };
            (slot, value)
        })
        .collect()
}
/// Loads FileDataID-named companions from `asset_root/filedata`, or the root itself.
/// This capture recipe deliberately accepts only chunked version 274 M2 documents.
/// Other native layouts continue through the ordinary capture asset loader.
pub fn load(
    recipe_path: &Path,
    asset_root: &Path,
    model_path: &Path,
    allow_loss: bool,
    images: &mut Assets<Image>,
) -> Result<CompiledModel> {
    let recipe: Recipe = serde_json::from_slice(&read(recipe_path)?)?;
    let model_ids: Vec<_> = [recipe.body]
        .into_iter()
        .chain(recipe.parts.iter().map(|part| part.model))
        .collect();
    let parts = recipe
        .parts
        .into_iter()
        .map(|part| ModelPart {
            clips: Default::default(),
            model: ResourceLocator::FileDataId(part.model),
            binding: part
                .attachment
                .map_or(EquipmentBinding::SharedSkeleton, |role| {
                    EquipmentBinding::Attachment(SocketIdentity::Role(role))
                }),
            joints: BTreeMap::new(),
            retain_unmatched_joints: false,
            geosets: part.geosets.map(|visible| GeosetSelection { visible }),
            textures: replacements(part.textures),
        })
        .collect::<Vec<_>>();
    eprintln!(
        "Assembling character {} with {} model parts",
        recipe.body,
        parts.len()
    );
    let definition = CharacterDefinition {
        bone_sets: Default::default(),
        model: ResourceLocator::FileDataId(recipe.body),
        geosets: GeosetSelection {
            visible: recipe.geosets,
        },
        textures: replacements(recipe.textures),
        parts,
    };
    definition.validate()?;
    let directory = if asset_root.join("filedata").is_dir() {
        asset_root.join("filedata")
    } else {
        asset_root.to_owned()
    };
    let mut buffers = BTreeMap::new();
    for entry in read_dir(&directory)? {
        let path = entry?.path();
        // Texture pixels are loaded only if the compiled appearance needs them.
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("m2" | "skin" | "skel" | "bone" | "anim" | "phys")
        ) {
            continue;
        }
        if let Some(id) = path
            .file_stem()
            .and_then(|value| value.to_str())
            .and_then(|value| value.parse::<u32>().ok())
        {
            if buffers.insert(FileKey::FileId(id), read(&path)?).is_some() {
                return Err(format!(
                    "duplicate native FileDataID {id} in {}",
                    directory.display()
                )
                .into());
            }
        }
    }
    if buffers
        .get(&FileKey::FileId(recipe.body))
        .map(Vec::as_slice)
        != Some(read(model_path)?.as_slice())
    {
        return Err("positional model must match the character recipe body".into());
    }
    let bundles = model_ids
        .iter()
        .map(|id| {
            let key = FileKey::FileId(*id);
            let bytes = buffers
                .get(&key)
                .ok_or_else(|| format!("missing character model FileDataID {id}"))?;
            Bundle::<Chunked<V274>>::decode(bytes, key, |dependency| {
                if !buffers.contains_key(&dependency.key) {
                    eprintln!("Missing character companion: {:?}", dependency.key);
                }
                Ok(buffers.get(&dependency.key).map(Vec::as_slice))
            })
            .map_err(|error| format!("character model {id}: {error}").into())
        })
        .collect::<Result<Vec<_>>>()?;
    let options = if allow_loss {
        ImportOptions {
            loss_policy: LossPolicy::Drop,
            unknown: UnknownPolicy::Drop,
            ..Default::default()
        }
    } else {
        ImportOptions::default()
    };
    let converted = bundles
        .iter()
        .zip(&model_ids)
        .map(|(bundle, id)| {
            semantic_m2::import(bundle, &options)
                .map_err(|error| format!("character model {id} conversion: {error}").into())
        })
        .collect::<Result<Vec<_>>>()?;
    let mut models = Models::new();
    for (id, converted) in model_ids.into_iter().zip(converted) {
        for loss in &converted.report.issues {
            eprintln!("Character import {id}: {loss:?}");
        }
        models.insert(ResourceLocator::FileDataId(id), converted.value.widen());
    }
    Ok(compile_character(
        &definition,
        &models,
        images,
        |resource| {
            let ImageResource::FileDataId { id, .. } = resource else {
                return Err(ModelError::new(format!(
                    "character image must have a FileDataID: {resource:?}"
                )));
            };
            let result = || -> Result<RgbaImage> {
                let bytes = read(directory.join(format!("{id}.blp")))?;
                let image = Blp::read(&bytes)?.decode_mip(0)?;
                Ok(RgbaImage::new(
                    image.width(),
                    image.height(),
                    image.into_raw(),
                )?)
            };
            result().map_err(|error| ModelError::new(format!("character image {id}: {error}")))
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_distinguishes_shared_armor_and_rigid_mounts() {
        let recipe: Recipe = serde_json::from_str(
            r#"{
            "body": 1, "geosets": [0, 401],
            "textures": {"1": {"width": 16, "height": 8, "layers": [
                {"image": 7, "destination": [0, 0, 16, 8]}
            ]}},
            "parts": [
                {"model": 2, "slot": "chest"},
                {"model": 3, "slot": "head", "attachment": 11}
            ]
        }"#,
        )
        .unwrap();
        assert_eq!(recipe.parts[0].attachment, None);
        assert_eq!(recipe.parts[1].attachment, Some(11));
        let replacements = replacements(recipe.textures);
        let TextureReplacement::Atlas { atlas, layers } = &replacements[&1] else {
            panic!("expected composed skin");
        };
        assert_eq!((atlas.width, atlas.height), (16, 8));
        assert_eq!(layers[0].opacity, 255);
    }

    #[test]
    fn recipe_rejects_unknown_fields() {
        assert!(serde_json::from_str::<Recipe>(r#"{"body":1,"geosets":[],"geoset":401}"#).is_err());
        assert!(serde_json::from_str::<Part>(r#"{"model":2,"inventory_slot":"head"}"#).is_err());
    }

    #[test]
    fn part_labels_are_optional_and_do_not_restrict_rendered_parts() {
        let unlabeled: Part = serde_json::from_str(r#"{"model":2}"#).unwrap();
        let labeled: Part = serde_json::from_str(r#"{"model":3,"slot":"antenna"}"#).unwrap();
        assert_eq!(unlabeled._label, None);
        assert_eq!(labeled._label.as_deref(), Some("antenna"));
    }
}
