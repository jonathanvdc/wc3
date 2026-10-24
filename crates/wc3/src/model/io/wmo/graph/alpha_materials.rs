//! Fixed-function material records for monolithic historical roots.
use super::encode::array;
use super::materials::{native_blend, native_flags};
use crate::model::conversion::{Context, Error};
use crate::model::error::Location;
use crate::model::formats::wmo::legacy::{Material14, Root14, RootChunk14};
use crate::model::formats::wmo::root::RootChunk;
use crate::model::profiles::Wmo;
use crate::model::WorldImageResource;
use crate::model::{MaterialId, Model};
use glam::Vec2;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error as StdError;
pub(super) fn write<'a>(
    model: &Model<'_, Wmo>,
    output: &mut Root14<'a>,
    context: &mut Context,
) -> Result<BTreeMap<MaterialId, u16>, Error> {
    let used = model
        .materials()
        .iter()
        .flat_map(|(_, material)| {
            material
                .surface
                .inputs()
                .into_iter()
                .filter_map(|binding| binding.image)
        })
        .collect::<BTreeSet<_>>();
    let mut names = vec![0];
    let mut textures = BTreeMap::new();
    for id in used {
        let selector = match model.image_resource(id).expect("validated image") {
            WorldImageResource::Path(path) => {
                if path.contains(&0) {
                    return Err(Error::requirement(
                        Location::new().field("world").field("texture"),
                        "texture path contains an interior terminator",
                    ));
                }
                let index = u32::try_from(names.len()).map_err(|_| {
                    Error::requirement(
                        Location::new().field("world").field("texture"),
                        "texture-name table exceeds address space",
                    )
                })?;
                names.extend_from_slice(path);
                names.push(0);
                index
            }
            _ => {
                context.unsupported(
                    Location::new().field("world").field("texture"),
                    "historical material textures require file paths",
                )?;
                0
            }
        };
        textures.insert(id, selector);
    }
    let mut records = Vec::new();
    let mut mapping = BTreeMap::new();
    for (id, material) in model.materials() {
        let step = &material.surface;
        let surface = step.declaration();
        let inputs = step.inputs();
        if step.native_shader().is_some()
            || inputs.len() > 2
            || surface.shader_overrides.is_some()
            || surface.legacy_color.is_some()
            || surface.legacy_controls.is_some()
            || surface.legacy_runtime.is_some()
            || surface.uv_scroll.iter().any(|value| *value != Vec2::ZERO)
        {
            context.unsupported(Location::new().field("world").field("material"), "historical fixed-function materials have two texture slots and no shader selection, scroll curves, or later shader controls")?;
        }
        if surface.state.rasterization.controls.self_illuminated {
            context.unsupported(
                Location::new()
                    .field("world")
                    .field("material")
                    .field("self_illuminated"),
                "no established native self-illumination switch is available",
            )?;
        }
        let texture = |slot: usize| {
            inputs
                .get(slot)
                .and_then(|binding| binding.image)
                .map_or(0, |id| textures[&id])
        };
        let record = Material14 {
            flags: native_flags(surface.state.rasterization, &material.surface.inputs())?,
            blend_mode: native_blend(surface.state.blend),
            texture0: texture(0),
            texture1: texture(1),
            sidn_color: super::rgba(surface.self_illumination),
            frame_sidn_color: super::rgba(surface.runtime_self_illumination),
            diffuse_color: super::rgba(surface.diffuse),
            ground_type: surface.ground_type,
            ..Default::default()
        };
        mapping.insert(
            id,
            u16::try_from(records.len()).map_err(|_| {
                Error::requirement(
                    Location::new().field("world").field("material"),
                    "material count exceeds native selection space",
                )
            })?,
        );
        records.push(record);
    }
    output
        .push_chunk(RootChunk14::Common(RootChunk::TextureNames(array(&names)?)))
        .map_err(write_error)?;
    output
        .push_chunk(RootChunk14::Materials(array(&records)?))
        .map_err(write_error)?;
    output.header_mut().texture_count = u32::try_from(records.len()).map_err(|_| {
        Error::requirement(
            Location::new().field("world").field("material"),
            "material count exceeds header capacity",
        )
    })?;
    Ok(mapping)
}
fn write_error(error: impl StdError + Send + Sync + 'static) -> Error {
    Error::from_source(Location::new().field("world").field("material"), error)
}
