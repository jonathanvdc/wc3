//! Direct lights and camera bindings, including extended animated controls.
use super::animation::Decoder;
use super::state::ImportState;
use super::storage::Storage;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::profiles::mdlx as profile;
use crate::model::PerspectiveProjection;
use crate::model::{CameraLens, FieldOfViewAxis, PathResource};
use crate::model::{LightEmission, LightShadows, NodeId};
use glam::{Vec2, Vec3};
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

        macro_rules! a {
            ($field:ident) => {
                decoder.bind(
                    &source.$field,
                    concat!("light.", stringify!($field)),
                    |value| value.into(),
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
                decoder.bind(&range.start, "light::SHADOW_START", |value| value)?,
                decoder.bind(&range.end, "light::SHADOW_END", |value| value)?,
            ])
        } else {
            None
        };
        let falloff = if let Some(falloff) = source.authored_falloff() {
            Some([
                decoder.bind(&falloff.quadratic, "light::QUADRATIC", |value| value)?,
                decoder.bind(&falloff.linear, "light::LINEAR", |value| value)?,
                decoder.bind(&falloff.damping, "light::DAMPING", |value| value)?,
            ])
        } else {
            None
        };
        {
            let payload = profile::Node::Light(profile::Light {
                kind,
                attenuation: [a!(attenuation_start), a!(attenuation_end)],
                diffuse: LightEmission {
                    color: a!(color),
                    intensity: a!(intensity),
                },
                ambient: LightEmission {
                    color: a!(ambient_color),
                    intensity: a!(ambient_intensity),
                },
                shadows: LightShadows {
                    casting: source.try_shadow_casting().ok(),
                    intensity: source.try_shadow_intensity().ok(),
                    range: shadow_range,
                },
                falloff,
                activation: decoder.track(
                    source.visibility.as_ref(),
                    1.,
                    "light::VISIBILITY",
                    |value| value,
                )?,
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    let mut camera_nodes = Vec::new();
    for source in source.cameras() {
        let node = decoder.builder.new_node(S::text(source.name.text()));
        decoder.builder.values_mut().set(
            node.transform.translation,
            Vec3::from_array(source.position),
        )?;
        let id = decoder.builder.add_node(node)?;
        camera_nodes.push(id);
        let feature = decoder.builder.reserve_feature(id);

        let eye_offset = decoder.track(
            source.translation.as_ref(),
            Vec3::ZERO,
            "camera::EYE_OFFSET",
            |value| value.into(),
        )?;
        let target_offset = decoder.track(
            source.target_translation.as_ref(),
            Vec3::ZERO,
            "camera::TARGET_OFFSET",
            |value| value.into(),
        )?;
        macro_rules! optional {
            ($field:ident) => {
                if source.$field.is_some() {
                    Some(decoder.track(
                        source.$field.as_ref(),
                        0.,
                        concat!("camera.", stringify!($field)),
                        |value| value.into(),
                    )?)
                } else {
                    None
                }
            };
        }
        {
            let payload = profile::Node::Camera(profile::Camera {
                eye_offset,
                target: Vec3::from_array(source.target_position),
                target_offset,
                projection: PerspectiveProjection {
                    field_of_view: source.field_of_view,
                    clipping: Vec2::from_array([source.near_clip, source.far_clip]),
                    axis: FieldOfViewAxis::Vertical,
                },
                roll: decoder.track(source.rotation.as_ref(), 0., "camera::ROLL", |value| value)?,
                role: None,
                visibility: Some(decoder.track(
                    source.visibility.as_ref(),
                    1.,
                    "camera::VISIBILITY",
                    |value| value,
                )?),
                lens: CameraLens {
                    focus_distance: optional!(focus_distance),
                    focal_length: optional!(focal_length),
                    f_stop: optional!(f_stop),
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
            .chain(camera_nodes)
            .collect();
        for (index, pose) in poses.enumerate() {
            profile.bind_poses.push((
                *ids.get(index).ok_or_else(|| {
                    Error::new(
                        "bind_pose",
                        "pose has no associated hierarchy node or camera",
                    )
                })?,
                pose.0,
            ));
        }
    }
    if let Ok(faces) = source.try_face_fx() {
        for face in faces {
            let path = face
                .path
                .as_bytes()
                .split(|byte| *byte == 0)
                .next()
                .unwrap_or_default();
            let resource = decoder
                .builder
                .add_effect_resource(PathResource::Path(S::bytes(path)));
            profile.metadata.face_effects.push(profile::FaceEffect {
                name: S::text(face.name.text()),
                resource,
            });
        }
    }
    Ok(())
}
