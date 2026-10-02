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
