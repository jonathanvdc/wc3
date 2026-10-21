use super::super::effects::model_children::hierarchy_visible;
use super::super::effects::{EffectFrame, EffectWorld};
use super::super::pose::PoseContext;
use super::model_driver::ModelParticleDriver;
use super::{ModelRig, Playback, PoseOverrides};
use bevy::prelude::*;

pub(crate) fn simulate_particles(
    mut world: EffectWorld<'_, '_>,
    mut instances: Query<(
        Entity,
        &mut ModelRig,
        &Playback,
        &PoseOverrides,
        Option<&mut ModelParticleDriver>,
    )>,
) {
    for (root, mut instance, playback, overrides, mut driver) in &mut instances {
        if instance.model.particles.is_empty() {
            continue;
        }
        let prepared = match driver.as_mut() {
            Some(driver) => {
                let Some(prepared) = driver.prepared.take() else {
                    continue;
                };
                Some(prepared)
            }
            None => None,
        };
        let animation = instance.model.animation.clone();
        let Ok(selection) = prepared
            .map(|step| Ok(step.selection))
            .unwrap_or_else(|| playback.selection())
        else {
            continue;
        };
        let Ok(sample) = animation.sample(selection) else {
            continue;
        };
        let Ok(basis) = world.globals.get(instance.basis_entity) else {
            continue;
        };
        let basis_world = prepared.map_or(basis.to_matrix(), |step| step.basis);
        let camera = prepared.map(|step| step.camera).unwrap_or_else(|| {
            playback
                .driving_camera
                .and_then(|camera| world.globals.get(camera).ok())
                .map(GlobalTransform::to_matrix)
        });
        let ModelRig {
            model,
            poses,
            visible,
            particles,
            transition,
            ..
        } = &mut *instance;
        let frame = EffectFrame {
            root_visible: prepared.map_or_else(
                || hierarchy_visible(root, &world.hierarchy),
                |step| step.visible,
            ),
            traversal: playback.event_traversal,
            change: prepared.map_or(Default::default(), |step| step.change),
            time: prepared.map_or(playback.shared_time, |step| step.time),
            selection,
            sample: &sample,
            pose: Some(PoseContext {
                nodes: &model.nodes,
                transition: transition.view_at(playback.transition_clock),
                overrides: &overrides.0,
                camera: camera.and_then(|camera| {
                    (basis_world.determinant().abs() > 1e-12)
                        .then(|| GlobalTransform::from(basis_world.inverse() * camera).affine())
                }),
            }),
            node_poses: poses,
            node_visible: visible,
            basis_world,
        };
        if prepared.is_some() {
            model
                .particles
                .update_after_models(particles, root, frame, &mut world);
        } else {
            model.particles.update(particles, root, frame, &mut world);
        }
    }
}
