//! Independent world-surface fields and borrowed external texture identities.
use super::encode::append;
use crate::model::conversion::Error;
use crate::model::error::Location;
use crate::model::formats::wmo::{self as native, Dialect};
use crate::model::materials::bindings::TextureBinding;
use crate::model::materials::state::{Lighting, PassState};
use crate::model::materials::wmo::RasterizationOptions;
use crate::model::materials::wmo::WorldBlend;
use crate::model::profiles::wmo as profile;
use crate::model::profiles::Wmo;
use crate::model::Model;
use crate::model::WorldImageResource;
use crate::model::{
    AddressMode, ImageResource, ImageResourceId, Material, MaterialId, ModelBuilder, Sampler,
};
use glam::Vec2;
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
        .map(|row| {
            row.map_err(|error| {
                Error::from_source(Location::new().field("world").field("material"), error)
            })
        })
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
    let mut textures = Vec::<(ImageResource<'a>, ImageResourceId)>::new();
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
                        let image_resource =
                            builder.add_image_resource(resource.clone().try_into()?);
                        let id = image_resource;
                        textures.push((resource, id));
                        id
                    },
                )
            } else {
                None
            };
            bindings.push(TextureBinding {
                image,
                sampling: Sampler {
                    u: if row.flags.clamp_s() {
                        AddressMode::Clamp
                    } else {
                        AddressMode::Repeat
                    },
                    v: if row.flags.clamp_t() {
                        AddressMode::Clamp
                    } else {
                        AddressMode::Repeat
                    },
                },
                ..Default::default()
            });
        }
        let flags = row.flags;
        let tail = row.tail.texture_slots();
        let extended = matches!(row.shader, 22..=24);
        let surface = profile::MaterialStep::from_native(
            Some(row.shader),
            profile::MaterialDeclaration {
                self_illumination: color(row.sidn_color),
                runtime_self_illumination: color(row.frame_sidn_color),
                diffuse: color(row.diffuse_color),
                ground_type: row.ground_type,
                uv_scroll: scroll
                    .get(index)
                    .map_or([Vec2::from_array([0.; 2]); 2], |value| {
                        [
                            Vec2::from_array(value.speed0),
                            Vec2::from_array(value.speed1),
                        ]
                    }),
                shader_overrides: extensions.get(index).map(|value| profile::ShaderOverrides {
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
                        BlendMode::ModulateAdd => {
                            profile::Blend::Extension(WorldBlend::ModulateAdd)
                        }
                        BlendMode::InverseSourceAlphaAdd => {
                            profile::Blend::Extension(WorldBlend::InverseSourceAlphaAdd)
                        }
                        BlendMode::Unknown(value) => profile::Blend::Unknown(value),
                    },
                    rasterization: profile::MaterialRasterization {
                        depth: (),
                        faces: flags.two_sided(),
                        lighting: Lighting {
                            unlit: flags.unlit(),
                            unfogged: flags.unfogged(),
                        },
                        controls: RasterizationOptions {
                            exterior_lit: flags.exterior_lit(),
                            self_illuminated: false,
                        },
                    },
                    ordering: (),
                },
                ..Default::default()
            },
            bindings,
        );
        output.push(builder.add_material(Material {
            name: Cow::Owned(format!("Material {index}")),
            surface,
        })?);
    }
    Ok(output)
}

pub(super) fn write<'a, D: Dialect>(
    model: &Model<'_, Wmo>,
    root: &mut impl super::root_output::RootOutput<'a, D>,
    context: &mut crate::model::conversion::Context,
) -> Result<std::collections::BTreeMap<MaterialId, u16>, Error> {
    let mut names = vec![0u8];
    let mut named = false;
    let mut file_ids = false;
    let mut selectors = std::collections::BTreeMap::new();
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
        .collect::<std::collections::BTreeSet<_>>();
    for id in used {
        let value = match model.image_resource(id).expect("validated image") {
            WorldImageResource::Path(path) => {
                named = true;
                let offset = u32::try_from(names.len()).map_err(|_| {
                    Error::requirement(
                        Location::new().field("world").field("texture"),
                        "name table exceeds capacity",
                    )
                })?;
                if path.contains(&0) {
                    return Err(Error::requirement(
                        Location::new().field("world").field("texture"),
                        "resource path contains NUL",
                    ));
                }
                names.extend_from_slice(path);
                names.push(0);
                offset
            }
            WorldImageResource::FileDataId(id) => match root.addressing() {
                super::root_output::Addressing::PathsOrFileIds => {
                    file_ids = true;
                    *id
                }
                super::root_output::Addressing::Paths => {
                    context.unsupported(
                        Location::new().field("world").field("texture"),
                        "historical material textures require model file paths",
                    )?;
                    0
                }
            },
            _ => {
                context.unsupported(
                    Location::new().field("world").field("texture"),
                    "world-model textures require paths or game file identifiers",
                )?;
                0
            }
        };
        selectors.insert(id, value);
    }
    if named && file_ids {
        return Err(Error::requirement(
            Location::new().field("world").field("textures"),
            "one native root cannot mix path-addressed and database-addressed material textures",
        ));
    }
    let mut materials = Vec::new();
    let mut mapping = std::collections::BTreeMap::new();
    let mut scrolls = Vec::new();
    let mut extensions = Vec::new();
    for (id, material) in model.materials() {
        let step = &material.surface;
        let source = step.declaration();
        let mut row = native::records::Material::<D> {
            shader: if let Some(shader) = step.native_shader() {
                shader
            } else {
                context.unsupported(
                    Location::new()
                        .field("world")
                        .field("material")
                        .field("shader"),
                    "selected native material requires an explicit shader-table selection",
                )?;
                0
            },
            blend_mode: native_blend(source.state.blend),
            flags: native_flags(source.state.rasterization, &material.surface.inputs())?,
            ..Default::default()
        };
        if source.state.rasterization.controls.self_illuminated {
            context.unsupported(
                Location::new()
                    .field("world")
                    .field("material")
                    .field("self_illuminated"),
                "no established native self-illumination switch is available",
            )?;
        }
        row.sidn_color = color(source.self_illumination);
        row.frame_sidn_color = color(source.runtime_self_illumination);
        row.diffuse_color = color(source.diffuse);
        row.ground_type = source.ground_type;
        for (slot, texture) in step.inputs().into_iter().enumerate() {
            row.set_texture(slot, texture.image.map_or(0, |id| selectors[&id]))
                .map_err(|error| {
                    Error::from_source(
                        Location::new()
                            .field("world")
                            .field("material")
                            .field("texture"),
                        error,
                    )
                })?;
        }
        if let Some(value) = source.legacy_color {
            row.tail
                .set_texture(0, u32::from_le_bytes(color(value)))
                .map_err(|error| {
                    Error::from_source(
                        Location::new()
                            .field("world")
                            .field("material")
                            .field("color"),
                        error,
                    )
                })?;
        }
        if let Some(value) = source.legacy_controls {
            row.tail.set_texture(1, value).map_err(|error| {
                Error::from_source(
                    Location::new()
                        .field("world")
                        .field("material")
                        .field("controls"),
                    error,
                )
            })?;
        }
        if let Some(values) = source.legacy_runtime {
            for (slot, value) in values.into_iter().enumerate() {
                row.tail.set_texture(slot + 2, value).map_err(|error| {
                    Error::from_source(
                        Location::new()
                            .field("world")
                            .field("material")
                            .field("runtime"),
                        error,
                    )
                })?;
            }
        }
        scrolls.push(native::records::MaterialUvAnimation {
            speed0: (source.uv_scroll[0]).to_array(),
            speed1: (source.uv_scroll[1]).to_array(),
        });
        extensions.push(source.shader_overrides.as_ref().map_or_else(
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
            u16::try_from(materials.len()).map_err(|_| {
                Error::requirement(
                    Location::new().field("world").field("materials"),
                    "material count exceeds capacity",
                )
            })?,
        );
        materials.push(row);
    }
    if !file_ids {
        append(
            root,
            RootChunk::TextureNames(native::codec::Array::from_records(&names).map_err(
                |error| {
                    Error::from_source(
                        Location::new()
                            .field("world")
                            .field("texture")
                            .field("names"),
                        error,
                    )
                },
            )?),
        )?;
    }
    append(
        root,
        RootChunk::Materials(
            native::codec::Array::from_records(&materials).map_err(|error| {
                Error::from_source(Location::new().field("world").field("materials"), error)
            })?,
        ),
    )?;
    if scrolls
        .iter()
        .any(|value| value.speed0 != [0.; 2] || value.speed1 != [0.; 2])
    {
        append(
            root,
            RootChunk::UvAnimations(native::codec::Array::from_records(&scrolls).map_err(
                |error| {
                    Error::from_source(
                        Location::new()
                            .field("world")
                            .field("material")
                            .field("scroll"),
                        error,
                    )
                },
            )?),
        )?;
    }
    if model
        .materials()
        .iter()
        .any(|(_, material)| material.surface.declaration().shader_overrides.is_some())
    {
        append(
            root,
            RootChunk::MaterialExtensions(
                native::codec::Array::from_records(&extensions).map_err(|error| {
                    Error::from_source(
                        Location::new()
                            .field("world")
                            .field("material")
                            .field("extension"),
                        error,
                    )
                })?,
            ),
        )?;
    }
    root.material_count(materials.len() as u32);
    Ok(mapping)
}

fn resource<'view, 'data: 'view>(
    source: &'view impl super::root_view::CommonRoot<'data>,
    selector: u32,
) -> Result<Option<ImageResource<'view>>, Error> {
    if let Some(names) = source.common_chunks().find_map(|chunk| match chunk {
        RootChunk::TextureNames(names) => Some(names.as_bytes()),
        _ => None,
    }) {
        let bytes = native::resources::string_at(names, selector).map_err(|error| {
            Error::from_source(Location::new().field("world").field("texture"), error)
        })?;
        Ok((!bytes.is_empty()).then_some(ImageResource::Path(Cow::Borrowed(bytes))))
    } else {
        Ok((selector != 0).then_some(ImageResource::file_data_id(selector)))
    }
}

pub(super) fn read_alpha<'view, 'data: 'view>(
    source: &'view native::legacy::Root14<'data>,
    builder: &mut ModelBuilder<'view, Wmo>,
) -> Result<Vec<MaterialId>, Error> {
    use native::legacy::RootChunk14;
    let mut result = Vec::new();
    let mut textures = Vec::<(ImageResource<'view>, ImageResourceId)>::new();
    for chunk in source.chunks() {
        if let RootChunk14::Materials(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(|error| {
                    Error::from_source(Location::new().field("world").field("material"), error)
                })?;
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
                            let image_resource =
                                builder.add_image_resource(resource.clone().try_into()?);
                            let id = image_resource;
                            textures.push((resource, id));
                            id
                        };
                        Some(id)
                    } else {
                        None
                    };
                    bindings.push(TextureBinding {
                        image: binding,
                        sampling: Sampler {
                            u: if row.flags.clamp_s() {
                                AddressMode::Clamp
                            } else {
                                AddressMode::Repeat
                            },
                            v: if row.flags.clamp_t() {
                                AddressMode::Clamp
                            } else {
                                AddressMode::Repeat
                            },
                        },
                        ..Default::default()
                    });
                }
                let flags = row.flags;
                let surface = profile::MaterialStep::from_native(
                    None,
                    profile::MaterialDeclaration {
                        self_illumination: color(row.sidn_color),
                        runtime_self_illumination: color(row.frame_sidn_color),
                        diffuse: color(row.diffuse_color),
                        ground_type: row.ground_type,
                        state: PassState {
                            blend: blend(row.blend_mode),
                            rasterization: profile::MaterialRasterization {
                                depth: (),
                                faces: flags.two_sided(),
                                lighting: Lighting {
                                    unlit: flags.unlit(),
                                    unfogged: flags.unfogged(),
                                },
                                controls: RasterizationOptions {
                                    exterior_lit: flags.exterior_lit(),
                                    ..Default::default()
                                },
                            },
                            ordering: (),
                        },
                        ..Default::default()
                    },
                    bindings,
                );
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
pub(super) fn native_flags(
    value: profile::MaterialRasterization,
    inputs: &[&profile::ImageInput],
) -> Result<native::records::MaterialFlags, Error> {
    let mut flags = native::records::MaterialFlags(0);
    flags.set_unlit(value.lighting.unlit);
    flags.set_unfogged(value.lighting.unfogged);
    flags.set_two_sided(value.faces);
    flags.set_exterior_lit(value.controls.exterior_lit);
    let mut samplers = inputs.iter().map(|input| input.sampling);
    let sampler = samplers.next().unwrap_or_default();
    if samplers.any(|other| other != sampler) {
        return Err(Error::requirement(
            Location::new()
                .field("world")
                .field("material")
                .field("sampler"),
            "WMO requires all image inputs in one material to share address modes",
        ));
    }
    flags.set_clamp_s(sampler.u == AddressMode::Clamp);
    flags.set_clamp_t(sampler.v == AddressMode::Clamp);
    Ok(flags)
}
