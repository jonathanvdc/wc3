//! Region-local lights and activation membership independent of native table offsets.

use super::rgba;
use crate::model::conversion::Error;
use crate::model::formats::wmo::group::GroupChunk;
use crate::model::formats::wmo::records;
use crate::model::formats::wmo::{Dialect, Group};
use crate::model::profiles::wmo::{self, RegionControls};
use crate::model::profiles::Wmo;
use crate::model::PropertyValues;
use crate::model::WorldImageResource;
use crate::model::{ColorSpace, FeatureId, Model, ModelBuilder, Node, Sampler, Texture};
use std::collections::BTreeMap;
pub(super) fn read<'a, D: Dialect>(
    source: &'a Group<'_, D>,
    builder: &mut ModelBuilder<'a, Wmo>,
    region: &mut RegionControls<'a>,
) -> Result<(), Error> {
    for chunk in source.chunks() {
        if let GroupChunk::PointLights(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(error)?;
                let node = builder.new_node("Region point light");
                builder
                    .values_mut()
                    .set(node.transform.translation, row.position)?;
                let node = builder.add_node(node)?;
                region.point_lights.push(builder.add_feature(
                    node,
                    wmo::Node::PointLight(wmo::PointLight {
                        color: rgba(row.color_bgra),
                        intensity: row.intensity,
                        attenuation: [row.attenuation_start, row.attenuation_end],
                        control: row.reserved_header,
                        scalar_control: row.reserved_float,
                        trailing_controls: row.reserved_tail,
                    }),
                )?);
            }
        }
    }
    for chunk in source.chunks() {
        if let GroupChunk::PointLightAnimations(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(error)?;
                let light = *region
                    .point_lights
                    .get(row.light_index as usize)
                    .ok_or_else(|| {
                        Error::new(
                            "world.region.program.light",
                            "program references an absent point light",
                        )
                    })?;
                let cookie = if row.texture_file_id == 0 {
                    None
                } else {
                    let image_resource = builder
                        .add_image_resource(WorldImageResource::FileDataId(row.texture_file_id));
                    Some(Texture {
                        resource: image_resource,
                        controls: Default::default(),
                        color_space: ColorSpace::Unspecified,
                        sampler: Sampler::default(),
                    })
                };
                let node = builder.new_node("Region point-light program");
                builder
                    .values_mut()
                    .set(node.transform.translation, row.position)?;
                let node = builder.add_node(node)?;
                region.point_programs.push(builder.add_feature(
                    node,
                    wmo::Node::LightProgram(wmo::PointProgram {
                        light,
                        color: rgba(row.color_bgra),
                        intensity: row.intensity,
                        attenuation: [row.attenuation_start, row.attenuation_end],
                        orientation: row.rotation,
                        flicker: [row.flicker_intensity, row.flicker_speed],
                        mode: decode_flicker(row.flicker_mode),
                        cookie,
                        controls: row.reserved,
                        trailing_controls: row.reserved_tail,
                    }),
                )?);
            }
        }
    }
    for chunk in source.chunks() {
        let (rows, ids, sets) = match chunk {
            GroupChunk::PointLightSets(rows) => {
                (rows, &region.point_lights, &mut region.point_light_sets)
            }
            GroupChunk::PointAnimationSets(rows) => {
                (rows, &region.point_programs, &mut region.point_program_sets)
            }
            _ => continue,
        };
        for row in rows.iter() {
            let row = row.map_err(error)?;
            let start = row.offset as usize;
            let end = start
                .checked_add(row.count as usize)
                .ok_or_else(|| Error::new("world.region.light_set", "activation range overflow"))?;
            sets.push(
                ids.get(start..end)
                    .ok_or_else(|| {
                        Error::new(
                            "world.region.light_set",
                            "activation range exceeds light table",
                        )
                    })?
                    .to_vec(),
            );
        }
    }
    Ok(())
}
fn error(error: crate::model::formats::wmo::ReadError) -> Error {
    Error::new("world.region.light", error.to_string())
}

pub(super) fn write<'a, D: Dialect>(
    model: &Model<'_, Wmo>,
    region: &RegionControls<'_>,
    group: &mut impl super::group_output::GroupOutput<'a, D>,
) -> Result<(), Error> {
    let mut points = Vec::new();
    let mut programs = Vec::new();
    let mut indices = BTreeMap::new();
    for id in &region.point_lights {
        let feature = model.feature(*id).expect("validated region light");
        let wmo::Node::PointLight(source) = &feature.payload else {
            return Err(Error::new(
                "world.region.light",
                "region point-light membership has another feature kind",
            ));
        };
        let node = model.node(feature.node).expect("validated light anchor");
        absolute(node, model.values())?;
        indices.insert(*id, points.len() as u32);
        points.push(records::PointLight {
            reserved_header: source.control,
            color_bgra: rgba(source.color),
            position: node.transform.defaults(model.values()).translation,
            intensity: source.intensity,
            attenuation_start: source.attenuation[0],
            attenuation_end: source.attenuation[1],
            reserved_float: source.scalar_control,
            reserved_tail: source.trailing_controls,
        });
    }
    for id in &region.point_programs {
        let feature = model.feature(*id).expect("validated region program");
        let wmo::Node::LightProgram(source) = &feature.payload else {
            return Err(Error::new(
                "world.region.program",
                "program membership has another feature kind",
            ));
        };
        let node = model.node(feature.node).expect("validated program anchor");
        absolute(node, model.values())?;
        let cookie = source.cookie.map_or(Ok(0), |id| {
            match model.image_resource(id.resource).expect("validated image") {
                WorldImageResource::FileDataId(id) => Ok(*id),
                _ => Err(Error::new(
                    "world.region.program.cookie",
                    "native cookie requires a database identity",
                )),
            }
        })?;
        programs.push(records::PointLightAnimation {
            light_index: *indices.get(&source.light).ok_or_else(|| {
                Error::new(
                    "world.region.program.light",
                    "program light is not a member of this region",
                )
            })?,
            color_bgra: rgba(source.color),
            position: node.transform.defaults(model.values()).translation,
            attenuation_start: source.attenuation[0],
            attenuation_end: source.attenuation[1],
            intensity: source.intensity,
            rotation: source.orientation,
            flicker_intensity: source.flicker[0],
            flicker_speed: source.flicker[1],
            flicker_mode: flicker(source.mode),
            reserved: source.controls,
            texture_file_id: cookie,
            reserved_tail: source.trailing_controls,
        });
    }
    if !points.is_empty() {
        group.push(GroupChunk::PointLights(super::encode::array(&points)?));
    }
    if !programs.is_empty() {
        group.push(GroupChunk::PointLightAnimations(super::encode::array(
            &programs,
        )?));
    }
    let light_sets = sets(&region.point_light_sets, &region.point_lights)?;
    let program_sets = sets(&region.point_program_sets, &region.point_programs)?;
    if !light_sets.is_empty() {
        group.push(GroupChunk::PointLightSets(super::encode::array(
            &light_sets,
        )?));
    }
    if !program_sets.is_empty() {
        group.push(GroupChunk::PointAnimationSets(super::encode::array(
            &program_sets,
        )?));
    }
    Ok(())
}
fn sets(
    source: &[Vec<FeatureId>],
    ids: &[FeatureId],
) -> Result<Vec<records::LightSetRange>, Error> {
    source
        .iter()
        .map(|members| {
            let offset = if let Some(first) = members.first() {
                ids.iter().position(|id| id == first).ok_or_else(|| {
                    Error::new("world.region.light_set", "set contains an absent light")
                })?
            } else {
                0
            };
            if ids.get(offset..offset + members.len()) != Some(members.as_slice()) {
                return Err(Error::new(
                    "world.region.light_set",
                    "native activation sets require consecutive table membership",
                ));
            }
            Ok(records::LightSetRange {
                offset: offset as u32,
                count: members.len() as u32,
            })
        })
        .collect()
}
fn absolute(node: &Node<'_, Wmo>, values: &PropertyValues) -> Result<(), Error> {
    if node.parent.is_some()
        || node.transform.defaults(values).rotation != Default::default()
        || node.transform.defaults(values).scale != [1.; 3]
    {
        return Err(Error::new(
            "world.region.light.transform",
            "native region light has only absolute translation",
        ));
    }
    Ok(())
}
fn flicker(value: wmo::Flicker) -> records::FlickerMode {
    match value {
        wmo::Flicker::Off => records::FlickerMode::Off,
        wmo::Flicker::Sine => records::FlickerMode::Sine,
        wmo::Flicker::Noise => records::FlickerMode::NoiseCurve,
        wmo::Flicker::SteppedNoise => records::FlickerMode::NoiseStep,
        wmo::Flicker::Unknown(value) => records::FlickerMode::Unknown(value),
    }
}

fn decode_flicker(value: records::FlickerMode) -> wmo::Flicker {
    match value {
        records::FlickerMode::Off => wmo::Flicker::Off,
        records::FlickerMode::Sine => wmo::Flicker::Sine,
        records::FlickerMode::NoiseCurve => wmo::Flicker::Noise,
        records::FlickerMode::NoiseStep => wmo::Flicker::SteppedNoise,
        records::FlickerMode::Unknown(value) => wmo::Flicker::Unknown(value),
    }
}
