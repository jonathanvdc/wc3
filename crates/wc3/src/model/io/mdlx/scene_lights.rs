//! Direct lights and camera bindings, including extended animated controls.
use super::animation::Decoder;
use super::state::ImportState;
use super::storage::Storage;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::profiles::mdlx as profile;
use crate::model::{LightEmission, NodeId, PhotographicControls, VisibleLight};
use std::collections::BTreeMap;

pub(super) fn read<'source, 'a, D: Dialect, S: Storage<'source, 'a>>(
    source: &'source native::Document<'_, D>,
    decoder: &mut Decoder<'a, '_>,
    ids: &BTreeMap<u32, NodeId>,
    profile: &mut ImportState<'a>,
) -> Result<(), Error> {
    for source in source.lights() {
        let id = *ids
            .get(&source.node.object_id)
            .ok_or_else(|| Error::new("light.node", "node identity does not exist"))?;
        let feature = decoder.builder.reserve_feature(id);
        let owner = feature;
        macro_rules! a {
            ($field:ident, $key:ident) => {
                decoder.bind(
                    &source.$field,
                    owner,
                    profile::properties::light::$key,
                    |value| value,
                )?
            };
        }
        let kind = match source.light_type {
            native::LightType::Omnidirectional => profile::LightKind::Omni,
            native::LightType::Directional => profile::LightKind::Directional,
            native::LightType::Ambient => profile::LightKind::Ambient,
            native::LightType::Unknown(value) => profile::LightKind::Unknown(value),
        };
        let shadow_range = if let Ok(range) = source.try_shadow_casting_range_ref() {
            Some([
                decoder.bind(
                    &range.start,
                    owner,
                    profile::properties::light::SHADOW_START,
                    |value| value,
                )?,
                decoder.bind(
                    &range.end,
                    owner,
                    profile::properties::light::SHADOW_END,
                    |value| value,
                )?,
            ])
        } else {
            None
        };
        let falloff = if let Some(falloff) = source.authored_falloff() {
            Some([
                decoder.bind(
                    &falloff.quadratic,
                    owner,
                    profile::properties::light::QUADRATIC,
                    |value| value,
                )?,
                decoder.bind(
                    &falloff.linear,
                    owner,
                    profile::properties::light::LINEAR,
                    |value| value,
                )?,
                decoder.bind(
                    &falloff.damping,
                    owner,
                    profile::properties::light::DAMPING,
                    |value| value,
                )?,
            ])
        } else {
            None
        };
        {
            let payload = profile::Node::Light(profile::Light {
                kind,
                attenuation: [
                    a!(attenuation_start, ATTENUATION_START),
                    a!(attenuation_end, ATTENUATION_END),
                ],
                diffuse: LightEmission {
                    color: a!(color, COLOR),
                    intensity: a!(intensity, INTENSITY),
                },
                ambient: LightEmission {
                    color: a!(ambient_color, AMBIENT_COLOR),
                    intensity: a!(ambient_intensity, AMBIENT_INTENSITY),
                },
                activation: VisibleLight {
                    shadow_casting: source.try_shadow_casting().ok(),
                    shadow_intensity: source.try_shadow_intensity().ok(),
                    shadow_range,
                    falloff,
                    visibility: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        owner,
                        profile::properties::light::VISIBILITY,
                        |value| value,
                    )?,
                },
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    for source in source.cameras() {
        let node = decoder.builder.new_node(S::text(source.name.text()));
        decoder
            .builder
            .values_mut()
            .set(node.transform.translation, source.position)?;
        let id = decoder.builder.add_node(node)?;
        let feature = decoder.builder.reserve_feature(id);
        let owner = feature;
        let eye_offset = decoder.track(
            source.translation.as_ref(),
            [0.; 3],
            owner,
            profile::properties::camera::EYE_OFFSET,
            |value| value,
        )?;
        let target_offset = decoder.track(
            source.target_translation.as_ref(),
            [0.; 3],
            owner,
            profile::properties::camera::TARGET_OFFSET,
            |value| value,
        )?;
        macro_rules! optional {
            ($field:ident, $key:ident) => {
                if source.$field.is_some() {
                    Some(decoder.track(
                        source.$field.as_ref(),
                        0.,
                        owner,
                        profile::properties::camera::$key,
                        |value| value,
                    )?)
                } else {
                    None
                }
            };
        }
        {
            let payload = profile::Node::Camera(profile::Camera {
                eye_offset,
                target: source.target_position,
                target_offset,
                field_of_view: source.field_of_view,
                clipping: [source.near_clip, source.far_clip],
                roll: decoder.track(
                    source.rotation.as_ref(),
                    0.,
                    owner,
                    profile::properties::camera::ROLL,
                    |value| value,
                )?,
                convention: PhotographicControls {
                    visibility: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        owner,
                        profile::properties::camera::VISIBILITY,
                        |value| value,
                    )?,
                    focus_distance: optional!(focus_distance, FOCUS_DISTANCE),
                    focal_length: optional!(focal_length, FOCAL_LENGTH),
                    f_stop: optional!(f_stop, F_STOP),
                },
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    if let Ok(poses) = source.try_bind_poses() {
        let ids: Vec<_> = source
            .nodes()
            .iter()
            .map(|node| ids[&node.object_id])
            .collect();
        for (index, pose) in poses.enumerate() {
            profile.bind_poses.push((
                *ids.get(index).ok_or_else(|| {
                    Error::new("bind_pose", "pose has no associated hierarchy node")
                })?,
                pose.0,
            ));
        }
    }
    if let Ok(faces) = source.try_face_fx() {
        profile
            .metadata
            .face_effects
            .extend(faces.map(|face| (S::text(face.name.text()), S::text(face.path.text()))));
    }
    Ok(())
}
