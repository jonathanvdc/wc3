use super::*;
use crate::animation::pose::AnimatedNode;
use wc3::model::animation::Animatable;
use wc3::model::animation::{Sequence, Track, ValueKeyframe};
use wc3::model::scene::Node;

fn animation() -> Wc3Animation {
    Wc3Animation {
        sequence: 0,
        elapsed_ms: 0.0,
        speed: 1.0,
        playing: true,
        sequences: vec![Sequence::new("Stand", [0, 1000]).unwrap()],
        global_sequences: vec![],
    }
}

fn emitter() -> ParticleEmitter2 {
    let mut emitter = ParticleEmitter2::new(Node::new("Particles", 0).unwrap());
    emitter.life_span = 2.0;
    emitter.emission_rate = 10.0.into();
    emitter.frames = Particle2Frames::Both;
    emitter.time = 0.5;
    emitter.particle_scaling = [1.0, 2.0, 0.0];
    emitter.alpha = [255, 128, 0];
    emitter.segment_colors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    emitter.rows = 2;
    emitter.columns = 2;
    emitter
}

#[test]
fn emission_is_fractional_and_head_tail_are_independent() {
    let root = Entity::from_bits(1);
    let mut state = Particle2State::new(root, root, emitter());
    let animation = animation();
    state.advance(&animation, 0.15, |_| GlobalTransform::default());
    assert_eq!(state.live.metadata().len(), 2);
    state.advance(&animation, 0.15, |_| GlobalTransform::default());
    assert_eq!(state.live.metadata().len(), 6);
    assert_eq!(
        state
            .live
            .records
            .iter()
            .filter(|record| record.is_tail())
            .count(),
        3
    );
    assert_eq!(state.live.records.len(), 6);
}

#[test]
fn no_live_particles_produce_no_gpu_instances() {
    let root = Entity::from_bits(1);
    let state = Particle2State::new(root, root, emitter());
    assert!(state.live.records.is_empty());
}

#[test]
fn squirt_retriggers_when_sequence_loops() {
    let root = Entity::from_bits(1);
    let mut definition = emitter();
    definition.frames = Particle2Frames::Head;
    definition.set_squirt_enabled(true);
    definition.emission_rate = Animatable::Animated(
        Track::step(
            vec![
                ValueKeyframe {
                    frame: 0,
                    value: 2.0,
                },
                ValueKeyframe {
                    frame: 500,
                    value: 3.0,
                },
            ],
            None,
        )
        .unwrap(),
    );
    let mut state = Particle2State::new(root, root, definition);
    let mut animation = animation();
    state.advance(&animation, 0.01, |_| GlobalTransform::default());
    assert_eq!(state.live.metadata().len(), 2);
    animation.elapsed_ms = 600.0;
    state.advance(&animation, 0.01, |_| GlobalTransform::default());
    assert_eq!(state.live.metadata().len(), 5);
    animation.elapsed_ms = 1100.0;
    state.advance(&animation, 0.01, |_| GlobalTransform::default());
    assert_eq!(state.live.metadata().len(), 7);
}

#[test]
fn spawn_freezes_xyz_scale_for_heads_and_tails_in_both_spaces() {
    let root = Entity::from_bits(1);
    let scale = Vec3::new(2.0, 0.5, 3.0);
    let transform = GlobalTransform::from(Transform {
        translation: Vec3::new(5.0, 2.0, 1.0),
        rotation: Quat::from_rotation_y(0.7),
        scale,
    });
    for model_space in [false, true] {
        let mut definition = emitter();
        definition.node.flags.set_model_space(model_space);
        let mut state = Particle2State::new(root, root, definition);
        state.spawn(&animation(), &transform, 0.0);
        assert_eq!(state.live.records.len(), 2);
        for record in state.live.records.iter() {
            assert!(record.spawn_scale().abs_diff_eq(scale, 0.00001));
        }
        assert!(state.live.records[1].is_tail());
        let records = state.live.records.clone();
        let mut paused = animation();
        paused.playing = false;
        state.advance(&paused, 0.25, |_| GlobalTransform::default());
        assert_eq!(*state.live.records, *records);
    }
}

#[test]
fn particles_keep_spawn_records_and_append_after_retirement() {
    let root = Entity::from_bits(1);
    let mut definition = emitter();
    definition.frames = Particle2Frames::Head;
    definition.gravity = 10.0.into();
    let mut state = Particle2State::new(root, root, definition);
    let mut animation = animation();
    state.advance(&animation, 0.1, |_| GlobalTransform::default());
    let records = state.live.records.clone();
    animation.playing = false;
    state.advance(&animation, 1.0, |_| GlobalTransform::default());
    assert!(Arc::ptr_eq(&records, &state.live.records));
    assert_eq!(state.live.metadata().len(), 1);
    state.advance(&animation, 1.0, |_| GlobalTransform::default());
    assert!(state.live.metadata().is_empty());
    animation.playing = true;
    state.advance(&animation, 0.1, |_| GlobalTransform::default());
    assert_eq!(state.live.records.len(), 1);
    assert_eq!(state.live.records.index(0), 1);
    assert_ne!(
        records[0],
        state.live.records[state.live.records.index(0) as usize]
    );
}
#[test]
fn update_respects_pause_speed_and_keeps_clock_across_sequence_reset() {
    use std::time::Duration;
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, update_particles);
    let mut animation = animation();
    animation.speed = 2.0;
    let root = app
        .world_mut()
        .spawn((animation, GlobalTransform::default()))
        .id();
    let definition = emitter();
    let entity = app
        .world_mut()
        .spawn((
            Particle2State::new(root, root, definition.clone()),
            ParticleInstances {
                records: Arc::default(),
                live_indices: Arc::default(),
                uniform: ParticleEmitterUniform::default(),
                texture: None,
                filter: definition.filter_mode,
                priority_plane: 0,
                sort_far: false,
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(0.25));
    app.update();
    assert_eq!(
        app.world()
            .get::<Particle2State>(entity)
            .unwrap()
            .clock
            .time,
        0.5
    );
    let records = app
        .world()
        .get::<ParticleInstances>(entity)
        .unwrap()
        .records
        .clone();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .playing = false;
    app.update();
    assert_eq!(
        app.world()
            .get::<Particle2State>(entity)
            .unwrap()
            .clock
            .time,
        0.5
    );
    assert!(Arc::ptr_eq(
        &records,
        &app.world()
            .get::<ParticleInstances>(entity)
            .unwrap()
            .records
    ));
    let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
    animation.playing = true;
    animation.elapsed_ms = 0.0;
    animation.sequence = 1;
    app.update();
    let state = app.world().get::<Particle2State>(entity).unwrap();
    assert_eq!(state.clock.time, 1.0);
    assert_eq!(state.live.metadata()[0].birth_time, 0.1);
    assert_eq!(state.live.records[0], records[0]);
}

#[test]
fn head_tail_pair_never_exceeds_particle_limit() {
    let root = Entity::from_bits(1);
    let mut state = Particle2State::new(root, root, emitter());
    let animation = animation();
    let transform = GlobalTransform::default();
    for _ in 0..MAX_PARTICLES / 2 {
        state.spawn(&animation, &transform, state.clock.time);
    }
    let records = state.live.records.clone();
    state.spawn(&animation, &transform, state.clock.time);
    assert_eq!(state.live.metadata().len(), MAX_PARTICLES);
    assert_eq!(state.live.records.len(), MAX_PARTICLES);
    assert!(Arc::ptr_eq(&records, &state.live.records));
}
#[test]
fn continuous_births_and_spawn_parameters_do_not_depend_on_update_partition() {
    let root = Entity::from_bits(1);
    let mut definition = emitter();
    definition.frames = Particle2Frames::Head;
    definition.emission_rate = 20.0.into();
    definition.gravity = 1.0.into();
    definition.speed = Animatable::Animated(
        Track::linear(
            vec![
                ValueKeyframe {
                    frame: 0,
                    value: 0.0,
                },
                ValueKeyframe {
                    frame: 2000,
                    value: 20.0,
                },
            ],
            None,
        )
        .unwrap(),
    );
    let run = |steps: &[f64]| {
        let mut state = Particle2State::new(root, root, definition.clone());
        let mut animation = animation();
        animation.sequences = vec![Sequence::new("Stand", [0, 2000]).unwrap()];
        for &dt in steps {
            animation.elapsed_ms += dt * 1000.0;
            state.advance(&animation, dt, |birth| {
                GlobalTransform::from_translation(Vec3::X * (birth.elapsed_ms * 0.001) as f32)
            });
        }
        state
    };
    let whole = run(&[1.0]);
    let split = run(&[0.3, 0.2, 0.5]);
    assert_eq!(whole.live.metadata().len(), 20);
    assert_eq!(split.live.metadata().len(), 20);
    let uniform = ParticleEmitterUniform::new(&definition, &GlobalTransform::default(), 1.0);
    for (i, (a, b)) in whole
        .live
        .metadata()
        .iter()
        .zip(split.live.metadata())
        .enumerate()
    {
        let expected_birth = (i + 1) as f64 / 20.0;
        assert!((a.birth_time - expected_birth).abs() < 1e-12);
        assert!((a.birth_time - b.birth_time).abs() < 1e-12);
        let age = (1.0 - expected_birth) as f32;
        let expected = Vec3::new(
            expected_birth as f32,
            0.0,
            expected_birth as f32 * 10.0 * age - 0.5 * age * age,
        );
        let center_a = whole.live.records[whole.live.records.index(i) as usize].center(&uniform);
        let center_b = split.live.records[split.live.records.index(i) as usize].center(&uniform);
        assert!(
            center_a.abs_diff_eq(expected, 1e-6),
            "{center_a:?} != {expected:?}"
        );
        assert!(center_a.abs_diff_eq(center_b, 1e-6));
    }
}

#[test]
fn birth_transform_samples_the_animated_parent_chain() {
    use std::time::Duration;
    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(400));
    app.insert_resource(time);
    app.add_systems(Update, update_particles);
    let mut animation = animation();
    animation.elapsed_ms = 1000.0;
    animation.sequences = vec![Sequence::new("Stand", [0, 2000]).unwrap()];
    let root = app
        .world_mut()
        .spawn((
            animation,
            GlobalTransform::from_translation(Vec3::X * 100.0),
        ))
        .id();
    let parent = app
        .world_mut()
        .spawn((
            AnimatedNode {
                flags: Default::default(),
                camera: None,
                root,
                pivot: Vec3::ZERO,
                parent_pivot: Vec3::ZERO,
                translation: Some(
                    Track::linear(
                        vec![
                            ValueKeyframe {
                                frame: 0,
                                value: [0.0; 3],
                            },
                            ValueKeyframe {
                                frame: 2000,
                                value: [20.0, 0.0, 0.0],
                            },
                        ],
                        None,
                    )
                    .unwrap(),
                ),
                rotation: None,
                scaling: None,
            },
            GlobalTransform::from_translation(Vec3::X * 110.0),
            ChildOf(root),
        ))
        .id();
    let node = app
        .world_mut()
        .spawn((
            AnimatedNode {
                flags: Default::default(),
                camera: None,
                root,
                pivot: Vec3::X * 2.0,
                parent_pivot: Vec3::ZERO,
                translation: None,
                rotation: None,
                scaling: None,
            },
            GlobalTransform::from_translation(Vec3::X * 112.0),
            ChildOf(parent),
        ))
        .id();
    let mut definition = emitter();
    definition.frames = Particle2Frames::Head;
    let entity = app
        .world_mut()
        .spawn((
            Particle2State::new(root, node, definition.clone()),
            ParticleInstances {
                records: Arc::default(),
                live_indices: Arc::default(),
                uniform: ParticleEmitterUniform::default(),
                texture: None,
                filter: definition.filter_mode,
                priority_plane: 0,
                sort_far: false,
            },
        ))
        .id();
    app.update();
    let data = app.world().get::<ParticleInstances>(entity).unwrap();
    assert_eq!(data.live_indices.len(), 4);
    for (i, &slot) in data.live_indices.iter().enumerate() {
        assert_eq!(
            data.records[slot as usize].center(&data.uniform).x,
            109.0 + i as f32
        );
    }
}

#[test]
fn large_updates_retire_particles_between_births() {
    let root = Entity::from_bits(1);
    let mut definition = emitter();
    definition.frames = Particle2Frames::Head;
    definition.life_span = 0.05;
    let mut state = Particle2State::new(root, root, definition);
    let mut animation = animation();
    animation.elapsed_ms = 1000.0;
    state.advance(&animation, 1.0, |_| GlobalTransform::default());
    assert_eq!(state.live.metadata().len(), 1);
    assert_eq!(state.live.metadata()[0].birth_time, 1.0);
    assert_eq!(state.live.records.len(), 1);
}
