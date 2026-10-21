use super::*;

fn state() -> State {
    State {
        entity: Entity::PLACEHOLDER,
        children: Vec::new(),
        phase: Default::default(),
        ordinal: 0,
        scratch: Vec::new(),
        previous_node: Mat4::IDENTITY,
        omitted: 0,
        skipped_seconds: 0.,
    }
}
fn packet(ordinal: u64) -> [f32; 3] {
    let mut rng = EmitterRng::at_occurrence(17, ordinal);
    [rng.random(), rng.signed(), rng.random()]
}
#[test]
fn budgets_and_invalid_occurrences_do_not_shift_later_model_packets() {
    let mut whole = state();
    let mut limited = state();
    let (_, whole_first) = whole.schedule(20., 0.2, 1024);
    let (_, limited_first) = limited.schedule(20., 0.2, 1);
    assert_eq!(packet(whole_first), packet(limited_first));
    assert_eq!((limited.ordinal, limited.omitted), (4, 3));
    let (_, whole_next) = whole.schedule(20., 0.1, 1024);
    let (_, limited_next) = limited.schedule(20., 0.1, 1024);
    assert_eq!(whole_next, 4);
    assert_eq!(packet(whole_next), packet(limited_next));
    // The production loop validates pose/lifetime/resource only after this
    // reservation. Discarding all candidates must still consume their identities.
    let _invalid = limited.schedule(20., 0.1, 1024);
    let (_, following) = limited.schedule(20., 0.05, 1024);
    assert_eq!(following, 8);
}
#[test]
fn split_updates_and_reset_preserve_source_occurrence_identity() {
    let mut whole = state();
    let (all, first) = whole.schedule(30., 0.1, 1024);
    let expected: Vec<_> = (0..all.count as u64).map(|i| packet(first + i)).collect();
    let mut split = state();
    let mut observed = Vec::new();
    for dt in [0.03, 0.04, 0.03] {
        let (schedule, first) = split.schedule(30., dt, 1024);
        observed.extend((0..schedule.count as u64).map(|i| packet(first + i)));
    }
    assert_eq!(observed, expected);
    split.reset_emission();
    let (_, first) = split.schedule(30., 0.1, 1024);
    assert_eq!(first, 0);
    assert_eq!(packet(first), expected[0]);
}
#[test]
fn large_omitted_schedules_advance_once_and_budget_totals_saturate() {
    let mut state = state();
    let (schedule, first) = state.schedule(1e12, 1., 0);
    assert_eq!((schedule.count, first), (0., 0));
    assert_eq!(
        (state.ordinal, state.omitted),
        (1_000_000_000_000, 1_000_000_000_000)
    );
    state.omitted = u64::MAX - 1;
    let (_, first) = state.schedule(4., 1., 1);
    assert_eq!(first, 1_000_000_000_000);
    assert_eq!(state.omitted, u64::MAX);
}

/// Exercise native import, occurrence sampling, and the actual model-child
/// motion bundle rather than mirroring the directional formula in a unit helper.
fn longitude_births(longitude: &str, latitude: f32) -> Vec<(f64, Vec3, Quat)> {
    use crate::compile_model;
    use bevy::ecs::world::CommandQueue;
    use wc3::model::{
        conversion::ImportOptions, io::mdlx, profiles::Universal, AnimationProgram,
        AnimationSample, ModelMetadata, Time,
    };
    let source = format!(
        r#"
        Version {{ FormatVersion 800, }}
        Model "Angular control" {{}}
        GlobalSequences 1 {{ Duration 1000, }}
        ParticleEmitter "Source" {{
            ObjectId 0, EmitterUsesMdl, Path "child.mdl",
            static EmissionRate 64, static Gravity 0,
            static Latitude {latitude}, {longitude}
            static LifeSpan 10, static InitVelocity 4,
        }}
        PivotPoints 1 {{ {{ 0,0,0 }}, }}
    "#
    );
    let model = mdlx::decode_mdl(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let child = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
    let mut assets = Assets::<ModelAsset>::default();
    let child = assets.add(ModelAsset {
        compiled: compile_model(&child, |_| None).unwrap(),
        images: HashMap::new(),
        unresolved_images: Vec::new(),
    });
    let feature = model.features().iter().next().unwrap().0;
    let indices = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let runtime = RuntimeModelParticles::compile(
        &model,
        &indices,
        &[(
            ChildModelKey::particle_geometry(feature),
            ChildModelBinding::new(child),
        )]
        .into(),
    );
    assert!(!runtime
        .diagnostics()
        .iter()
        .any(|(_, issue)| issue.kind == RenderDiagnosticKind::Unsupported
            && issue.message.contains("longitude")));
    let mut world = World::new();
    let root = world.spawn_empty().id();
    let mut queue = CommandQueue::default();
    let mut instances = runtime.spawn(&mut Commands::new(&mut queue, &world), root);
    queue.apply(&mut world);
    instances.previous_time = Some(0.);
    let selection = AnimationSample {
        clip_time: Time::seconds(0.75).unwrap(),
        shared_time: Time::seconds(0.75).unwrap(),
        ..Default::default()
    };
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(selection).unwrap();
    let mut statistics = World::new();
    let mut query = statistics.query::<&mut ModelEmitter>();
    runtime.update(
        &mut instances,
        &mut Commands::new(&mut queue, &world),
        root,
        &EffectFrame {
            pose: None,
            root_visible: true,
            traversal: Default::default(),
            change: EffectChange::Continuous,
            time: 0.75,
            selection,
            sample: &sample,
            node_poses: &[Mat4::IDENTITY],
            node_visible: &[true],
            basis_world: Mat4::IDENTITY,
        },
        &assets,
        &mut query.query_mut(&mut statistics),
    );
    queue.apply(&mut world);
    let children = &instances.states[0].children;
    assert_eq!(children.len(), 48);
    children
        .iter()
        .map(|(entity, birth, _)| {
            let motion = world.get::<ModelParticleMotion>(*entity).unwrap();
            (*birth, motion.velocity, motion.rotation)
        })
        .collect()
}

#[test]
fn longitude_zero_narrow_and_full_control_polar_support_without_changing_spin_packets() {
    use std::f32::consts::FRAC_PI_8;
    let zero = longitude_births("static Longitude 0,", 1.);
    let narrow = longitude_births("static Longitude 0.3926990817,", 1.);
    let full = longitude_births("static Longitude 3.1415926536,", 1.);
    for ((zero, narrow), full) in zero.iter().zip(&narrow).zip(&full) {
        assert_eq!(
            zero.1.y, 0.,
            "zero longitude must preserve the emitter polar plane"
        );
        assert!(narrow.1.y.abs() <= narrow.1.x.abs() * FRAC_PI_8.tan() + 1e-5);
        assert!((full.1.length() - 4.).abs() < 1e-5);
        assert!(
            full.1.z >= 4. * 1f32.cos() - 1e-5,
            "latitude is one radian, not degrees"
        );
        assert_eq!(
            zero.2, narrow.2,
            "longitude does not consume another random draw"
        );
        assert_eq!(zero.2, full.2);
    }
    assert!(full
        .iter()
        .any(|(_, velocity, _)| velocity.y.abs() > velocity.x.abs()));
    assert!(
        full.iter()
            .any(|(_, velocity, _)| velocity.truncate().length() > 2.),
        "raw-radian latitude must retain its broad spread"
    );
}
#[test]
fn longitude_is_sampled_at_each_birth_not_at_the_update_endpoint() {
    let births = longitude_births(
        "Longitude 2 { DontInterp, GlobalSeqId 0, 0: 0, 500: 1.5707963268, }",
        1.,
    );
    assert!(births
        .iter()
        .filter(|(time, _, _)| *time < 0.49)
        .all(|(_, velocity, _)| velocity.y == 0.));
    assert!(births
        .iter()
        .filter(|(time, _, _)| *time > 0.51)
        .any(|(_, velocity, _)| velocity.y.abs() > 0.01));
}
#[test]
fn zero_latitude_keeps_positive_z_for_every_longitude() {
    for longitude in [
        "static Longitude 0,",
        "static Longitude 0.3926990817,",
        "static Longitude 3.1415926536,",
    ] {
        for (_, velocity, _) in longitude_births(longitude, 0.) {
            assert!((velocity - Vec3::Z * 4.).length() < 1e-5);
        }
    }
}
