//! Sample historical node poses only when emitting a new cross-section.
use bevy::prelude::*;
use std::sync::Arc;
use wc3::model::animation::TextureAnimation;
use wc3::model::emitters::RibbonEmitter;
use wc3::model::materials::Layer;
use wc3::model::V1800;

use crate::effect_ring::RingCursor;
use crate::effects::simulation::{
    birth_animation, sample_clock, sequence_ended, simulation_delta, split_time, EmissionPhase,
    EmissionRounding, LiveRecords, SimulationClock,
};

use super::render::{RibbonInstances, RibbonSection, RibbonSegment, RibbonUniform};
use crate::animation::{
    sample, sample_emitter_transform, sample_value, AnimatedNode, Wc3Animation,
};
use crate::texture_bindings::Wc3TextureBindings;

const MAX_SECTIONS: usize = 8192;

struct Section {
    birth: f64,
    chain: u64,
}

#[derive(Component)]
pub(crate) struct RibbonState {
    pub(crate) root: Entity,
    pub(crate) node: Entity,
    pub(crate) definition: RibbonEmitter,
    live: LiveRecords<RibbonSection, Section>,
    segments: Arc<Vec<RibbonSegment>>,
    segments_version: Option<(RingCursor, usize)>,
    clock: SimulationClock,
    emission: EmissionPhase,
    chain: u64,
    connected: bool,
}

impl RibbonState {
    pub(crate) fn new(root: Entity, node: Entity, definition: RibbonEmitter) -> Self {
        Self {
            root,
            node,
            definition,
            live: LiveRecords::default(),
            segments: Arc::default(),
            segments_version: None,
            clock: SimulationClock::default(),
            emission: EmissionPhase::default(),
            chain: 0,
            connected: false,
        }
    }

    fn break_chain(&mut self) {
        if self.connected {
            self.chain = self.chain.wrapping_add(1);
            self.connected = false;
        }
    }

    fn retire(&mut self, time: f64) {
        let lifespan = f64::from(self.definition.life_span);
        self.live
            .retire(|section| time - section.birth + 1e-9 >= lifespan);
    }

    fn insert(&mut self, record: RibbonSection, birth: f64) {
        if self.live.len() >= MAX_SECTIONS {
            self.live.pop_front();
        }
        self.live.push(
            record,
            Section {
                birth,
                chain: self.chain,
            },
            MAX_SECTIONS,
        );
        self.connected = true;
    }

    fn rebuild_segments(&mut self) {
        let version = (self.live.records.cursor(), self.live.records.len());
        if self.segments_version == Some(version) {
            return;
        }
        self.segments_version = Some(version);
        let mut segments = Vec::with_capacity(self.live.len().saturating_sub(1));
        let sections = self.live.metadata();
        let mut start = 0;
        while start < sections.len() {
            let mut end = start + 1;
            while end < sections.len() && sections[end].chain == sections[start].chain {
                end += 1;
            }
            let count = (end - start - 1) as u32;
            for i in start..end - 1 {
                segments.push(RibbonSegment([
                    self.live.records.index(i),
                    self.live.records.index(i + 1),
                    (i - start) as u32,
                    count,
                ]));
            }
            start = end;
        }
        if self.segments.as_slice() != segments {
            self.segments = Arc::new(segments);
        }
    }

    fn advance(
        &mut self,
        animation: &Wc3Animation,
        dt: f64,
        mut transform_at: impl FnMut(&Wc3Animation) -> GlobalTransform,
    ) {
        if self.clock.observe(animation) {
            self.break_chain();
            self.emission.reset();
        }
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let sampled_animation = sample_clock(animation.clone());
        let animation = &sampled_animation;
        let start = self.clock.time;
        self.clock.advance(dt);
        self.retire(self.clock.time);
        let rate = f64::from(self.definition.emission_rate);
        let lifespan = f64::from(self.definition.life_span);
        if rate <= 0.0 || !lifespan.is_finite() || lifespan <= 0.0 {
            self.break_chain();
            self.rebuild_segments();
            return;
        }
        let schedule = self
            .emission
            .continuous(rate, dt, MAX_SECTIONS, EmissionRounding::Ribbon);
        let first_birth = schedule.first_birth(start);
        let skip = schedule.skip(first_birth, self.clock.time, lifespan, MAX_SECTIONS);
        if skip > 0.0 {
            self.break_chain();
        }
        for index in 0..(schedule.count - skip) as usize {
            let birth = schedule.birth(first_birth, skip + index as f64);
            let at_birth = sample_clock(birth_animation(animation, self.clock.time - birth));
            let ended = sequence_ended(&at_birth);
            let visible = self
                .definition
                .visibility
                .as_ref()
                .and_then(|track| sample(track, &at_birth))
                .unwrap_or(1.0);
            if ended || !visible.is_finite() || visible <= 0.0 {
                self.break_chain();
                continue;
            }
            let above = sample_value(&self.definition.height_above, &at_birth);
            let below = sample_value(&self.definition.height_below, &at_birth);
            let transform = transform_at(&at_birth);
            // The Bevy node transform already includes its pivot.
            let above = transform.transform_point(Vec3::Y * above);
            let below = transform.transform_point(-Vec3::Y * below);
            if !above.is_finite() || !below.is_finite() {
                self.break_chain();
                continue;
            }
            self.insert(RibbonSection::new(above, below, birth), birth);
        }
        if self
            .definition
            .visibility
            .as_ref()
            .and_then(|track| sample(track, animation))
            .is_some_and(|visible| !visible.is_finite() || visible <= 0.0)
        {
            self.break_chain();
        }
        self.rebuild_segments();
    }
}

/// Each material layer draws the same sections with independent animated state.
#[derive(Component)]
pub(crate) struct RibbonLayer {
    pub(crate) emitter: Entity,
    pub(crate) definition: Layer<V1800>,
    pub(crate) texture_animation: Option<TextureAnimation>,
}

fn uv_transform(definition: Option<&TextureAnimation>, animation: &Wc3Animation) -> Mat4 {
    let Some(definition) = definition else {
        return Mat4::IDENTITY;
    };
    let translation = definition
        .translation
        .as_ref()
        .and_then(|track| sample(track, animation))
        .map(Vec3::from_array)
        .unwrap_or(Vec3::ZERO);
    let rotation = definition
        .rotation
        .as_ref()
        .and_then(|track| sample(track, animation))
        .map(Quat::from_array)
        .unwrap_or(Quat::IDENTITY)
        .normalize();
    let scale = definition
        .scaling
        .as_ref()
        .and_then(|track| sample(track, animation))
        .map(Vec3::from_array)
        .unwrap_or(Vec3::ONE);
    let pivot = Vec3::new(0.5, 0.5, 0.0);
    Mat4::from_translation(translation + pivot)
        * Mat4::from_scale_rotation_translation(scale, rotation, Vec3::ZERO)
        * Mat4::from_translation(-pivot)
}

pub(crate) fn update_ribbons(
    time: Res<Time>,
    animations: Query<(&Wc3Animation, Option<&Wc3TextureBindings>)>,
    nodes: Query<(
        &GlobalTransform,
        Option<&Transform>,
        Option<&AnimatedNode>,
        Option<&ChildOf>,
    )>,
    mut emitters: Query<&mut RibbonState>,
    mut layers: Query<(&RibbonLayer, &mut RibbonInstances)>,
) {
    for mut state in &mut emitters {
        let Ok((animation, _)) = animations.get(state.root) else {
            continue;
        };
        let Ok((transform, _, _, _)) = nodes.get(state.node) else {
            continue;
        };
        let dt = simulation_delta(&time, animation);
        let node = state.node;
        let root = state.root;
        state.advance(animation, dt, |at_birth| {
            sample_emitter_transform(node, root, at_birth, &nodes).unwrap_or(*transform)
        });
    }
    for (layer, mut instances) in &mut layers {
        let Ok(state) = emitters.get(layer.emitter) else {
            continue;
        };
        let Ok((animation, bindings)) = animations.get(state.root) else {
            continue;
        };
        let animation = sample_clock(animation.clone());
        let animation = &animation;
        let definition = &state.definition;
        let color = sample_value(&definition.color, animation);
        let alpha = sample_value(&definition.alpha, animation)
            * sample_value(&layer.definition.alpha, animation);
        let [high, low] = split_time(state.clock.time);
        let rows = definition.rows.max(1);
        let columns = definition.columns.max(1);
        let slot = sample_value(&definition.texture_slot, animation)
            .min(rows.saturating_mul(columns).saturating_sub(1));
        instances.records = state.live.records.clone();
        instances.live_indices = state.segments.clone();
        instances.uniform = RibbonUniform {
            color: [color[0], color[1], color[2], alpha.clamp(0.0, 1.0)],
            clock_gravity: [high, low, definition.gravity, 0.0],
            atlas_flags: [
                rows,
                columns,
                slot,
                u32::from(
                    layer.definition.shading_flags.unshaded()
                        || layer.definition.shading_flags.unlit(),
                ),
            ],
            uv_transform: uv_transform(layer.texture_animation.as_ref(), animation)
                .to_cols_array_2d(),
        };
        if let Some(bindings) = bindings {
            instances.texture =
                bindings.bitmap(sample_value(&layer.definition.texture_id, animation) as usize);
        }
    }
}

#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
