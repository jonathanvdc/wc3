//! Shared lifecycle for ordinary models owned by particle births.
use bevy::prelude::*;

#[derive(Component, Default)]
pub(crate) struct ParticleModelSubtree;
#[derive(Component, Clone, Copy, Debug)]
#[require(ParticleModelSubtree)]
pub(crate) struct ModelParticleChild {
    pub root: Entity,
    pub birth: f64,
    pub lifetime: f64,
    pub visible: bool,
    pub first_clip_pending: bool,
}
impl ModelParticleChild {
    pub fn parent_model(&self) -> Entity {
        self.root
    }
}
/// Optional closed-form motion used by independent ballistic model particles.
/// Sampled age-driven children receive their Transform in the early effect step.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct ModelParticleMotion {
    pub origin: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl ModelParticleMotion {
    pub fn sample(&self, age: f32) -> Transform {
        Transform {
            translation: self.origin + self.velocity * age + self.acceleration * (0.5 * age * age),
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}

use super::model_ownership::ModelParticleOwner;
use crate::{ModelRig, Playback};

pub(crate) fn hierarchy_visible(
    mut entity: Entity,
    hierarchy: &Query<(&Visibility, Option<&ChildOf>)>,
) -> bool {
    for _ in 0..4096 {
        let Ok((visibility, parent)) = hierarchy.get(entity) else {
            return true;
        };
        match visibility {
            Visibility::Visible => return true,
            Visibility::Hidden => return false,
            Visibility::Inherited => {}
        }
        let Some(parent) = parent else {
            return true;
        };
        entity = parent.parent();
    }
    false
}
/// Current Visibility ancestry, before Bevy's inherited visibility propagation.
pub(crate) fn current_hierarchy_visible(world: &World, mut entity: Entity) -> bool {
    for _ in 0..4096 {
        match world.get::<Visibility>(entity) {
            Some(Visibility::Hidden) => return false,
            Some(Visibility::Visible) => return true,
            _ => {}
        }
        let Some(parent) = world.get::<ChildOf>(entity) else {
            return true;
        };
        entity = parent.parent();
    }
    false
}
/// Synchronize an owned birth after its source's current emission/motion step.
/// Ready and still-loading requests share this lifecycle, so late assets cannot
/// bring a dead or reset particle back to life.
pub(crate) fn synchronize_child(world: &mut World, entity: Entity) -> bool {
    let Some(particle) = world.get::<ModelParticleChild>(entity).copied() else {
        return false;
    };
    let parent_time = world.get::<Playback>(particle.root).map(|playback| {
        world
            .get::<ModelRig>(particle.root)
            .map_or(playback.shared_time, |rig| rig.effect_time(playback))
    });
    let Some(parent_time) =
        parent_time.filter(|_| world.get::<ModelParticleOwner>(entity).is_some())
    else {
        world.despawn(entity);
        return false;
    };
    let age = parent_time - particle.birth;
    if age < 0. || age >= particle.lifetime {
        world.despawn(entity);
        return false;
    }
    let desired = if particle.visible && current_hierarchy_visible(world, particle.root) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if let Some(mut visibility) = world.get_mut::<Visibility>(entity) {
        if *visibility != desired {
            *visibility = desired;
        }
    }
    if let Some(motion) = world.get::<ModelParticleMotion>(entity).copied() {
        if let Some(mut transform) = world.get_mut::<Transform>(entity) {
            let sample = motion.sample(age as f32);
            if *transform != sample {
                *transform = sample;
            }
        }
    }
    let camera = world
        .get::<Playback>(particle.root)
        .and_then(|clock| clock.driving_camera);
    if let Some(mut playback) = world.get_mut::<Playback>(entity) {
        if playback.clip_time != age || playback.shared_time != age {
            playback.clip_time = age;
            playback.shared_time = age;
        }
        if playback.driving_camera != camera {
            playback.driving_camera = camera;
        }
        if !playback.paused {
            playback.paused = true;
        }
    }
    true
}
#[cfg(test)]
fn synchronize_model_particles(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<ModelParticleChild>>();
    let children: Vec<_> = query.iter(world).collect();
    for entity in children {
        synchronize_child(world, entity);
    }
}

#[cfg(test)]
#[path = "model_children_tests.rs"]
mod tests;
