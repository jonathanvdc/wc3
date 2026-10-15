//! Independent world-surface fields and borrowed external texture identities.
use super::encode::append;
use crate::model::conversion::Error;
use crate::model::formats::wmo::{self as native, Dialect};
use crate::model::materials::bindings::TextureBinding;
use crate::model::materials::state::{Lighting, PassState};
use crate::model::materials::world::SurfaceOptions;
use crate::model::materials::world::WorldBlend;
use crate::model::profiles::wmo as profile;
use crate::model::profiles::Wmo;
use crate::model::{
    ColorSpace, Material, MaterialId, ModelBuilder, Resource, Sampler, Texture, TextureId,
};
use native::records::{BlendMode, MaterialTailField};
use native::root::RootChunk;
use std::borrow::Cow;

fn color([b, g, r, a]: [u8; 4]) -> [u8; 4] {
    [r, g, b, a]
}
fn records<T: for<'d> native::Read<'d> + native::RecordLayout>(
    rows: &native::codec::Array<'_, T>,
) -> Result<Vec<T>, Error> {
    rows.iter()
        .map(|row| row.map_err(|error| Error::new("world.material", error.to_string())))
        .collect()
}
pub(super) fn read<'a, 'data: 'a>(
    root: &'a impl super::root_view::CommonRoot<'data>,
    builder: &mut ModelBuilder<'a, Wmo>,
) -> Result<Vec<MaterialId>, Error> {
    let rows = root
        .common_chunks()
        .filter_map(|chunk| match chunk {
            RootChunk::Materials(rows) => Some(records(rows)),
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()?
        .concat();
    let scroll = root
        .common_chunks()
        .find_map(|chunk| match chunk {
            RootChunk::UvAnimations(rows) => Some(records(rows)),
            _ => None,
        })
        .transpose()?
        .unwrap_or_default();
    let extensions = root
        .common_chunks()
        .find_map(|chunk| match chunk {
            RootChunk::MaterialExtensions(rows) => Some(records(rows)),
            _ => None,
        })
        .transpose()?
        .unwrap_or_default();
    let mut output = Vec::new();
    let mut textures = Vec::<(Resource<'a>, TextureId)>::new();
    for (index, row) in rows.iter().enumerate() {
        let mut bindings = Vec::new();
        for slot in 0..row.texture_slots() {
            let resource = resource(root, row.textures()[slot])?;
            let image = if let Some(resource) = resource {
                Some(
                    if let Some((_, id)) =
                        textures.iter().find(|(existing, _)| *existing == resource)
                    {
                        *id
                    } else {
                        let id = builder.add_texture(Texture {
                            resource: resource.clone(),
                            controls: Default::default(),
                            color_space: ColorSpace::Unspecified,
                            sampler: Sampler::default(),
                        })?;
                        textures.push((resource, id));
                        id
                    },
                )
            } else {
                None
            };
            bindings.push(TextureBinding {
                image,
                ..Default::default()
            });
        }
        let flags = row.flags;
        let mut unclassified = flags;
        unclassified.set_unlit(false);
        unclassified.set_unfogged(false);
        unclassified.set_two_sided(false);
        unclassified.set_exterior_lit(false);
        unclassified.set_clamp_s(false);
        unclassified.set_clamp_t(false);
        let tail = row.tail.texture_slots();
        let extended = matches!(row.shader, 22..=24);
        let surface = profile::Material {
            shader: Some(row.shader),
            textures: bindings,
            self_illumination: color(row.sidn_color),
            runtime_self_illumination: color(row.frame_sidn_color),
            diffuse: color(row.diffuse_color),
            ground_type: row.ground_type,
            uv_scroll: scroll
                .get(index)
                .map_or([[0.; 2]; 2], |value| [value.speed0, value.speed1]),
            extension: extensions.get(index).map(|value| profile::Extension {
                color: color(value.color),
                texture_override: u64::from(value.override_key[0])
                    | (u64::from(value.override_key[1]) << 32),
                control: value.unknown,
            }),
            legacy_color: (!extended).then(|| color(tail[0].to_le_bytes())),
            legacy_controls: (!extended).then_some(tail[1]),
            legacy_runtime: (!extended)
                .then_some(tail[2..].try_into().expect("four tail controls")),
            state: PassState {
                blend: match row.blend_mode {
                    BlendMode::Opaque => profile::Blend::Opaque,
                    BlendMode::AlphaKey => profile::Blend::AlphaKey(()),
                    BlendMode::Alpha => profile::Blend::Alpha,
                    BlendMode::Add => profile::Blend::Additive,
                    BlendMode::Modulate => profile::Blend::Modulate,
                    BlendMode::Modulate2x => profile::Blend::Modulate2x,
                    BlendMode::ModulateAdd => profile::Blend::Extension(WorldBlend::ModulateAdd),
                    BlendMode::InverseSourceAlphaAdd => {
                        profile::Blend::Extension(WorldBlend::InverseSourceAlphaAdd)
                    }
                    BlendMode::Unknown(value) => profile::Blend::Unknown(value),
                },
                rasterization: profile::Options {
                    depth: (),
                    faces: flags.two_sided(),
                    lighting: Lighting {
                        unlit: flags.unlit(),
                        unfogged: flags.unfogged(),
                    },
                    controls: SurfaceOptions {
                        exterior_lit: flags.exterior_lit(),
                        clamp_u: flags.clamp_s(),
                        clamp_v: flags.clamp_t(),
                        self_illuminated: false,
                        unclassified: unclassified.bits().into(),
                    },
                },
                ordering: (),
            },
            ..Default::default()
        };
        output.push(builder.add_material(Material {
            name: Cow::Owned(format!("Material {index}")),
            surface,
        })?);
    }
    Ok(output)
}

pub(super) fn write<'a, D: Dialect>(
    model: &crate::model::Model<'_, Wmo>,
    resources: &crate::model::Resources<'_, Wmo>,
    root: &mut impl super::root_output::RootOutput<'a, D>,
    context: &mut crate::model::conversion::Context,
) -> Result<std::collections::BTreeMap<MaterialId, u16>, Error> {
    let mut names = vec![0u8];
    let mut named = false;
    let mut file_ids = false;
    let mut selectors = std::collections::BTreeMap::new();
    let used = model
        .materials()
        .flat_map(|(_, material)| {
            material
                .surface
                .textures
                .iter()
                .filter_map(|binding| binding.image)
        })
        .collect::<std::collections::BTreeSet<_>>();
    for (id, texture) in resources.textures() {
        if !used.contains(&id) {
            continue;
        }
        let value = match &texture.resource {
            Resource::Path(path) => {
                named = true;
                let offset = u32::try_from(names.len())
                    .map_err(|_| Error::new("world.texture", "name table exceeds capacity"))?;
                if path.contains(&0) {
                    return Err(Error::new("world.texture", "resource path contains NUL"));
                }
                names.extend_from_slice(path);
                names.push(0);
                offset
            }
            Resource::FileDataId(id) => match root.addressing() {
                super::root_output::Addressing::PathsOrFileIds => {
                    file_ids = true;
                    *id
                }
                super::root_output::Addressing::Paths => {
                    context.unsupported(
                        "world.texture",
                        "historical material textures require model file paths",
                    )?;
                    0
                }
            },
            _ => {
                context.unsupported(
                    "world.texture",
                    "world-model textures require paths or game file identifiers",
                )?;
                0
            }
        };
        selectors.insert(id, value);
    }
    if named && file_ids {
        return Err(Error::new(
            "world.textures",
            "one native root cannot mix path-addressed and database-addressed material textures",
        ));
    }
    let mut materials = Vec::new();
    let mut mapping = std::collections::BTreeMap::new();
    let mut scrolls = Vec::new();
    let mut extensions = Vec::new();
    for (id, material) in model.materials() {
        let source = &material.surface;
        let mut row = native::records::Material::<D> {
            shader: if let Some(shader) = source.shader {
                shader
            } else {
                context.unsupported(
                    "world.material.shader",
                    "selected native material requires an explicit shader-table selection",
                )?;
                0
            },
            blend_mode: native_blend(source.state.blend),
            flags: native_flags(source.state.rasterization),
            ..Default::default()
        };
        if source.state.rasterization.controls.self_illuminated {
            context.unsupported(
                "world.material.self_illuminated",
                "no established native self-illumination switch is available",
            )?;
        }
        row.sidn_color = color(source.self_illumination);
        row.frame_sidn_color = color(source.runtime_self_illumination);
        row.diffuse_color = color(source.diffuse);
        row.ground_type = source.ground_type;
        for (slot, texture) in source.textures.iter().enumerate() {
            row.set_texture(slot, texture.image.map_or(0, |id| selectors[&id]))
                .map_err(|error| Error::new("world.material.texture", error.to_string()))?;
        }
        if let Some(value) = source.legacy_color {
            row.tail
                .set_texture(0, u32::from_le_bytes(color(value)))
                .map_err(|error| Error::new("world.material.color", error.to_string()))?;
        }
        if let Some(value) = source.legacy_controls {
            row.tail
                .set_texture(1, value)
                .map_err(|error| Error::new("world.material.controls", error.to_string()))?;
        }
        if let Some(values) = source.legacy_runtime {
            for (slot, value) in values.into_iter().enumerate() {
                row.tail
                    .set_texture(slot + 2, value)
                    .map_err(|error| Error::new("world.material.runtime", error.to_string()))?;
            }
        }
        scrolls.push(native::records::MaterialUvAnimation {
            speed0: source.uv_scroll[0],
            speed1: source.uv_scroll[1],
        });
        extensions.push(source.extension.as_ref().map_or_else(
            native::records::MaterialExtension::default,
            |source| native::records::MaterialExtension {
                color: color(source.color),
                override_key: [
                    source.texture_override as u32,
                    (source.texture_override >> 32) as u32,
                ],
                unknown: source.control,
            },
        ));
        mapping.insert(
            id,
            u16::try_from(materials.len())
                .map_err(|_| Error::new("world.materials", "material count exceeds capacity"))?,
        );
        materials.push(row);
    }
    if !file_ids {
        append(
            root,
            RootChunk::TextureNames(
                native::codec::Array::from_records(&names)
                    .map_err(|error| Error::new("world.texture.names", error.to_string()))?,
            ),
        )?;
    }
    append(
        root,
        RootChunk::Materials(
            native::codec::Array::from_records(&materials)
                .map_err(|error| Error::new("world.materials", error.to_string()))?,
        ),
    )?;
    if scrolls
        .iter()
        .any(|value| value.speed0 != [0.; 2] || value.speed1 != [0.; 2])
    {
        append(
            root,
            RootChunk::UvAnimations(
                native::codec::Array::from_records(&scrolls)
                    .map_err(|error| Error::new("world.material.scroll", error.to_string()))?,
            ),
        )?;
    }
    if model
        .materials()
        .any(|(_, material)| material.surface.extension.is_some())
    {
        append(
            root,
            RootChunk::MaterialExtensions(
                native::codec::Array::from_records(&extensions)
                    .map_err(|error| Error::new("world.material.extension", error.to_string()))?,
            ),
        )?;
    }
    root.material_count(materials.len() as u32);
    Ok(mapping)
}

fn resource<'view, 'data: 'view>(
    source: &'view impl super::root_view::CommonRoot<'data>,
    selector: u32,
) -> Result<Option<Resource<'view>>, Error> {
    if let Some(names) = source.common_chunks().find_map(|chunk| match chunk {
        RootChunk::TextureNames(names) => Some(names.as_bytes()),
        _ => None,
    }) {
        let bytes = native::resources::string_at(names, selector)
            .map_err(|error| Error::new("world.texture", error.to_string()))?;
        Ok((!bytes.is_empty()).then_some(Resource::Path(Cow::Borrowed(bytes))))
    } else {
        Ok((selector != 0).then_some(Resource::FileDataId(selector)))
    }
}

pub(super) fn read_alpha<'view, 'data: 'view>(
    source: &'view native::legacy::Root14<'data>,
    builder: &mut ModelBuilder<'view, Wmo>,
) -> Result<Vec<MaterialId>, Error> {
    use native::legacy::RootChunk14;
    let mut result = Vec::new();
    let mut textures = Vec::<(Resource<'view>, TextureId)>::new();
    for chunk in source.chunks() {
        if let RootChunk14::Materials(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(|error| Error::new("world.material", error.to_string()))?;
                let mut bindings = Vec::new();
                for selector in [row.texture0, row.texture1] {
                    let resource = resource(source, selector)?;
                    let binding = if let Some(resource) = resource {
                        let id = if let Some((_, id)) = textures
                            .iter()
                            .find(|(candidate, _)| *candidate == resource)
                        {
                            *id
                        } else {
                            let id = builder.add_texture(Texture {
                                resource: resource.clone(),
                                controls: Default::default(),
                                color_space: ColorSpace::Unspecified,
                                sampler: Sampler::default(),
                            })?;
                            textures.push((resource, id));
                            id
                        };
                        Some(id)
                    } else {
                        None
                    };
                    bindings.push(TextureBinding {
                        image: binding,
                        ..Default::default()
                    });
                }
                let flags = row.flags;
                let mut unclassified = flags;
                unclassified.set_unlit(false);
                unclassified.set_unfogged(false);
                unclassified.set_two_sided(false);
                unclassified.set_exterior_lit(false);
                unclassified.set_clamp_s(false);
                unclassified.set_clamp_t(false);
                let surface = profile::Material {
                    shader: None,
                    textures: bindings,
                    self_illumination: color(row.sidn_color),
                    runtime_self_illumination: color(row.frame_sidn_color),
                    diffuse: color(row.diffuse_color),
                    ground_type: row.ground_type,
                    state: PassState {
                        blend: blend(row.blend_mode),
                        rasterization: profile::Options {
                            depth: (),
                            faces: flags.two_sided(),
                            lighting: Lighting {
                                unlit: flags.unlit(),
                                unfogged: flags.unfogged(),
                            },
                            controls: SurfaceOptions {
                                exterior_lit: flags.exterior_lit(),
                                clamp_u: flags.clamp_s(),
                                clamp_v: flags.clamp_t(),
                                unclassified: unclassified.bits().into(),
                                ..Default::default()
                            },
                        },
                        ordering: (),
                    },
                    ..Default::default()
                };
                result.push(builder.add_material(Material {
                    name: Cow::Owned(format!("Material {}", result.len())),
                    surface,
                })?);
            }
        }
    }
    Ok(result)
}
fn blend(value: BlendMode) -> profile::Blend {
    match value {
        BlendMode::Opaque => profile::Blend::Opaque,
        BlendMode::AlphaKey => profile::Blend::AlphaKey(()),
        BlendMode::Alpha => profile::Blend::Alpha,
        BlendMode::Add => profile::Blend::Additive,
        BlendMode::Modulate => profile::Blend::Modulate,
        BlendMode::Modulate2x => profile::Blend::Modulate2x,
        BlendMode::ModulateAdd => profile::Blend::Extension(WorldBlend::ModulateAdd),
        BlendMode::InverseSourceAlphaAdd => {
            profile::Blend::Extension(WorldBlend::InverseSourceAlphaAdd)
        }
        BlendMode::Unknown(value) => profile::Blend::Unknown(value),
    }
}

pub(super) fn native_blend(value: profile::Blend) -> BlendMode {
    match value {
        profile::Blend::Opaque => BlendMode::Opaque,
        profile::Blend::AlphaKey(()) => BlendMode::AlphaKey,
        profile::Blend::Alpha => BlendMode::Alpha,
        profile::Blend::Additive => BlendMode::Add,
        profile::Blend::Modulate => BlendMode::Modulate,
        profile::Blend::Modulate2x => BlendMode::Modulate2x,
        profile::Blend::Extension(WorldBlend::ModulateAdd) => BlendMode::ModulateAdd,
        profile::Blend::Extension(WorldBlend::InverseSourceAlphaAdd) => {
            BlendMode::InverseSourceAlphaAdd
        }
        profile::Blend::Unknown(value) => BlendMode::Unknown(value),
        profile::Blend::AdditiveAlpha(value) => match value {},
    }
}
pub(super) fn native_flags(value: profile::Options) -> native::records::MaterialFlags {
    let mut flags = native::records::MaterialFlags(value.controls.unclassified.bits());
    flags.set_unlit(value.lighting.unlit);
    flags.set_unfogged(value.lighting.unfogged);
    flags.set_two_sided(value.faces);
    flags.set_exterior_lit(value.controls.exterior_lit);
    flags.set_clamp_s(value.controls.clamp_u);
    flags.set_clamp_t(value.controls.clamp_v);
    flags
}
