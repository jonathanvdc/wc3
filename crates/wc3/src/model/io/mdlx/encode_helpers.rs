//! Visibility, event occurrences, and model-wide native authoring controls.
use super::encode_animation::{frame_at, Encoder};
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::profiles::mdlx as profile;
use crate::model::{Animated, Clock, FeatureId, MeshId, Quaternion};
use native::animation::Track;
use native::scene::Node;
use std::collections::BTreeMap;

pub(super) fn visibility(
    source: &Animated<f32>,
    encoder: &mut Encoder<'_, '_, '_>,
    path: &str,
) -> Result<Option<Track<'static, f32>>, Error> {
    let track = source
        .binding
        .as_ref()
        .map(|property| encoder.track(property, Ok, path))
        .transpose()?
        .flatten();
    Ok(track.or_else(|| (source.default != 1.).then(|| Track::constant(source.default))))
}
pub(super) fn events(
    source: &profile::Events<'_>,
    feature: FeatureId,
    node: Node<'static>,
    encoder: &mut Encoder<'_, '_, '_>,
) -> Result<Vec<native::EventObject<'static>>, Error> {
    let model = encoder.model;
    let mut groups = BTreeMap::<(Option<u32>, String), BTreeMap<i32, usize>>::new();
    let clips = model
        .clips()
        .map(|(id, _)| {
            (
                encoder.starts[&id],
                encoder
                    .model
                    .clip_program(id)
                    .expect("validated clip alias")
                    .events(),
            )
        })
        .chain(std::iter::once((0., model.shared_events())))
        .chain(std::iter::once((0., model.timeline_events())));
    for (start, events) in clips {
        let mut occurrences = BTreeMap::<(Option<u32>, String), BTreeMap<i32, usize>>::new();
        for event in events {
            if event.feature != Some(feature) {
                continue;
            }
            let global = match event.clock {
                Clock::Clip | Clock::Timeline => None,
                Clock::Shared(id) => Some(encoder.globals[&id]),
            };
            let frame = frame_at(
                event.time.as_seconds(),
                if global.is_some() { 0. } else { start },
                encoder.context,
                "event.time",
            )?;
            *occurrences
                .entry((global, event.name.to_string()))
                .or_default()
                .entry(frame)
                .or_default() += 1;
            if !event.payload.is_empty() {
                encoder.context.unsupported("event.payload", "Warcraft III event occurrences carry names and timestamps but no binary messages")?;
            }
        }
        for (group, counts) in occurrences {
            let old = groups.entry(group).or_default();
            for (frame, count) in counts {
                let old = old.entry(frame).or_default();
                *old = (*old).max(count);
            }
        }
    }
    if groups.is_empty() {
        groups.insert((None, source.name.to_string()), BTreeMap::new());
    }
    if groups.len() > 1 {
        encoder.context.unsupported(
            "event.source",
            "one Warcraft III event source has one name and evaluation clock",
        )?;
    }
    let mut output = Vec::new();
    for ((global, name), frames) in groups.into_iter().take(1) {
        let mut node = node.clone();
        node.name
            .set_text(&name)
            .map_err(|error| Error::new("event.name", error.to_string()))?;
        let times = frames
            .into_iter()
            .flat_map(|(frame, count)| std::iter::repeat_n(frame, count))
            .collect::<Vec<_>>();
        output.push(native::EventObject::new(
            node,
            global.unwrap_or(u32::MAX),
            &times,
        ));
    }
    Ok(output)
}
pub(super) fn controls<T: Dialect>(
    target: &mut native::Document<'_, T>,
    encoder: &mut Encoder<'_, '_, '_>,
    meshes: &BTreeMap<MeshId, Vec<u32>>,
) -> Result<(), Error> {
    let model = encoder.model;
    {
        let mut transforms = Vec::new();
        for (_, source) in model.texture_transforms() {
            transforms.push(native::TextureAnimation {
                translation: source
                    .translation
                    .binding
                    .as_ref()
                    .map(|property| encoder.track(property, Ok, "texture_transform.translation"))
                    .transpose()?
                    .flatten(),
                rotation: source
                    .rotation
                    .binding
                    .as_ref()
                    .map(|property| {
                        encoder.track(
                            property,
                            |value: Quaternion| Ok(value.components()),
                            "texture_transform.rotation",
                        )
                    })
                    .transpose()?
                    .flatten(),
                scaling: source
                    .scale
                    .binding
                    .as_ref()
                    .map(|property| encoder.track(property, Ok, "texture_transform.scale"))
                    .transpose()?
                    .flatten(),
            });
            let transform = transforms.last_mut().expect("just added transform");
            if transform.translation.is_none() && source.translation.default != [0.; 3] {
                transform.translation = Some(Track::constant(source.translation.default));
            }
            if transform.rotation.is_none() && source.rotation.default != Quaternion::default() {
                transform.rotation = Some(Track::constant(source.rotation.default.components()));
            }
            if transform.scaling.is_none() && source.scale.default != [1.; 3] {
                transform.scaling = Some(Track::constant(source.scale.default));
            }
        }
        target.set_texture_animations(transforms);
        target.set_gliders(
            model
                .meshes()
                .filter_map(|(id, mesh)| mesh.controls.glider.then_some(id))
                .flat_map(|mesh| meshes.get(&mesh).into_iter().flatten())
                .map(|id| native::Glider { geoset_id: *id }),
        );
        {
            let profile = model.metadata();
            if !profile.face_effects.is_empty() {
                let faces = profile
                    .face_effects
                    .iter()
                    .map(|(name, resource)| {
                        native::FaceFx::new(name, resource)
                            .map_err(|error| Error::new("face_effect", error.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if let Err(error) = target.try_set_face_fx(faces) {
                    encoder
                        .context
                        .unsupported("face_effect", error.to_string())?;
                }
            }
        }
        if model
            .nodes()
            .any(|(_, node)| node.controls.bind_pose.is_some())
        {
            let node_ids: BTreeMap<_, _> = encoder
                .model
                .nodes()
                .enumerate()
                .map(|(index, (id, _))| (index as u32, id))
                .collect();
            let identity = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0.];
            let poses = target
                .nodes()
                .iter()
                .map(|node| {
                    native::BindPoseMatrix(
                        node_ids
                            .get(&node.object_id)
                            .and_then(|id| model.node(*id))
                            .and_then(|node| node.controls.bind_pose)
                            .unwrap_or(identity),
                    )
                })
                .collect::<Vec<_>>();
            if let Err(error) = target.try_set_bind_poses(poses) {
                encoder
                    .context
                    .unsupported("bind_poses", error.to_string())?;
            }
        }
    }
    let max = target
        .nodes()
        .iter()
        .map(|node| node.object_id as usize + 1)
        .max()
        .unwrap_or_default();
    let mut pivots = target.pivot_points();
    pivots.resize(max.max(pivots.len()), [0.; 3]);
    target.set_pivot_points(&pivots);
    Ok(())
}
