//! Bounded source-only motion snapshots spanning parent endpoint retirement.
use super::adopted_pool::{AdoptionWork, SourceDrive};
use super::{Birth, Emitter, MAX_BIRTHS_PER_UPDATE, MAX_PARTICLES};
use crate::effects::implosion::outward_crossing;
use bevy::prelude::{Mat4, Vec3};
use wc3::model::values::model::ParticleGenerator;

const MAX_SOURCES: usize = MAX_PARTICLES + MAX_BIRTHS_PER_UPDATE;
struct SourceSpan {
    birth: Birth,
    active: bool,
}
#[derive(Default)]
pub(super) struct SourceSpans {
    spans: Vec<SourceSpan>,
}
#[derive(Clone, Copy)]
pub(super) struct SourceStep<'a> {
    pub emitter: &'a Emitter,
    pub start: f64,
    pub end: f64,
    pub frame_start: f64,
    pub frame_end: f64,
    /// Full outer carry already included in snapshot origins.
    pub follow: Vec3,
    pub centers: [Vec3; 2],
    /// Common occurrence-sampled owner emitter frame and model basis.
    pub node: Mat4,
    pub basis: Mat4,
}
impl SourceSpans {
    pub(super) fn reset(&mut self) {
        self.spans.clear();
    }
    /// Reuse scratch capacity, copying only source motion. No geometry lease or
    /// parent appearance sidecar is retained. Call BEFORE endpoint retirement.
    pub(super) fn begin(&mut self, existing: &[Birth], follow: Vec3) {
        self.spans.clear();
        for birth in existing {
            self.record(birth, follow);
        }
    }
    /// Observe an admitted, constructed birth BEFORE the final-time lifespan
    /// filter. The normal parent path receives the same remaining follow carry.
    pub(super) fn newborn(&mut self, birth: &Birth, follow: Vec3, fraction: f32) {
        self.record(birth, follow * (1. - fraction.clamp(0., 1.)));
    }
    fn record(&mut self, birth: &Birth, carry: Vec3) {
        // Normal parent admission bounds existing residents plus constructed
        // births to this exact sidecar limit. Keep the release guard too.
        debug_assert!(self.spans.len() < MAX_SOURCES);
        if self.spans.len() >= MAX_SOURCES {
            return;
        }
        let mut birth = birth.clone();
        birth.origin += carry;
        self.spans.push(SourceSpan {
            birth,
            active: true,
        });
    }
    /// Return false when candidate examination is exhausted. The caller stops
    /// further adoption for this update, without stopping normal parent output.
    pub(super) fn drive(
        &mut self,
        step: SourceStep<'_>,
        work: &mut AdoptionWork,
        mut emit: impl FnMut(SourceDrive, &mut AdoptionWork),
    ) -> bool {
        let fraction = |time: f64| {
            if step.frame_end > step.frame_start {
                ((time - step.frame_start) / (step.frame_end - step.frame_start)).clamp(0., 1.)
                    as f32
            } else {
                1.
            }
        };
        for source in &mut self.spans {
            if !work.examine() {
                return false;
            }
            if !source.active || source.birth.time > step.end {
                continue;
            }
            if step.end - source.birth.time >= f64::from(source.birth.lifetime) {
                source.active = false;
                continue;
            }
            let position = |time| {
                step.emitter.motion(&source.birth, time).0 - step.follow * (1. - fraction(time))
            };
            if step.emitter.generator == ParticleGenerator::Sphere
                && step.emitter.flags.implosion_filter()
            {
                let result = outward_crossing(
                    step.start.max(source.birth.time),
                    step.end,
                    position,
                    |time| {
                        if step.emitter.flags.attached() {
                            Vec3::ZERO
                        } else {
                            step.centers[0].lerp(step.centers[1], fraction(time))
                        }
                    },
                );
                debug_assert_eq!(result.skipped_seconds, 0.);
                if result.time.is_some() {
                    source.active = false;
                    continue;
                }
            }
            let point = position(step.end);
            let (_, velocity) = step.emitter.motion(&source.birth, step.end);
            let (point, velocity) = if step.emitter.flags.attached() {
                (
                    step.node.transform_point3(point),
                    step.node.transform_vector3(velocity),
                )
            } else {
                (point, velocity)
            };
            let mut placement = step.node;
            placement.w_axis = point.extend(1.);
            emit(
                SourceDrive {
                    start: step.start.max(source.birth.time),
                    end: step.end,
                    placement,
                    basis: step.basis,
                    velocity,
                    emission_scale: 1.,
                },
                work,
            );
        }
        true
    }
}
