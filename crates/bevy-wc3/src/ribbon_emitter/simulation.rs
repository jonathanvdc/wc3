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
    nodes: Query<(&GlobalTransform, Option<&AnimatedNode>, Option<&ChildOf>)>,
    mut emitters: Query<&mut RibbonState>,
    mut layers: Query<(&RibbonLayer, &mut RibbonInstances)>,
) {
    for mut state in &mut emitters {
        let Ok((animation, _)) = animations.get(state.root) else {
            continue;
        };
        let Ok((transform, _, _)) = nodes.get(state.node) else {
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
mod tests {
    use super::*;
    use wc3::model::animation::{Animatable, Sequence, Track, ValueKeyframe};
    use wc3::model::scene::Node;

    fn animation() -> Wc3Animation {
        Wc3Animation {
            sequence: 0,
            elapsed_ms: 0.0,
            speed: 1.0,
            playing: true,
            sequences: vec![Sequence::new("Stand", [0, 4000]).unwrap()],
            global_sequences: vec![],
        }
    }

    fn state() -> RibbonState {
        let mut world = World::new();
        let root = world.spawn_empty().id();
        let mut definition = RibbonEmitter::new(Node::new("Trail", 0).unwrap());
        definition.height_above = Animatable::Static(2.0);
        definition.height_below = Animatable::Static(1.0);
        definition.life_span = 1.0;
        definition.emission_rate = 10;
        RibbonState::new(root, root, definition)
    }

    fn advance(state: &mut RibbonState, animation: &mut Wc3Animation, dt: f64) {
        animation.elapsed_ms += dt * 1000.0;
        state.advance(animation, dt, |birth| {
            GlobalTransform::from_translation(Vec3::X * birth.elapsed_ms as f32 * 0.001)
        });
    }

    #[test]
    fn subframe_births_match_across_frame_rates_and_preserve_world_space_history() {
        let mut coarse = state();
        let mut fine = state();
        let mut a = animation();
        let mut b = animation();
        advance(&mut coarse, &mut a, 0.5);
        for _ in 0..5 {
            advance(&mut fine, &mut b, 0.1);
        }
        assert_eq!(coarse.live.metadata().len(), 5);
        assert_eq!(coarse.segments.len(), 4);
        for (x, y) in coarse.live.records.iter().zip(fine.live.records.iter()) {
            for (x, y) in x.above_birth.iter().zip(y.above_birth.iter()) {
                assert!((x - y).abs() < 1e-6);
            }
            for (x, y) in x.below_birth.iter().zip(y.below_birth.iter()) {
                assert!((x - y).abs() < 1e-6);
            }
        }
        assert_eq!(coarse.live.records[0].above_birth[..3], [0.1, 2.0, 0.0]);
        assert_eq!(coarse.live.records[0].below_birth[..3], [0.1, -1.0, 0.0]);
        assert_eq!(coarse.segments[0], RibbonSegment([0, 1, 0, 4]));
        let first = coarse.live.records[0];
        advance(&mut coarse, &mut a, 0.2);
        assert_eq!(coarse.live.records[0], first);
    }

    #[test]
    fn exact_visibility_key_boundaries_match_at_30_and_60_fps() {
        let run = |fps: usize| {
            let mut state = state();
            state.definition.emission_rate = 20;
            state.definition.visibility = Some(
                Track::step(
                    vec![
                        ValueKeyframe {
                            frame: 0,
                            value: 1.0,
                        },
                        ValueKeyframe {
                            frame: 700,
                            value: 0.0,
                        },
                        ValueKeyframe {
                            frame: 1100,
                            value: 1.0,
                        },
                    ],
                    None,
                )
                .unwrap(),
            );
            let mut animation = animation();
            for _ in 0..fps * 2 {
                advance(&mut state, &mut animation, 1.0 / fps as f64);
            }
            state
        };
        let a = run(30);
        let b = run(60);
        assert_eq!(a.live.metadata().len(), b.live.metadata().len());
        assert_eq!(a.segments.len(), b.segments.len());
        for (a, b) in a.live.metadata().iter().zip(b.live.metadata().iter()) {
            assert!((a.birth - b.birth).abs() < 1e-9);
            assert_eq!(a.chain, b.chain);
        }
        for (a, b) in a.segments.iter().zip(b.segments.iter()) {
            assert_eq!(a.0[2..], b.0[2..]);
        }
    }

    #[test]
    fn gaps_do_not_bridge_and_each_live_chain_gets_its_own_uv_span() {
        let mut state = state();
        state.definition.visibility = Some(
            Track::step(
                vec![
                    ValueKeyframe {
                        frame: 0,
                        value: 1.0,
                    },
                    ValueKeyframe {
                        frame: 250,
                        value: 0.0,
                    },
                    ValueKeyframe {
                        frame: 450,
                        value: 1.0,
                    },
                ],
                None,
            )
            .unwrap(),
        );
        let mut animation = animation();
        advance(&mut state, &mut animation, 0.6);
        assert_eq!(state.live.metadata().len(), 4);
        assert_eq!(
            *state.segments,
            vec![RibbonSegment([0, 1, 0, 1]), RibbonSegment([2, 3, 0, 1])]
        );
        advance(&mut state, &mut animation, 1.0);
        for segment in state.segments.iter() {
            assert!((segment.0[0] as usize) < state.live.records.capacity());
            assert!((segment.0[1] as usize) < state.live.records.capacity());
        }
        assert!(state
            .live
            .metadata()
            .iter()
            .all(|section| state.clock.time - section.birth < 1.0));
    }

    #[test]
    fn pause_warmup_and_sequence_changes_keep_clock_and_break_connectivity() {
        let mut state = state();
        let mut animation = animation();
        advance(&mut state, &mut animation, 0.2);
        let records = state.live.records.clone();
        let segments = state.segments.clone();
        advance(&mut state, &mut animation, 0.0);
        assert!(Arc::ptr_eq(&records, &state.live.records));
        assert!(Arc::ptr_eq(&segments, &state.segments));
        animation.elapsed_ms = 0.0;
        advance(&mut state, &mut animation, 0.1);
        assert_eq!(state.live.metadata().len(), 3);
        assert_eq!(state.segments.len(), 1);
        advance(&mut state, &mut animation, 0.1);
        assert_eq!(state.segments.len(), 2);
        state.definition.emission_rate = 0;
        advance(&mut state, &mut animation, 1.0);
        assert!(state.live.metadata().is_empty());
        assert!(state.segments.is_empty());
    }

    #[test]
    fn large_steps_expire_old_births_and_extreme_rates_are_bounded() {
        let mut state = state();
        let mut animation = animation();
        advance(&mut state, &mut animation, 100.0);
        assert!(state.live.metadata().len() <= 10);
        assert!(state
            .live
            .metadata()
            .iter()
            .all(|section| state.clock.time - section.birth < 1.0));
        state.definition.emission_rate = u32::MAX;
        advance(&mut state, &mut animation, 1.0);
        assert_eq!(state.live.metadata().len(), MAX_SECTIONS);
        assert!(state.live.records.len() <= MAX_SECTIONS);
        assert_eq!(state.segments.len(), MAX_SECTIONS - 1);
    }

    #[test]
    fn wrapped_and_growing_sections_keep_chain_endpoints_and_birth_history() {
        let mut state = state();
        let mut animation = animation();
        for _ in 0..20 {
            advance(&mut state, &mut animation, 0.1);
        }
        assert_eq!(state.live.records.capacity(), 16);
        assert!(state
            .segments
            .iter()
            .any(|segment| segment.0[0] == 15 && segment.0[1] == 0));
        let before = state.live.records.clone();
        state.definition.emission_rate = 100;
        advance(&mut state, &mut animation, 0.2);
        assert_eq!(state.live.records.capacity(), 32);
        for (i, section) in state.live.metadata().iter().enumerate() {
            let record = state.live.records[state.live.records.index(i) as usize];
            assert!((record.above_birth[0] - section.birth as f32).abs() < 1e-5);
        }
        for (i, segment) in state.segments.iter().enumerate() {
            assert_eq!(segment.0[0], state.live.records.index(i));
            assert_eq!(segment.0[1], state.live.records.index(i + 1));
            assert_eq!(segment.0[2], i as u32);
            assert_eq!(segment.0[3], state.segments.len() as u32);
        }
        assert_eq!(before.capacity(), 16);
        assert_eq!(before.len(), 10);
    }

    #[test]
    fn non_looping_sequence_stops_births_while_existing_sections_expire() {
        let mut state = state();
        let mut animation = animation();
        animation.sequences[0].interval = [0, 250];
        animation.sequences[0].flags.set_non_looping(true);
        advance(&mut state, &mut animation, 0.5);
        assert_eq!(state.live.metadata().len(), 2);
        assert_eq!(state.segments.len(), 1);
        advance(&mut state, &mut animation, 1.0);
        assert!(state.live.metadata().is_empty());
        assert!(state.segments.is_empty());
    }

    #[test]
    fn texture_animation_rotates_and_scales_around_cell_surface_center() {
        let animation = animation();
        let definition = TextureAnimation {
            translation: Some(Track::constant([0.25, 0.0, 0.0])),
            rotation: Some(Track::constant(
                Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
            )),
            scaling: Some(Track::constant([2.0, 1.0, 1.0])),
        };
        let transform = uv_transform(Some(&definition), &animation);
        let center = transform.transform_point3(Vec3::new(0.5, 0.5, 0.0));
        assert!((center - Vec3::new(0.75, 0.5, 0.0)).length() < 1e-5);
        let right = transform.transform_point3(Vec3::new(1.0, 0.5, 0.0));
        assert!((right - Vec3::new(0.75, 1.5, 0.0)).length() < 1e-5);
    }

    #[test]
    fn rotated_scaled_sections_use_local_y_and_animated_heights_at_birth() {
        let mut state = state();
        state.definition.height_above.set_track(
            Track::linear(
                vec![
                    ValueKeyframe {
                        frame: 0,
                        value: 0.0,
                    },
                    ValueKeyframe {
                        frame: 1000,
                        value: 10.0,
                    },
                ],
                None,
            )
            .unwrap(),
        );
        let mut animation = animation();
        animation.elapsed_ms = 200.0;
        state.advance(&animation, 0.2, |_| {
            GlobalTransform::from(
                Transform::from_xyz(10.0, 20.0, 30.0)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(2.0)),
            )
        });
        assert!((state.live.records[0].above_birth[0] - 8.0).abs() < 1e-5);
        assert!((state.live.records[1].above_birth[0] - 6.0).abs() < 1e-5);
        assert!((state.live.records[0].below_birth[0] - 12.0).abs() < 1e-5);
        assert_eq!(state.live.records[0].above_birth[2], 30.0);
    }
}
