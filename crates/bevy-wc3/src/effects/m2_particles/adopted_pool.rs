//! One common M2 sprite pool and fractional phase shared by all parent drives.
//! This layer has no autonomous clock, model actor or native-resource lookup.
use super::birth::BirthFrame;
use super::templates::FrozenTemplate;
use super::{signed, EmitterInstance, ImplosionInterval};
use crate::effects::age_sprites_gpu::AgeSpriteData;
use crate::effects::frame::EffectFrame;
use crate::effects::simulation::{EmissionPhase, EmissionRounding};
use bevy::prelude::{Mat4, Vec3};
use std::sync::Arc;
use wc3::model::values::model::ParticleGenerator;

pub(super) const POOL_CAPACITY: usize = 4096;
pub(super) const SOURCE_CHECKS: usize = 32768;
pub(super) const CHILD_BIRTHS: usize = 8192;
pub(super) struct AdoptionWork {
    pub source_checks: usize,
    pub child_births: usize,
    pub motion_checks: usize,
}
impl Default for AdoptionWork {
    fn default() -> Self {
        Self {
            source_checks: SOURCE_CHECKS,
            child_births: CHILD_BIRTHS,
            motion_checks: SOURCE_CHECKS,
        }
    }
}
impl AdoptionWork {
    /// Charge candidate examination, including a candidate later found dead.
    pub(super) fn examine(&mut self) -> bool {
        if self.source_checks == 0 {
            return false;
        }
        self.source_checks -= 1;
        true
    }
}
#[derive(Clone, Copy)]
pub(super) struct SourceDrive {
    pub start: f64,
    pub end: f64,
    /// Full owner emitter axes with this source point replacing translation.
    pub placement: Mat4,
    /// Owner model basis; child authored conversion is never appended here.
    pub basis: Mat4,
    pub velocity: Vec3,
    pub emission_scale: f32,
}
pub(super) struct AdoptedPool {
    pub template: Arc<FrozenTemplate>,
    pub storage: EmitterInstance,
    phase: EmissionPhase,
    ordinal: u64,
    drive_ordinal: u64,
    retired_at: Option<f64>,
    next_retirement: f64,
    #[cfg(test)]
    pub(super) retirement_visits: usize,
    owner_initialized: bool,
    source_cutoff: Option<f64>,
    motion_cutoff: Option<f64>,
}
impl AdoptedPool {
    pub(super) fn new(template: Arc<FrozenTemplate>) -> Self {
        Self {
            template,
            storage: EmitterInstance::new(false),
            phase: EmissionPhase::default(),
            ordinal: 0,
            drive_ordinal: 0,
            retired_at: None,
            next_retirement: f64::INFINITY,
            #[cfg(test)]
            retirement_visits: 0,
            owner_initialized: false,
            source_cutoff: None,
            motion_cutoff: None,
        }
    }
    pub(super) fn reset(&mut self) {
        self.storage.births.clear();
        self.storage.age_appearances.clear();
        self.storage.dropped = 0;
        self.storage.skipped_seconds = 0.;
        self.phase.reset();
        self.ordinal = 0;
        self.drive_ordinal = 0;
        self.retired_at = None;
        self.next_retirement = f64::INFINITY;
        #[cfg(test)]
        {
            self.retirement_visits = 0;
        }
        self.owner_initialized = false;
        self.source_cutoff = None;
        self.motion_cutoff = None;
    }
    /// All drives for one quantum use the same end time. Retirement occurs once
    /// per end time, not once per parent. Before any resident can expire, the
    /// cached bound skips the full scan. Motion/appearance sample absolute ages.
    pub(super) fn finish(&mut self, time: f64) {
        if self.retired_at != Some(time) {
            if time >= self.next_retirement {
                #[cfg(test)]
                {
                    self.retirement_visits += self.storage.births.len();
                }
                let mut next = f64::INFINITY;
                self.storage.retire_with(time, |birth| {
                    next = next.min(birth.retirement_bound());
                });
                self.next_retirement = next;
            }
            self.retired_at = Some(time);
        }
    }
    /// Source examination is charged by the bounded source iterator, before it
    /// decides whether this parent survives. This method has no implicit source.
    pub(super) fn drive(&mut self, drive: SourceDrive, work: &mut AdoptionWork) {
        let dt = drive.end - drive.start;
        if !dt.is_finite() || dt <= 0. || !drive.emission_scale.is_finite() {
            return;
        }
        self.finish(drive.end);
        let mut seed =
            self.template.kernel().seed ^ self.drive_ordinal.wrapping_mul(0x517cc1b727220a95);
        self.drive_ordinal = self.drive_ordinal.wrapping_add(1);
        let rate = (self.template.rate
            + f64::from(signed(&mut seed) * self.template.rate_variation))
        .max(0.)
            * f64::from(drive.emission_scale.max(0.));
        let limit = POOL_CAPACITY
            .saturating_sub(self.storage.births.len())
            .min(work.child_births);
        let schedule = self
            .phase
            .continuous(rate, dt, limit, EmissionRounding::Model);
        let first = self.ordinal;
        self.ordinal = self.ordinal.wrapping_add(schedule.authored_count as u64);
        self.storage.dropped = self
            .storage
            .dropped
            .saturating_add((schedule.authored_count - schedule.count).max(0.) as u64);
        work.child_births -= schedule.count as usize;
        for index in 0..schedule.count as usize {
            let frame = BirthFrame {
                time: drive.start + schedule.offset(index),
                node: drive.placement,
                basis: drive.basis,
                inherited: drive.velocity,
                ordinal: first.wrapping_add(index as u64),
            };
            if let Some((birth, appearance)) = self.template.birth(frame) {
                if drive.end - birth.time < f64::from(birth.lifetime) {
                    self.next_retirement = self.next_retirement.min(birth.retirement_bound());
                    self.storage.push_sprite(birth, appearance);
                }
            } else {
                self.storage.dropped = self.storage.dropped.saturating_add(1);
            }
        }
    }
    pub(super) fn begin_frame(&mut self, owner: Mat4) {
        if !self.owner_initialized {
            self.storage.previous_node = owner;
            self.owner_initialized = true;
        }
        self.source_cutoff = None;
        self.motion_cutoff = None;
    }
    pub(super) fn skip_source_from(&mut self, time: f64) {
        self.source_cutoff = Some(
            self.source_cutoff
                .map_or(time, |previous| previous.min(time)),
        );
    }
    /// Once after all source drives: restore one owner frame, then apply the
    /// common analytical follow/implosion policy, never an unrelated A→B frame.
    pub(super) fn advance_owner(
        &mut self,
        start: f64,
        end: f64,
        owner: Mat4,
        unit_scale: f32,
        work: &mut AdoptionWork,
    ) {
        self.finish(end);
        let previous = self.storage.previous_node;
        let kernel = self.template.kernel();
        let follow = kernel.follow_in_units(previous, owner, end - start, unit_scale);
        let filtering =
            kernel.generator == ParticleGenerator::Sphere && kernel.flags.implosion_filter();
        let checks = if follow != Vec3::ZERO || filtering {
            self.storage.births.len()
        } else {
            0
        };
        if self.motion_cutoff.is_some() || checks > work.motion_checks {
            self.motion_cutoff.get_or_insert(start);
        } else {
            work.motion_checks -= checks;
            if follow != Vec3::ZERO {
                for birth in &mut self.storage.births {
                    let fraction = if birth.time > start {
                        ((end - birth.time) / (end - start)).clamp(0., 1.) as f32
                    } else {
                        1.
                    };
                    birth.origin += follow * fraction;
                }
            }
            kernel.retire_implosion(
                &mut self.storage,
                ImplosionInterval {
                    start,
                    end,
                    checked_end: end,
                    centers: [previous.w_axis.truncate(), owner.w_axis.truncate()],
                    follow,
                },
            );
        }
        // Omitting motion consumes the observed interval, without later catch-up.
        self.storage.previous_node = owner;
    }
    pub(super) fn finish_frame(&mut self, time: f64) -> f64 {
        let cutoff = match (self.source_cutoff.take(), self.motion_cutoff.take()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let omitted = cutoff.map_or(0., |start| (time - start).max(0.));
        self.storage.skipped_seconds += omitted;
        self.finish(time);
        omitted
    }
    pub(super) fn publish(
        &mut self,
        frame: &EffectFrame<'_, '_, '_>,
        node: Mat4,
        data: &mut AgeSpriteData,
    ) {
        self.finish(frame.time);
        self.template
            .kernel()
            .sample_gpu(&mut self.storage, frame, node, data);
    }
}
