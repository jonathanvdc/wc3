//! Visibility, event occurrences, and model-wide native authoring controls.
use super::encode_animation::{frame_at, Encoder};
use crate::model::conversion::Error;
use crate::model::error::Location;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::profiles::mdlx as profile;
use crate::model::{
    EventSourceId, MeshId, PathResource, Property, Quat, TextureTransform, TimeSource,
};
use native::animation::Track;
use native::scene::Node;
use std::collections::{BTreeMap, HashSet};

pub(super) fn visibility(
    source: &Property<f32>,
    encoder: &mut Encoder<'_, '_, '_>,
    path: &Location,
) -> Result<Option<Track<'static, f32>>, Error> {
    encoder.optional(*source, Ok, 1., path)
}
pub(super) fn events(
    source: &profile::EventSource<'_>,
    source_id: EventSourceId,
    node: Node<'static>,
    encoder: &mut Encoder<'_, '_, '_>,
) -> Result<native::EventObject<'static>, Error> {
    let model = encoder.model;
    let mut selections = model.global_animations().to_vec();
    if let Some((_, clip)) = model.clips().iter().next() {
        selections.extend_from_slice(&clip.animations);
    } else {
        selections.extend(model.animations().iter().map(|(id, _)| id));
    }
    let mut seen = HashSet::new();
    let mut frames = Vec::new();
    for animation in selections.into_iter().filter(|id| seen.insert(*id)) {
        for event in model
            .animation(animation)
            .expect("validated animation")
            .events
            .values()
        {
            if event.source == source_id {
                frames.push(frame_at(
                    event.time.as_seconds(),
                    0.,
                    encoder.context,
                    &Location::new().field("event").field("time"),
                )?);
            }
        }
    }
    frames.sort_unstable();
    let mut node = node;
    node.name
        .set_text(&source.name)
        .map_err(|error| Error::from_source(Location::new().field("event").field("name"), error))?;
    let global = match source.clock {
        TimeSource::Animation => u32::MAX,
        TimeSource::Shared(id) => encoder.globals[&id],
    };
    Ok(native::EventObject::new(node, global, &frames))
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
                .filter_map(|(id, mesh)| mesh.properties.glider.then_some(id))
                .flat_map(|mesh| meshes.get(&mesh).into_iter().flatten())
                .map(|id| native::Glider { geoset_id: *id }),
        );
        {
            let profile = model.metadata();
            if !profile.face_effects.is_empty() {
                let faces = profile
                    .face_effects
                    .iter()
                    .map(|effect| {
                        let mut face = native::FaceFx::new(&effect.name, "").map_err(|error| {
                            Error::from_source(
                                Location::new().field("face_effect").field("name"),
                                error,
                            )
                        })?;
                        let PathResource::Path(path) = model
                            .effect_resource(effect.resource)
                            .expect("validated facial-animation resource");
                        face.path.set_bytes(path).map_err(|error| {
                            Error::from_source(
                                Location::new().field("face_effect").field("resource"),
                                error,
                            )
                        })?;
                        Ok(face)
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                if let Err(error) = target.try_set_face_fx(faces) {
                    encoder
                        .context
                        .unsupported_source(Location::new().field("face_effect"), error)?;
                }
            }
        }
        if model
            .nodes()
            .iter()
            .any(|(_, node)| node.properties.bind_pose.is_some())
        {
            let node_ids: BTreeMap<_, _> = encoder
                .model
                .nodes()
                .iter()
                .enumerate()
                .map(|(index, (id, _))| (index as u32, id))
                .collect();
            let identity = [1., 0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0.];
            let poses = target
                .nodes()
                .iter()
                .map(|node| node_ids.get(&node.object_id).copied())
                .chain(model.features().iter().filter_map(|(_, feature)| {
                    matches!(feature.payload, profile::Node::Camera(_))
                        .then_some(Some(feature.node))
                }))
                .map(|id| {
                    native::BindPoseMatrix(
                        id.and_then(|id| model.node(id))
                            .and_then(|node| node.properties.bind_pose)
                            .unwrap_or(identity),
                    )
                })
                .collect::<Vec<_>>();
            if let Err(error) = target.try_set_bind_poses(poses) {
                encoder
                    .context
                    .unsupported_source(Location::new().field("bind_poses"), error)?;
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
            |v| Ok(v.to_array()),
            [0.; 3],
            &Location::new()
                .field("texture_transform")
                .field("translation"),
        )?,
        rotation: encoder.optional(
            source.rotation,
            |value: Quat| Ok(value.to_array()),
            Quat::default().to_array(),
            &Location::new().field("texture_transform").field("rotation"),
        )?,
        scaling: encoder.optional(
            source.scale,
            |v| Ok(v.to_array()),
            [1.; 3],
            &Location::new().field("texture_transform").field("scale"),
        )?,
    })
}
