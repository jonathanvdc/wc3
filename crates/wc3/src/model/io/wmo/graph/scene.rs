//! World placements and illumination as independent attached features.
use super::rgba;
use crate::model::conversion::{Context, Error};
use crate::model::formats::wmo::records::{FlickerMode, LightKind, NewLightKind};
use crate::model::formats::wmo::root::RootChunk;
use crate::model::profiles::wmo as profile;
use crate::model::profiles::Wmo;
use crate::model::WorldImageResource;
use crate::model::WorldPlacementAppearance;
use crate::model::{ColorSpace, ImageResource, Sampler, Texture};
use crate::model::{FeatureId, ModelBuilder, Quat};
use glam::{Vec2, Vec3, Vec4};
use std::borrow::Cow;

pub(super) struct Identities {
    pub lights: Vec<FeatureId>,
    pub new_lights: Vec<FeatureId>,
    pub doodads: Vec<FeatureId>,
}
pub(super) fn read<'a, 'data: 'a>(
    source: &'a impl super::root_view::CommonRoot<'data>,
    builder: &mut ModelBuilder<'a, Wmo>,
    world: &mut profile::WorldSystem<'a>,
    owner: crate::model::SystemId,
    context: &mut Context,
) -> Result<Identities, Error> {
    let mut ids = Identities {
        lights: Vec::new(),
        new_lights: Vec::new(),
        doodads: Vec::new(),
    };
    for (ordinal, row) in source
        .lights()
        .map_err(|error| Error::new("world.light", error.to_string()))?
        .into_iter()
        .enumerate()
    {
        let node = builder.new_node(format!("Light {ordinal}"));
        builder
            .values_mut()
            .set(node.transform.translation, Vec3::from_array(row.position))?;
        builder
            .values_mut()
            .set(node.transform.rotation, Quat::from_array(row.rotation))?;
        let id = builder.add_node(node)?;
        let feature_id = {
            let node = id;
            let payload = profile::Node::Light(Box::new(profile::Light {
                kind: match row.kind {
                    LightKind::Omni => profile::LightKind::Point,
                    LightKind::Spot => profile::LightKind::Spot,
                    LightKind::Directional => profile::LightKind::Directional,
                    LightKind::Ambient => profile::LightKind::Ambient,
                    LightKind::Unknown(value) => profile::LightKind::Unknown(i32::from(value)),
                },
                color: builder.add_value(Vec4::from_array(
                    rgba(row.color).map(|value| f32::from(value) / 255.),
                )),
                intensity: builder.add_value(row.intensity),
                use_attenuation: row.use_attenuation != 0,
                attenuation: Vec2::from_array([row.attenuation_start, row.attenuation_end]),
                controls: u32::from_le_bytes([row.padding[0], row.padding[1], 0, 0]).into(),
                cone: Vec2::from_array([0.; 2]),
                modern: None,
                directional_terms: None,
            }));
            builder.add_feature(node, payload)
        }?;
        ids.lights.push(feature_id);
    }
    for chunk in source.common_chunks() {
        if let RootChunk::NewLights(rows) = chunk {
            for row in rows.iter() {
                let row =
                    row.map_err(|error| Error::new("world.modern_light", error.to_string()))?;
                let node = builder.new_node(format!("Modern light {}", row.light_index));
                builder
                    .values_mut()
                    .set(node.transform.translation, Vec3::from_array(row.position))?;
                let id = builder.add_node(node)?;
                let cookie = if row.cookie_file_id == 0 {
                    None
                } else {
                    let resource = builder
                        .add_image_resource(WorldImageResource::FileDataId(row.cookie_file_id));
                    Some(Texture {
                        resource,
                        sampler: Sampler::default(),
                        color_space: ColorSpace::Unspecified,
                    })
                };
                let mut flags = row.flags;
                flags.set_blend_colors(false);
                flags.set_cast_shadows(false);
                let feature_id = {
                    let node = id;
                    let payload = profile::Node::Light(Box::new(profile::Light {
                        kind: match row.kind {
                            NewLightKind::Point => profile::LightKind::Point,
                            NewLightKind::Spot => profile::LightKind::Spot,
                            NewLightKind::Unknown(value) => profile::LightKind::Unknown(value),
                        },
                        color: builder.add_value(Vec4::from_array(
                            rgba(row.inner_color).map(|value| f32::from(value) / 255.),
                        )),
                        intensity: builder.add_value(row.intensity),
                        use_attenuation: true,
                        attenuation: Vec2::from_array([row.attenuation_start, row.attenuation_end]),
                        cone: Vec2::from_array([row.inner_angle, row.outer_angle]),
                        controls: (flags.bits() as u32).into(),
                        modern: Some(profile::ModernLight {
                            orientation: Vec3::from_array(row.rotation),
                            id: row.light_index,
                            doodad_set: if row.doodad_set == 0 {
                                None
                            } else {
                                Some(profile::DoodadSetId {
                                    system: owner,
                                    index: u32::try_from(row.doodad_set).map_err(|_| {
                                        Error::new("world.light.set", "negative set selection")
                                    })?,
                                })
                            },
                            blend_colors: row.flags.blend_colors(),
                            shadows: row.flags.cast_shadows(),
                            outer_color: rgba(row.outer_color),
                            color_blend: Vec2::from_array([row.blend_start, row.blend_end]),
                            flicker_parameters: Vec2::from_array([
                                row.flicker_intensity,
                                row.flicker_speed,
                            ]),
                            flicker: match row.flicker_mode {
                                FlickerMode::Off => profile::Flicker::Off,
                                FlickerMode::Sine => profile::Flicker::Sine,
                                FlickerMode::NoiseCurve => profile::Flicker::Noise,
                                FlickerMode::NoiseStep => profile::Flicker::SteppedNoise,
                                FlickerMode::Unknown(value) => profile::Flicker::Unknown(value),
                            },
                            cookie,
                            falloff: row.falloff,
                            scale: half(row.scale_half),
                            intensity_multiplier: half(row.intensity_multiplier_half),
                            vector_control: Vec3::from_array(row.unknown_vector),
                            controls: [
                                row.gap0,
                                row.gap1,
                                row.gap2[0],
                                row.gap2[1],
                                row.gap2[2],
                                row.gap2[3],
                                row.gap2[4],
                            ],
                            trailing_controls: row.unused,
                        }),
                        directional_terms: None,
                    }));
                    builder.add_feature(node, payload)
                }?;
                ids.new_lights.push(feature_id);
            }
        }
    }
    for chunk in source.common_chunks() {
        if let RootChunk::LightExtensions(rows) = chunk {
            for row in rows.iter() {
                let row =
                    row.map_err(|error| Error::new("world.light.extension", error.to_string()))?;
                let node = *ids
                    .lights
                    .get(usize::from(row.light_index))
                    .ok_or_else(|| {
                        Error::new("world.light.extension", "extension references absent light")
                    })?;
                let id = node;
                let payload = &builder
                    .model()
                    .feature(id)
                    .expect("light feature exists")
                    .payload;
                let mut payload = payload.clone();
                if let profile::Node::Light(light) = &mut payload {
                    light.directional_terms = Some(
                        row.terms
                            .map(|term| (Vec3::from_array(term.direction), term.value)),
                    );
                }
                builder.set_feature(id, payload)?;
            }
        }
    }
    let paths = source
        .common_chunks()
        .find_map(|chunk| {
            if let RootChunk::DoodadNames(rows) = chunk {
                Some(rows.as_bytes())
            } else {
                None
            }
        })
        .unwrap_or_default();
    let files = source.common_chunks().find_map(|chunk| {
        if let RootChunk::DoodadFileIds(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    });
    for chunk in source.common_chunks() {
        if let RootChunk::Doodads(rows) = chunk {
            for (ordinal, row) in rows.iter().enumerate() {
                let row = row.map_err(|error| Error::new("world.doodad", error.to_string()))?;
                let selector = row.name_and_flags.name_index();
                let file_id = files
                    .map(|files| {
                        files
                            .get(selector as usize)
                            .ok_or_else(|| Error::new("world.doodad", "file selector is absent"))?
                            .map_err(|error| Error::new("world.doodad", error.to_string()))
                    })
                    .transpose()?;
                let resource = if let Some(file) = file_id {
                    ImageResource::file_data_id(file)
                } else {
                    let bytes = crate::model::formats::wmo::resources::string_at(paths, selector)
                        .map_err(|error| Error::new("world.doodad", error.to_string()))?;
                    ImageResource::Path(Cow::Borrowed(bytes))
                };
                let node = builder.new_node(format!("Doodad {ordinal}"));
                builder
                    .values_mut()
                    .set(node.transform.translation, Vec3::from_array(row.position))?;
                builder
                    .values_mut()
                    .set(node.transform.rotation, Quat::from_array(row.rotation))?;
                builder
                    .values_mut()
                    .set(node.transform.scale, Vec3::from_array([row.scale; 3]))?;
                let id = builder.add_node(node)?;
                let flags = row.name_and_flags.flags();
                let light = if flags.has_light_reference() {
                    Some(*ids.lights.get(usize::from(row.color[3])).ok_or_else(|| {
                        Error::new("world.doodad.light", "doodad references absent light")
                    })?)
                } else {
                    None
                };
                let resource = builder.add_model_resource(resource.try_into()?);
                let feature_id = builder.add_feature(
                    id,
                    profile::Node::Attachment(profile::Doodad {
                        resource,
                        socket: (),
                        appearance: WorldPlacementAppearance {
                            color_multiplier: None,
                            tint: light.map_or_else(
                                || profile::DoodadTint::Color(rgba(row.color)),
                                |light| profile::DoodadTint::Lit {
                                    color: [row.color[2], row.color[1], row.color[0]],
                                    light,
                                },
                            ),
                            controls: {
                                let mut flags = flags;
                                flags.set_has_light_reference(false);
                                flags.bits().into()
                            },
                        },
                    }),
                )?;
                ids.doodads.push(feature_id);
            }
        }
    }
    for chunk in source.common_chunks() {
        if let RootChunk::DoodadColorMultipliers(rows) = chunk {
            if rows.len() != ids.doodads.len() {
                return Err(Error::new(
                    "world.doodad.color_multiplier",
                    "multiplier table must match placements",
                ));
            }
            for (id, value) in ids.doodads.iter().zip(rows.iter()) {
                let mut payload = builder
                    .model()
                    .feature(*id)
                    .expect("doodad exists")
                    .payload
                    .clone();
                if let profile::Node::Attachment(source) = &mut payload {
                    source.appearance.color_multiplier = Some(value.map_err(|error| {
                        Error::new("world.doodad.color_multiplier", error.to_string())
                    })?);
                }
                builder.set_feature(*id, payload)?;
            }
        }
    }
    for chunk in source.common_chunks() {
        if let RootChunk::DoodadSets(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(|error| Error::new("world.doodad_set", error.to_string()))?;
                let end = row.start_index as usize + row.count as usize;
                let nodes = ids
                    .doodads
                    .get(row.start_index as usize..end)
                    .ok_or_else(|| Error::new("world.doodad_set", "set range exceeds placements"))?
                    .to_vec();
                let text = &row.name[..row
                    .name
                    .iter()
                    .position(|value| *value == 0)
                    .unwrap_or(row.name.len())];
                world.doodad_sets.push(profile::DoodadSet {
                    name: Cow::Owned(
                        std::str::from_utf8(text)
                            .map_err(|error| {
                                Error::new("world.doodad_set.name", error.to_string())
                            })?
                            .to_owned(),
                    ),
                    members: nodes,
                    control: row.padding,
                });
            }
        }
    }
    let _ = context;
    Ok(ids)
}

pub(super) fn half(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000) << 16;
    let exponent = (bits >> 10) & 31;
    let fraction = u32::from(bits & 1023);
    match exponent {
        0 => {
            if fraction == 0 {
                f32::from_bits(sign)
            } else {
                let value = (fraction as f32) * 2f32.powi(-24);
                if sign == 0 {
                    value
                } else {
                    -value
                }
            }
        }
        31 => f32::from_bits(sign | 0x7f800000 | (fraction << 13)),
        _ => f32::from_bits(sign | ((u32::from(exponent) + 112) << 23) | (fraction << 13)),
    }
}
