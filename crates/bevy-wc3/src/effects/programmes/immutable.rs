//! Projected billboard appearance and immutable birth publication.
mod bounds;
mod wgsl;
use super::prepared::{ProgrammeRenderer, RendererGeometry};
use super::runtime::sprite_record;
use super::MODEL_UNITS_PER_EFFECT_UNIT;
use crate::effects::age_sprites_gpu::immutable::{
    BirthSlots, ImmutableAppearance, ImmutableSpriteData,
};
use crate::effects::age_sprites_gpu::{AgeSpriteData, AgeSpriteSource, AgeSpriteUniform};
use crate::effects::snapshots::SnapshotPool;
use bevy::prelude::*;
use bytemuck::cast_slice;
use std::slice::from_ref;
use std::sync::{Arc, OnceLock};
use wc3::model::scene::popcorn::{
    EffectInstance, EffectProgram, ImmutableExpression, ImmutableProjectionScratch, Value,
};

pub(super) fn prepare(
    program: &EffectProgram,
    renderer: &ProgrammeRenderer,
) -> Option<Arc<ImmutableAppearance>> {
    let RendererGeometry::Billboard {
        facing,
        axes,
        rotation,
    } = renderer.geometry
    else {
        return None;
    };
    let recipe = program.immutable_recipe(renderer.layer)?;
    if renderer.atlas[0].checked_mul(renderer.atlas[1])? > 1 << 24 {
        return None;
    }
    if renderer
        .enabled
        .is_some_and(|field| !bounds::stationary(recipe, field))
    {
        return None;
    }
    // Stationary outputs are already in the initialized/evolved header. Only
    // changing renderer outputs need shader expressions or birth captures.
    let dynamic = |field| !bounds::stationary(recipe, field);
    let fields = wgsl::AppearanceFields {
        position: dynamic(renderer.position).then_some(renderer.position),
        axes: axes.map(|field| field.filter(|field| dynamic(*field))),
        longitudinal_axis: facing.longitudinal_axis(),
        size: dynamic(renderer.size).then_some(renderer.size),
        color: renderer.color.filter(|field| dynamic(*field)),
        rotation: rotation.filter(|field| dynamic(*field)),
        texture: renderer.texture_id.filter(|field| dynamic(*field)),
    };
    let position_projection = fields
        .position
        .map(|field| recipe.project(&[field]))
        .transpose()
        .ok()?;
    let outputs: Vec<_> = [
        fields.position,
        fields.axes[0],
        fields.axes[1],
        fields.size,
        fields.color,
        fields.rotation,
        fields.texture,
    ]
    .into_iter()
    .flatten()
    .collect();
    let projection = recipe.project(&outputs).ok()?;
    let recipe = projection.recipe(recipe).ok()?;
    let prefix = wgsl::emit(recipe, fields)?;
    let requires_clock = (0..recipe.node_count()).any(|index| {
        recipe.node(index).is_some_and(|node| {
            matches!(
                node.expression,
                ImmutableExpression::Time | ImmutableExpression::Life
            )
        })
    });
    Some(Arc::new(ImmutableAppearance {
        source: format!(
            "{}\n{prefix}",
            include_str!("../../shaders/wc3_age_particle.wgsl")
        ),
        words_per_slot: wgsl::HEADER_WORDS + recipe.capture_count() * 4,
        requires_time_horizon: (0..recipe.node_count()).any(|index| {
            recipe
                .node(index)
                .is_some_and(|node| matches!(node.expression, ImmutableExpression::Time))
        }),
        position_projection,
        requires_clock,
        projection,
        shader: OnceLock::new(),
    }))
}

/// Transport state follows actual core authorization, never an inferred GPU age.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum AppearancePhase {
    Initialized = 0,
    Evolved = 1,
}

#[derive(Clone, Copy)]
struct BirthAdmission {
    generation: u32,
    serial: u64,
    enabled: bool,
    phase: AppearancePhase,
}
fn stationary_enabled(
    cached: &mut Option<BirthAdmission>,
    generation: u32,
    serial: u64,
    phase: AppearancePhase,
    sample: impl FnOnce() -> bool,
) -> bool {
    if let Some(value) = cached.filter(|value| {
        value.generation == generation && value.serial == serial && value.phase == phase
    }) {
        return value.enabled;
    }
    let enabled = sample();
    *cached = Some(BirthAdmission {
        generation,
        serial,
        enabled,
        phase,
    });
    enabled
}

pub(super) struct ImmutableRendererState {
    appearance: Arc<ImmutableAppearance>,
    position_scratch: Option<ImmutableProjectionScratch>,
    centers: SnapshotPool<Vec3>,
    slots: BirthSlots,
    order: Vec<u32>,
    row: Vec<u32>,
    disabled: bool,
    admission: Vec<Option<BirthAdmission>>,
    certified_until: f32,
    #[cfg(test)]
    certifications: usize,
}
impl ImmutableRendererState {
    pub fn new(appearance: Arc<ImmutableAppearance>, capacity: usize) -> Option<Self> {
        Some(Self {
            position_scratch: appearance
                .position_projection
                .as_ref()
                .map(|projection| projection.new_scratch()),
            centers: Default::default(),
            slots: BirthSlots::new(capacity, appearance.words_per_slot)?,
            row: vec![0; appearance.words_per_slot],
            order: Vec::with_capacity(capacity),
            appearance,
            disabled: false,
            admission: vec![None; capacity],
            certified_until: 0.,
            #[cfg(test)]
            certifications: 0,
        })
    }
    pub fn reset(&mut self) {
        self.disabled = false;
        self.admission.fill(None);
        self.certified_until = 0.;
        #[cfg(test)]
        {
            self.certifications = 0;
        }
        self.slots.reset();
    }
    /// False means publish normal CPU samples this observation. A failed finite
    /// certificate pins this renderer to samples until reset, without a core fault.
    pub fn publish(
        &mut self,
        renderer: &ProgrammeRenderer,
        instance: &EffectInstance,
        world_from_space: Mat4,
        uniform: AgeSpriteUniform,
        data: &mut AgeSpriteData,
    ) -> bool {
        if self.disabled {
            return false;
        }
        // Each visible lane must either have an authorized sample at this exact
        // observation or be a just-initialized endpoint birth. CPU fallback
        // latches immutable_active off before any unauthorized advancement.
        if !instance.immutable_active() || instance.immutable_recipe(renderer.layer).is_none() {
            return false;
        }
        let authorized_time = instance.time();
        let time = authorized_time as f32;
        if !time.is_finite() || time < 0. {
            self.disabled = true;
            return false;
        }
        // Life-only recipes are admitted over their entire [0,1] domain once
        // per birth. Only explicit Time nodes require a growing clock horizon.
        let recertify = self.appearance.requires_time_horizon
            && (self.certified_until == 0. || time > self.certified_until);
        let horizon = if !self.appearance.requires_time_horizon {
            f32::MAX
        } else if recertify {
            (f64::from(time) * 2. + 1.).min(f64::from(f32::MAX)) as f32
        } else {
            self.certified_until
        };
        self.order.clear();
        let centers = self
            .appearance
            .position_projection
            .as_ref()
            .map(|_| self.centers.begin());
        let mut centers = centers;
        for particle in instance.particles(renderer.layer) {
            let immutable = particle.immutable_sample();
            let phase = match immutable {
                Some(view) if view.sample().effect_time == authorized_time => {
                    AppearancePhase::Evolved
                }
                None if particle.birth_time == authorized_time => AppearancePhase::Initialized,
                _ => return false,
            };
            let slot = particle.id.slot;
            let Some(admission) = self.admission.get_mut(slot as usize) else {
                self.disabled = true;
                return false;
            };
            let fresh = !self
                .slots
                .matches(slot, particle.id.generation, particle.birth_serial)
                || admission.as_ref().is_none_or(|value| {
                    value.phase != phase
                        || value.generation != particle.id.generation
                        || value.serial != particle.birth_serial
                });
            if !stationary_enabled(
                admission,
                particle.id.generation,
                particle.birth_serial,
                phase,
                || {
                    !renderer
                        .enabled
                        .and_then(|field| particle.field(field))
                        .is_some_and(|value| value.words()[0] == 0)
                },
            ) {
                continue;
            }
            let projected = if let Some(view) = immutable {
                match self.appearance.projection.view(view) {
                    Ok(projected) => Some(projected),
                    Err(_) => return false,
                }
            } else {
                None
            };
            if let Some(immutable) = projected {
                let admitted = if fresh || recertify {
                    #[cfg(test)]
                    {
                        self.certifications += 1;
                    }
                    bounds::admit(immutable, renderer, world_from_space, horizon).is_some()
                } else {
                    true
                };
                if !admitted {
                    self.disabled = true;
                    return false;
                }
            }
            if fresh {
                let Some(base) = sprite_record(renderer, &particle, world_from_space) else {
                    if phase == AppearancePhase::Initialized {
                        // Authored initial geometry can be invalid/hidden yet its
                        // first evolved geometry valid. Omit only this phase.
                        continue;
                    }
                    self.disabled = true;
                    return false;
                };
                self.row.fill(0);
                self.row[..28].copy_from_slice(cast_slice(from_ref(&base)));
                self.row[wgsl::PHASE_WORD] = phase as u32;
                if let Some(immutable) = projected {
                    let birth = immutable.birth();
                    self.row[28..31].copy_from_slice(&[
                        birth.time.to_bits(),
                        birth.life.to_bits(),
                        birth.inverse_life.to_bits(),
                    ]);
                    for index in 0..immutable.recipe().capture_count() {
                        let Some(value) = immutable.capture(index) else {
                            self.disabled = true;
                            return false;
                        };
                        self.row
                            [wgsl::HEADER_WORDS + index * 4..wgsl::HEADER_WORDS + index * 4 + 4]
                            .copy_from_slice(&value.words());
                    }
                }
                if self
                    .slots
                    .set(
                        slot,
                        particle.id.generation,
                        particle.birth_serial,
                        &self.row,
                    )
                    .is_none()
                {
                    self.disabled = true;
                    return false;
                }
            }
            if let Some(centers) = &mut centers {
                let position = if let Some(view) = immutable {
                    let mut values = [Value::from(0f32)];
                    let projection = self
                        .appearance
                        .position_projection
                        .as_ref()
                        .expect("moving position projection");
                    if projection
                        .evaluate(
                            view,
                            self.position_scratch.as_mut().expect("position scratch"),
                            &mut values,
                        )
                        .is_err()
                    {
                        self.disabled = true;
                        return false;
                    }
                    world_from_space.transform_point3(
                        Vec3::from_slice(&values[0].words().map(f32::from_bits))
                            * MODEL_UNITS_PER_EFFECT_UNIT,
                    )
                } else {
                    // Endpoint newborns keep their authored initial position.
                    let Some(value) = particle.field(renderer.position) else {
                        return false;
                    };
                    world_from_space.transform_point3(
                        Vec3::from_slice(&value.words().map(f32::from_bits))
                            * MODEL_UNITS_PER_EFFECT_UNIT,
                    )
                };
                if !position.is_finite() {
                    self.disabled = true;
                    return false;
                }
                Arc::get_mut(centers)
                    .expect("released center snapshot")
                    .push(position);
            }
            self.order.push(slot);
        }
        self.certified_until = horizon;
        self.slots.set_live(&self.order);
        let centers = centers.map(|centers| {
            if let AgeSpriteSource::Immutable(previous) = &data.source {
                if let Some(previous) = previous.centers.as_ref().filter(|previous| {
                    previous.len() == centers.len()
                        && previous.iter().zip(centers.iter()).all(|(a, b)| {
                            a.to_array().map(f32::to_bits) == b.to_array().map(f32::to_bits)
                        })
                }) {
                    return previous.clone();
                }
            }
            centers.clone()
        });
        let rows = world_from_space.transpose().to_cols_array_2d();
        data.source = AgeSpriteSource::Immutable(ImmutableSpriteData {
            appearance: self.appearance.clone(),
            births: self.slots.snapshot.clone(),
            centers,
            world_from_space: [rows[0], rows[1], rows[2]],
        });
        data.live_indices = self.slots.live.clone();
        data.uniform = uniform;
        data.uniform
            .set_immutable_time(if self.appearance.requires_clock {
                time
            } else {
                0.
            });
        true
    }
}

#[cfg(test)]
mod tests;
