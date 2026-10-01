use super::material::Wc3LayerMaterial;
use crate::node_pose::{camera_anchor, resolve_pose, world_input, NodeSample, PoseInput};
use crate::texture_bindings::Wc3TextureBindings;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use wc3::model::animation::{Animatable, Interpolate, Sequence, Track, TrackValue};
use wc3::model::materials::LayerFilterMode;
use wc3::model::scene::NodeFlags;

#[derive(Component, Clone)]
pub struct Wc3Animation {
    pub sequence: usize,
    pub elapsed_ms: f64,
    pub speed: f64,
    pub playing: bool,
    pub(crate) sequences: Vec<Sequence>,
    pub(crate) global_sequences: Vec<u32>,
}

impl Wc3Animation {
    pub fn sequences(&self) -> &[Sequence] {
        &self.sequences
    }

    pub fn play(&mut self, sequence: usize) {
        if sequence < self.sequences.len() {
            self.sequence = sequence;
            self.elapsed_ms = 0.0;
            self.playing = true;
        }
    }
}

#[derive(Component)]
pub(crate) struct AnimatedNode {
    pub(crate) root: Entity,
    pub(crate) flags: NodeFlags,
    pub(crate) camera: Option<Transform>,
    pub(crate) pivot: Vec3,
    pub(crate) parent_pivot: Vec3,
    pub(crate) translation: Option<Track<[f32; 3]>>,
    pub(crate) rotation: Option<Track<[f32; 4]>>,
    pub(crate) scaling: Option<Track<[f32; 3]>>,
}

#[derive(Component)]
pub(crate) struct AnimatedLayer {
    pub(crate) root: Entity,
    pub(crate) alpha: Animatable<f32>,
    pub(crate) geoset_alpha: Option<Animatable<f32>>,
    pub(crate) geoset_color: Option<Animatable<[f32; 3]>>,
    pub(crate) texture_id: Animatable<u32>,
}

pub(crate) fn animate_layers(
    instances: Query<(&Wc3Animation, Option<&Wc3TextureBindings>)>,
    mut layers: Query<(
        &AnimatedLayer,
        &MeshMaterial3d<Wc3LayerMaterial>,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
) {
    for (layer, material_handle, mut visibility) in &mut layers {
        let Ok((animation, bindings)) = instances.get(layer.root) else {
            continue;
        };
        let Some(mut material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        let alpha = layer
            .alpha
            .track()
            .and_then(|track| sample(track, animation))
            .or_else(|| layer.alpha.value().copied())
            .unwrap_or(1.0);
        let geoset_alpha = layer
            .geoset_alpha
            .as_ref()
            .map(|alpha| {
                alpha
                    .track()
                    .and_then(|track| sample(track, animation))
                    .or_else(|| alpha.value().copied())
                    .unwrap_or(1.0)
            })
            .unwrap_or(1.0);
        *visibility = if alpha * geoset_alpha <= 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if layer.geoset_alpha.is_some() {
            let partial = (0.0..1.0).contains(&geoset_alpha);
            match material.extension.filter {
                LayerFilterMode::None => {
                    material.base.alpha_mode = if partial {
                        AlphaMode::AlphaToCoverage
                    } else {
                        AlphaMode::Opaque
                    };
                }
                LayerFilterMode::Transparent => {
                    material.base.alpha_mode = if partial {
                        AlphaMode::AlphaToCoverage
                    } else {
                        AlphaMode::Mask(if material.extension.hd.maps.x != 0 {
                            0.75
                        } else {
                            0.5
                        })
                    };
                }
                _ => {}
            }
        }
        let texture_id = layer
            .texture_id
            .track()
            .and_then(|track| sample(track, animation))
            .or_else(|| layer.texture_id.value().copied())
            .unwrap_or(0);
        let [red, green, blue] = layer
            .geoset_color
            .as_ref()
            .map(|color| {
                color
                    .track()
                    .and_then(|track| sample(track, animation))
                    .or_else(|| color.value().copied())
                    .unwrap_or([1.0; 3])
            })
            .unwrap_or([1.0; 3]);
        // Tint is a multiplier in the shader, not an sRGB display color.
        material.base.base_color = Color::linear_rgba(red, green, blue, alpha * geoset_alpha);
        if let Some(bindings) = bindings {
            material.base.base_color_texture = bindings.bitmap(texture_id as usize);
        }
    }
}

pub(crate) fn advance_animation(time: Res<Time>, mut instances: Query<&mut Wc3Animation>) {
    for mut instance in &mut instances {
        if instance.playing {
            instance.elapsed_ms += time.delta_secs_f64() * 1000.0 * instance.speed;
        }
    }
}

pub(crate) fn track_time<T: TrackValue>(
    track: &Track<T>,
    animation: &Wc3Animation,
) -> Option<(f64, RangeInclusive<i32>)> {
    if let Some(global_id) = track.global_sequence_id() {
        let length = *animation.global_sequences.get(global_id as usize)?;
        if length == 0 {
            return Some((0.0, i32::MIN..=i32::MAX));
        }
        return Some((
            animation.elapsed_ms.rem_euclid(length as f64),
            i32::MIN..=i32::MAX,
        ));
    }
    let sequence = animation.sequences.get(animation.sequence)?;
    let start = sequence.interval[0] as f64;
    let end = sequence.interval[1] as f64;
    let length = (end - start).max(0.0);
    let elapsed = if sequence.flags.non_looping() {
        animation.elapsed_ms.clamp(0.0, length)
    } else if length > 0.0 {
        animation.elapsed_ms.rem_euclid(length)
    } else {
        0.0
    };
    Some((
        start + elapsed,
        sequence.interval[0] as i32..=sequence.interval[1] as i32,
    ))
}

pub(crate) fn sample<T: Interpolate>(track: &Track<T>, animation: &Wc3Animation) -> Option<T> {
    let (time, interval) = track_time(track, animation)?;
    track.evaluate_in(time, interval)
}

pub(crate) fn sample_value<T: Interpolate>(value: &Animatable<T>, animation: &Wc3Animation) -> T {
    value
        .track()
        .and_then(|track| sample(track, animation))
        .or_else(|| value.value().copied())
        .unwrap_or_default()
}

impl AnimatedNode {
    pub(crate) fn camera_anchor(&self) -> Option<Vec3> {
        self.flags
            .camera_anchored()
            .then_some(self.camera)
            .flatten()
            .map(|camera| camera.translation)
    }

    pub(crate) fn pose_sample(&self, camera: Option<Transform>) -> NodeSample {
        NodeSample {
            root: self.root,
            flags: self.flags,
            pivot_offset: self.pivot - self.parent_pivot,
            camera,
        }
    }

    pub(crate) fn sample_transform(&self, animation: &Wc3Animation) -> Transform {
        let translation = self
            .translation
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or([0.0; 3]);
        let rotation = self
            .rotation
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let scaling = self
            .scaling
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or([1.0; 3]);
        Transform {
            translation: self.pivot - self.parent_pivot + Vec3::from_array(translation),
            rotation: Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
            scale: Vec3::from_array(scaling),
        }
    }
}

pub(crate) fn sample_emitter_transform(
    entity: Entity,
    root: Entity,
    animation: &Wc3Animation,
    nodes: &Query<(
        &GlobalTransform,
        Option<&Transform>,
        Option<&AnimatedNode>,
        Option<&ChildOf>,
    )>,
) -> Option<GlobalTransform> {
    let lookup = |entity| {
        let (global, transform, node, parent) = nodes.get(entity).ok()?;
        match node.filter(|node| node.root == root) {
            Some(node) => Some(PoseInput {
                world: None,
                anchor: None,
                local: node.sample_transform(animation),
                parent: parent.map(ChildOf::parent),
                node: Some(node.pose_sample(node.camera)),
            }),
            None if transform.is_some() => Some(PoseInput {
                local: *transform?,
                parent: parent.map(ChildOf::parent),
                node: None,
                world: None,
                anchor: node.and_then(AnimatedNode::camera_anchor),
            }),
            None => {
                let anchor = camera_anchor(entity, |entity| {
                    let (_, _, node, parent) = nodes.get(entity).ok()?;
                    Some((
                        node.and_then(AnimatedNode::camera_anchor),
                        parent.map(ChildOf::parent),
                    ))
                });
                Some(world_input(global, anchor))
            }
        }
    };
    resolve_pose(entity, &lookup, &mut HashMap::new(), &mut HashSet::new())
        .map(|pose| GlobalTransform::from(Mat4::from(pose.affine)))
}

#[cfg(test)]
#[path = "animation_tests.rs"]
mod tests;
