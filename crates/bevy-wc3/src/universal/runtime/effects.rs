use super::super::effects::model_particles::hierarchy_visible;
use super::super::effects::{EffectFrame, EffectWorld};
use super::super::pose::PoseContext;
use super::{UniversalInstance, UniversalPlayback, UniversalPoseOverrides};
use bevy::prelude::*;

pub(crate) fn simulate_universal_particles(
    mut world: EffectWorld<'_, '_>,
    mut instances: Query<(
        Entity,
        &mut UniversalInstance,
        &UniversalPlayback,
        &UniversalPoseOverrides,
    )>,
) {
    for (root, mut instance, playback, overrides) in &mut instances {
        if instance.model.particles.is_empty() {
            continue;
        }
        let animation = instance.model.animation.clone();
        let timing = instance.model.timing.clone();
        let Ok(selection) = playback.selection_from_timing(&timing) else {
            continue;
        };
        let Ok(sample) = animation.sample(selection) else {
            continue;
        };
        let Ok(basis) = world.globals.get(instance.basis_entity) else {
            continue;
        };
        let camera = playback
            .driving_camera
            .and_then(|camera| world.globals.get(camera).ok())
            .map(GlobalTransform::to_matrix);
        let UniversalInstance {
            model,
            poses,
            visible,
            particles,
            transition,
            ..
        } = &mut *instance;
        model.particles.update(
            particles,
            root,
            EffectFrame {
                root_visible: hierarchy_visible(root, &world.hierarchy),
                traversal: playback.event_traversal,
                change: Default::default(),
                time: playback.shared_time,
                selection,
                sample: &sample,
                pose: Some(PoseContext {
                    nodes: &model.nodes,
                    transition: transition.view_at(playback.transition_clock),
                    overrides: &overrides.0,
                    camera: camera.and_then(|camera| {
                        let basis = basis.to_matrix();
                        (basis.determinant().abs() > 1e-12)
                            .then(|| GlobalTransform::from(basis.inverse() * camera).affine())
                    }),
                }),
                node_poses: poses,
                node_visible: visible,
                basis_world: basis.to_matrix(),
                camera_world: camera,
            },
            &mut world,
        );
    }
}
