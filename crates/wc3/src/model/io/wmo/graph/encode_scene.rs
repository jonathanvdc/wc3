//! Static world actors and named placement membership compiled to native root tables.
use super::encode::{append, array};
use crate::model::conversion::{Context, Error};
use crate::model::formats::wmo::records;
use crate::model::formats::wmo::root::RootChunk;
use crate::model::formats::wmo::Dialect;
use crate::model::profiles::wmo as profile;
use crate::model::profiles::Wmo;
use crate::model::WorldImageResource;
use crate::model::WorldModelResource;
use crate::model::{FeatureId, Model};
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct Indices {
    pub lights: BTreeMap<FeatureId, u16>,
    pub modern: BTreeMap<FeatureId, u16>,
    pub doodads: BTreeMap<FeatureId, u16>,
}
pub(super) fn write<'a, D: Dialect>(
    model: &Model<'_, Wmo>,
    world: &profile::WorldSystem<'_>,
    root: &mut impl super::root_output::RootOutput<'a, D>,
    context: &mut Context,
) -> Result<Indices, Error> {
    let mut indices = Indices::default();
    let mut lights = Vec::new();
    let mut modern = Vec::new();
    let mut doodads = Vec::new();
    let mut multipliers = Vec::new();
    let mut extensions = Vec::new();
    let mut names = vec![0u8];
    let mut files = Vec::new();
    let mut path_mode = false;
    let mut file_mode = false;
    for (feature_id, feature) in model
        .features()
        .iter()
        .filter(|(_, feature)| matches!(feature.payload, profile::Node::Light(_)))
        .chain(
            model
                .features()
                .iter()
                .filter(|(_, feature)| matches!(feature.payload, profile::Node::Attachment(_))),
        )
    {
        let node = model.node(feature.node).expect("validated feature anchor");
        if node.parent.is_some() {
            context.unsupported(
                "world.actor.parent",
                "world root actor tables contain absolute transforms",
            )?;
        }
        match &feature.payload {
            profile::Node::Light(source) => {
                if source.color.binding.is_some() || source.intensity.binding.is_some() {
                    context.unsupported(
                        "world.light.animation",
                        "root light tables encode constant color and intensity",
                    )?;
                }
                let color = color(source.color.default, context)?;
                if let Some(extra) = &source.modern {
                    if node.transform.rotation != crate::model::Quaternion::default() {
                        context.unsupported("world.light.rotation","native modern light Euler axis order is not established for quaternion translation")?;
                    }
                    let kind = match source.kind {
                        profile::LightKind::Point => records::NewLightKind::Point,
                        profile::LightKind::Spot => records::NewLightKind::Spot,
                        profile::LightKind::Unknown(value) => records::NewLightKind::Unknown(value),
                        _ => {
                            context.unsupported(
                                "world.light.kind",
                                "modern light table supports point and spot illumination",
                            )?;
                            records::NewLightKind::Point
                        }
                    };
                    let mut flags = records::LightFlags(source.controls.bits() as i32);
                    flags.set_blend_colors(extra.blend_colors);
                    flags.set_cast_shadows(extra.shadows);
                    let cookie = extra
                        .cookie
                        .map(|id| {
                            match model
                                .image_resource(
                                    model.texture(id).expect("validated cookie").resource,
                                )
                                .expect("validated image")
                            {
                                WorldImageResource::FileDataId(value) => Ok(*value),
                                _ => Err(Error::new(
                                    "world.light.cookie",
                                    "light cookies require a database file identifier",
                                )),
                            }
                        })
                        .transpose()?
                        .unwrap_or(0);
                    let row = records::NewLight {
                        kind,
                        light_index: extra.id,
                        flags,
                        doodad_set: extra.doodad_set.map_or(Ok(0), |id| {
                            i32::try_from(id.index).map_err(|_| {
                                Error::new("world.light.set", "set index exceeds native capacity")
                            })
                        })?,
                        inner_color: color,
                        position: node.transform.translation,
                        rotation: extra.orientation,
                        attenuation_start: source.attenuation[0],
                        attenuation_end: source.attenuation[1],
                        intensity: source.intensity.default,
                        outer_color: super::rgba(extra.outer_color),
                        blend_start: extra.color_blend[0],
                        blend_end: extra.color_blend[1],
                        gap0: extra.controls[0],
                        flicker_intensity: extra.flicker_parameters[0],
                        flicker_speed: extra.flicker_parameters[1],
                        flicker_mode: match extra.flicker {
                            profile::Flicker::Off => records::FlickerMode::Off,
                            profile::Flicker::Sine => records::FlickerMode::Sine,
                            profile::Flicker::Noise => records::FlickerMode::NoiseCurve,
                            profile::Flicker::SteppedNoise => records::FlickerMode::NoiseStep,
                            profile::Flicker::Unknown(value) => {
                                records::FlickerMode::Unknown(value)
                            }
                        },
                        unknown_vector: extra.vector_control,
                        gap1: extra.controls[1],
                        cookie_file_id: cookie,
                        gap2: extra.controls[2..].try_into().expect("five controls"),
                        falloff: extra.falloff,
                        inner_angle: source.cone[0],
                        outer_angle: source.cone[1],
                        scale_half: half(extra.scale, context)?,
                        intensity_multiplier_half: half(extra.intensity_multiplier, context)?,
                        unused: extra.trailing_controls,
                    };
                    indices.modern.insert(
                        feature_id,
                        u16::try_from(modern.len()).map_err(|_| {
                            Error::new("world.lights", "light count exceeds capacity")
                        })?,
                    );
                    modern.push(row);
                } else {
                    let kind = match source.kind {
                        profile::LightKind::Point => records::LightKind::Omni,
                        profile::LightKind::Spot => records::LightKind::Spot,
                        profile::LightKind::Directional => records::LightKind::Directional,
                        profile::LightKind::Ambient => records::LightKind::Ambient,
                        profile::LightKind::Unknown(value) => {
                            records::LightKind::Unknown(u8::try_from(value).map_err(|_| {
                                Error::new(
                                    "world.light.kind",
                                    "legacy kind exceeds eight-bit selector",
                                )
                            })?)
                        }
                    };
                    let padding = source.controls.bits().to_le_bytes();
                    indices.lights.insert(
                        feature_id,
                        u16::try_from(lights.len()).map_err(|_| {
                            Error::new("world.lights", "light count exceeds capacity")
                        })?,
                    );
                    if let Some(terms) = source.directional_terms {
                        extensions.push(records::LightExtension {
                            terms: terms
                                .map(|(direction, value)| records::LightTerm { direction, value }),
                            light_index: u8::try_from(lights.len()).map_err(|_| {
                                Error::new(
                                    "world.light.extension",
                                    "extension light identity exceeds eight-bit capacity",
                                )
                            })?,
                            unknown: [0; 3],
                        });
                    }
                    lights.push(records::Light {
                        kind,
                        use_attenuation: u8::from(source.use_attenuation),
                        padding: [padding[0], padding[1]],
                        color,
                        position: node.transform.translation,
                        intensity: source.intensity.default,
                        rotation: node.transform.rotation.components(),
                        attenuation_start: source.attenuation[0],
                        attenuation_end: source.attenuation[1],
                    });
                }
            }
            profile::Node::PointLight(_) | profile::Node::LightProgram(_) => {}
            profile::Node::Attachment(source) => {
                if node.transform.scale[1] != node.transform.scale[0]
                    || node.transform.scale[2] != node.transform.scale[0]
                {
                    context.unsupported(
                        "world.doodad.scale",
                        "native doodads store one uniform scale",
                    )?;
                }
                let selector = match model
                    .model_resource(source.resource)
                    .expect("validated model")
                {
                    WorldModelResource::FileDataId(file) => match root.addressing() {
                        super::root_output::Addressing::PathsOrFileIds => {
                            file_mode = true;
                            let selector = u32::try_from(files.len())
                                .map_err(|error| Error::new("world.doodad", error.to_string()))?;
                            files.push(*file);
                            selector
                        }
                        super::root_output::Addressing::Paths => {
                            context.unsupported(
                                "world.doodad.resource",
                                "historical doodad placements require model file paths",
                            )?;
                            path_mode = true;
                            0
                        }
                    },
                    WorldModelResource::Path(path) => {
                        path_mode = true;
                        let selector = names.len() as u32;
                        if path.contains(&0) {
                            return Err(Error::new(
                                "world.doodad.path",
                                "resource path contains NUL",
                            ));
                        }
                        names.extend_from_slice(path);
                        names.push(0);
                        selector
                    }
                };
                if selector > 0xffffff {
                    return Err(Error::new(
                        "world.doodad.reference",
                        "doodad resource selector exceeds native capacity",
                    ));
                }
                let mut reference = records::DoodadReference::default();
                reference.set_name_index(selector);
                let mut flags = records::DoodadFlags(source.appearance.controls.bits());
                flags.set_has_light_reference(matches!(
                    source.appearance.tint,
                    profile::DoodadTint::Lit { .. }
                ));
                reference.set_flags(flags);
                let color = match source.appearance.tint {
                    profile::DoodadTint::Color(color) => super::rgba(color),
                    profile::DoodadTint::Lit {
                        color: [r, g, b],
                        light,
                    } => [
                        b,
                        g,
                        r,
                        u8::try_from(*indices.lights.get(&light).ok_or_else(|| {
                            Error::new("world.doodad.light", "tint references a nonlegacy light")
                        })?)
                        .map_err(|_| {
                            Error::new(
                                "world.doodad.light",
                                "light selector exceeds native capacity",
                            )
                        })?,
                    ],
                };
                indices.doodads.insert(
                    feature_id,
                    u16::try_from(doodads.len()).map_err(|_| {
                        Error::new("world.doodads", "doodad count exceeds reference capacity")
                    })?,
                );
                multipliers.push(source.appearance.color_multiplier);
                doodads.push(records::DoodadDef {
                    name_and_flags: reference,
                    position: node.transform.translation,
                    rotation: node.transform.rotation.components(),
                    scale: node.transform.scale[0],
                    color,
                });
            }

            profile::Node::Camera(unavailable)
            | profile::Node::Particle(unavailable)
            | profile::Node::Ribbon(unavailable)
            | profile::Node::Event(unavailable)
            | profile::Node::GeometryAnimation(unavailable)
            | profile::Node::CollisionShape(unavailable)
            | profile::Node::ParticleProgram(unavailable)
            | profile::Node::FaceActor(unavailable)
            | profile::Node::Extent(unavailable)
            | profile::Node::ParticleCopy(unavailable)
            | profile::Node::Projector(unavailable)
            | profile::Node::Force(unavailable)
            | profile::Node::Warp(unavailable)
            | profile::Node::HitTest(unavailable)
            | profile::Node::AttachmentVolume(unavailable)
            | profile::Node::Billboard(unavailable)
            | profile::Node::Turret(unavailable)
            | profile::Node::IkJoint(unavailable)
            | profile::Node::IkTwoJoint(unavailable)
            | profile::Node::IkCcd(unavailable)
            | profile::Node::OneBoneSolver(unavailable)
            | profile::Node::ShadowBox(unavailable)
            | profile::Node::ViewVolume(unavailable)
            | profile::Node::AtlasParticles(unavailable) => match *unavailable {},
        }
    }
    if path_mode && file_mode {
        return Err(Error::new(
            "world.doodads",
            "one root cannot mix path and database doodad reference modes",
        ));
    }
    let mut sets = Vec::new();
    for source in &world.doodad_sets {
        let members = source
            .members
            .iter()
            .map(|node| {
                indices
                    .doodads
                    .get(node)
                    .copied()
                    .ok_or_else(|| Error::new("world.set", "set member has no doodad feature"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let first = members.first().copied().unwrap_or(0);
        if members
            .iter()
            .enumerate()
            .any(|(offset, index)| usize::from(*index) != usize::from(first) + offset)
        {
            context.unsupported(
                "world.set.membership",
                "native doodad sets require a consecutive placement interval",
            )?;
        }
        let mut name = [0u8; 20];
        if source.name.len() > 20 || source.name.as_bytes().contains(&0) {
            return Err(Error::new(
                "world.set.name",
                "name exceeds native field or contains NUL",
            ));
        }
        name[..source.name.len()].copy_from_slice(source.name.as_bytes());
        sets.push(records::DoodadSet {
            name,
            start_index: u32::from(first),
            count: members.len() as u32,
            padding: source.control,
        });
    }
    root.actor_counts([
        lights.len() as u32,
        doodads.len() as u32,
        doodads.len() as u32,
        sets.len() as u32,
    ]);
    if !lights.is_empty() {
        append(root, RootChunk::Lights(array(&lights)?))?;
    }
    if !modern.is_empty() {
        append(root, RootChunk::NewLights(array(&modern)?))?;
    }
    if !extensions.is_empty() {
        append(root, RootChunk::LightExtensions(array(&extensions)?))?;
    }
    if multipliers.iter().any(Option::is_some) {
        if multipliers.iter().any(Option::is_none) {
            return Err(Error::new(
                "world.doodad.color_multiplier",
                "native multiplier table must cover every placement",
            ));
        }
        append(
            root,
            RootChunk::DoodadColorMultipliers(array(
                &multipliers
                    .into_iter()
                    .map(|value| value.expect("checked multiplier presence"))
                    .collect::<Vec<_>>(),
            )?),
        )?;
    }
    if !doodads.is_empty() {
        append(root, RootChunk::Doodads(array(&doodads)?))?;
    }
    if !sets.is_empty() {
        append(root, RootChunk::DoodadSets(array(&sets)?))?;
    }
    if path_mode {
        append(root, RootChunk::DoodadNames(array(&names)?))?;
    }
    if file_mode {
        append(root, RootChunk::DoodadFileIds(array(&files)?))?;
    }
    Ok(indices)
}
fn color(source: [f32; 4], context: &mut Context) -> Result<[u8; 4], Error> {
    let mut result = [0u8; 4];
    for (target, value) in result.iter_mut().zip(source) {
        *target = (value.clamp(0., 1.) * 255.).round() as u8;
        let error = (value - f32::from(*target) / 255.).abs();
        if error != 0. {
            context.approximate(
                "world.light.color",
                error,
                "native light colors use eight-bit normalized components",
            )?;
        }
    }
    Ok([result[2], result[1], result[0], result[3]])
}
fn half(value: f32, context: &mut Context) -> Result<u16, Error> {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 255) as i32 - 127 + 15;
    let fraction = bits & 0x7fffff;
    let encoded = if exponent <= 0 {
        if exponent < -10 {
            sign
        } else {
            let mantissa = fraction | 0x800000;
            let shift = (14 - exponent) as u32;
            let rounded = (mantissa + (1 << (shift - 1)) - 1 + ((mantissa >> shift) & 1)) >> shift;
            sign | rounded as u16
        }
    } else if exponent >= 31 {
        sign | 0x7c00
    } else {
        let rounded = fraction + 0xfff + ((fraction >> 13) & 1);
        sign | (((exponent as u16) << 10) + ((rounded >> 13) as u16))
    };
    let restored = super::scene::half(encoded);
    if restored.to_bits() != value.to_bits() {
        context.approximate(
            "world.light.half",
            (restored - value).abs(),
            "native light control uses binary16 precision",
        )?;
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::conversion::ExportOptions;
    #[test]
    fn strict_color_conversion_does_not_hide_small_authored_changes() {
        let mut context = Context::export(&ExportOptions::default());
        assert!(color([1e-8, 0., 0., 1.], &mut context).is_err());
        let mut context = Context::export(&ExportOptions::default());
        assert_eq!(
            color(
                [17., 91., 201., 255.].map(|component| component / 255.),
                &mut context
            )
            .unwrap(),
            [201, 91, 17, 255]
        );
    }
}
