//! Visibility, event occurrences, and model-wide native authoring controls.
use super::encode_animation::{frame_at, Encoder};
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::profiles::mdlx as profile;
use crate::model::{Animated, Clock, FeatureId, MeshId, Quaternion, TextureTransform};
use native::animation::Track;
use native::scene::Node;
use std::collections::BTreeMap;

pub(super) fn visibility(
    source: &Animated<f32>,
    encoder: &mut Encoder<'_, '_, '_>,
    path: &str,
) -> Result<Option<Track<'static, f32>>, Error> {
    encoder.optional(*source, Ok, 1., path)
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
        .iter()
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
        target.set_gliders(
            model
                .meshes()
                .iter()
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
            .iter()
            .any(|(_, node)| node.controls.bind_pose.is_some())
        {
            let node_ids: BTreeMap<_, _> = encoder
                .model
                .nodes()
                .iter()
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

pub(super) fn texture_transform<'a>(
    source: &TextureTransform,
    encoder: &mut Encoder<'_, '_, '_>,
) -> Result<native::TextureAnimation<'a>, Error> {
    Ok(native::TextureAnimation {
        translation: encoder.optional(
            source.translation,
            Ok,
            [0.; 3],
            "texture_transform.translation",
        )?,
        rotation: encoder.optional(
            source.rotation,
            |value: Quaternion| Ok(value.components()),
            Quaternion::default().components(),
            "texture_transform.rotation",
        )?,
        scaling: encoder.optional(source.scale, Ok, [1.; 3], "texture_transform.scale")?,
    })
}
