//! Ordinary Bevy model instances owned by shared age-driven particle births.
use super::m2_model_geometry::ParticleKey;
use super::model_children::{ModelParticleChild, ParticleModelSubtree};
use super::model_ownership::spawn_owned_model;
use crate::child_coordinates::ChildCoordinates;
use crate::{ModelAsset, ModelInstance, Playback};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub(super) struct ModelTemplate {
    pub model: Handle<ModelAsset>,
    pub coordinates: ChildCoordinates,
}
#[derive(Clone, Copy)]
pub(super) struct ModelProjection {
    pub ordinal: u64,
    pub birth: f64,
    pub lifetime: f64,
    pub age: f64,
    pub transform: Transform,
    pub visible: bool,
}
#[derive(Clone, Copy)]
pub(super) struct ModelProjectionContext {
    pub root: Entity,
    pub owner: Entity,
    pub visible: bool,
    pub driving_camera: Option<Entity>,
}
pub(crate) type ModelOutputs<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut ModelParticleChild,
        &'static mut Playback,
    ),
    With<ParticleModelSubtree>,
>;
struct Slot {
    entity: Entity,
    seen: u64,
    transform: Transform,
    visible: bool,
    effective_visible: bool,
}
#[derive(Default)]
pub(super) struct ModelChildren {
    epoch: u64,
    generation: u64,
    slots: HashMap<ParticleKey, Slot>,
}
impl ModelChildren {
    pub fn reset(&mut self, commands: &mut Commands) {
        self.epoch = self.epoch.wrapping_add(1);
        for (_, slot) in self.slots.drain() {
            commands.entity(slot.entity).try_despawn();
        }
    }
    pub fn synchronize(
        &mut self,
        commands: &mut Commands,
        context: ModelProjectionContext,
        template: &ModelTemplate,
        outputs: &mut ModelOutputs,
        particles: impl IntoIterator<Item = ModelProjection>,
    ) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        for sample in particles {
            let key = ParticleKey {
                epoch: self.epoch,
                ordinal: sample.ordinal,
            };
            let effective_visible = sample.visible && context.visible;
            let slot = self.slots.entry(key).or_insert_with(|| {
                let entity = spawn_owned_model(
                    commands,
                    context.owner,
                    (
                        ModelInstance(template.model.clone()),
                        template.coordinates,
                        ModelParticleChild {
                            root: context.root,
                            birth: sample.birth,
                            lifetime: sample.lifetime,
                            visible: sample.visible,
                            first_clip_pending: true,
                        },
                        Playback {
                            clip_time: sample.age,
                            shared_time: sample.age,
                            paused: true,
                            driving_camera: context.driving_camera,
                            ..Default::default()
                        },
                        sample.transform,
                        if effective_visible {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        },
                    ),
                );
                Slot {
                    entity,
                    seen: generation,
                    transform: sample.transform,
                    visible: sample.visible,
                    effective_visible,
                }
            });
            slot.seen = generation;
            if let Ok((mut transform, mut child, mut clock)) = outputs.get_mut(slot.entity) {
                if *transform != sample.transform {
                    *transform = sample.transform;
                }
                let desired = if effective_visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if slot.effective_visible != effective_visible {
                    commands.entity(slot.entity).insert(desired);
                }
                if child.visible != sample.visible {
                    child.visible = sample.visible;
                }
                if clock.clip_time != sample.age || clock.shared_time != sample.age {
                    clock.clip_time = sample.age;
                    clock.shared_time = sample.age;
                }
            }
            slot.transform = sample.transform;
            slot.visible = sample.visible;
            slot.effective_visible = effective_visible;
        }
        self.slots.retain(|_, slot| {
            if slot.seen == generation {
                true
            } else {
                commands.entity(slot.entity).try_despawn();
                false
            }
        });
    }
}
