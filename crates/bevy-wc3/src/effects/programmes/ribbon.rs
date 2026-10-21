//! Chronological strands and view-independent ribbon segment records.
use super::sampling::atlas_sample;
use crate::effects::age_sprites_gpu::{AgeSpriteRecord, RibbonSegmentParameters};
use bevy::prelude::*;
use wc3::model::scene::popcorn::EffectParticleId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RibbonMode {
    View,
    NormalAxis,
    SideAxis,
}
impl RibbonMode {
    pub(super) fn code(self) -> u32 {
        match self {
            Self::View => 0,
            Self::NormalAxis => 1,
            Self::SideAxis => 2,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct RibbonSample {
    pub parent: Option<EffectParticleId>,
    pub serial: u64,
    pub position: Vec3,
    pub half_width: f32,
    pub color: Vec4,
    pub axis: Vec3,
    pub texture_u: Option<f32>,
    pub texture_id: f32,
    pub cursor: f32,
}
#[derive(Default)]
pub(super) struct RibbonSamples {
    points: Vec<RibbonSample>,
}
fn parent_key(parent: Option<EffectParticleId>) -> (bool, u32, u32) {
    parent.map_or((false, 0, 0), |id| (true, id.slot, id.generation))
}
fn unit_or_zero(value: Vec3) -> Vec3 {
    let scale = value.abs().max_element();
    if scale == 0. {
        Vec3::ZERO
    } else {
        (value / scale).normalize()
    }
}
fn unit(value: Vec3) -> Vec3 {
    let value = unit_or_zero(value);
    if value == Vec3::ZERO {
        Vec3::X
    } else {
        value
    }
}
impl RibbonSamples {
    pub(super) fn begin(&mut self) {
        self.points.clear();
    }
    /// Disabled particles are filtered before collection and bridge surviving neighbors.
    pub(super) fn push(&mut self, point: RibbonSample) -> bool {
        if !point.position.is_finite()
            || !(point.position.abs().max_element() + point.half_width).is_finite()
            || !point.half_width.is_finite()
            || point.half_width <= 0.
            || !point.axis.is_finite()
            || !point.color.is_finite()
            || point.texture_u.is_some_and(|value| !value.is_finite())
            || !point.cursor.is_finite()
        {
            return false;
        }
        self.points.push(point);
        true
    }
    /// Returns collapsed adjacent segments omitted from the submitted geometry.
    pub(super) fn emit(
        &mut self,
        mode: RibbonMode,
        atlas_cells: u32,
        records: &mut Vec<AgeSpriteRecord>,
        indices: &mut Vec<u32>,
    ) -> usize {
        self.points
            .sort_unstable_by_key(|point| (parent_key(point.parent), point.serial));
        let mut collapsed = 0;
        let mut previous_direction = None;
        for index in 0..self.points.len().saturating_sub(1) {
            let start = self.points[index];
            let end = self.points[index + 1];
            if start.parent != end.parent {
                previous_direction = None;
                continue;
            }
            let outgoing = end.position - start.position;
            if outgoing == Vec3::ZERO || !outgoing.is_finite() {
                collapsed += 1;
                continue;
            }
            let incoming = previous_direction.unwrap_or(outgoing);
            previous_direction = Some(outgoing);
            let sections = match mode {
                RibbonMode::View => [unit(incoming), unit(outgoing)],
                RibbonMode::NormalAxis => [
                    unit(unit_or_zero(start.axis).cross(unit_or_zero(incoming))),
                    unit(unit_or_zero(end.axis).cross(unit_or_zero(outgoing))),
                ],
                RibbonMode::SideAxis => [unit(start.axis), unit(end.axis)],
            };
            let (cell, _, _) = atlas_sample(start.texture_id, atlas_cells);
            let record = AgeSpriteRecord::ribbon_segment(RibbonSegmentParameters {
                start: start.position,
                end: end.position,
                half_widths: [start.half_width, end.half_width],
                sections,
                colors: [start.color.max(Vec4::ZERO), end.color.max(Vec4::ZERO)],
                texture_u: [start.texture_u.unwrap_or(0.), end.texture_u.unwrap_or(1.)],
                atlas_cell: cell,
                cursors: [start.cursor, end.cursor],
            });
            indices.push(records.len() as u32);
            records.push(record);
        }
        collapsed
    }
}

#[cfg(test)]
#[path = "ribbon_tests.rs"]
mod tests;
