use super::*;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::profiles::Universal;
use wc3::model::scene::popcorn::{EffectLayerId, EffectProgram};
use wc3::model::{
    AnimationProgram, AnimationSample, Curve, DynamicTrack, ModelMetadata, Time, TimeSource, Track,
};

fn programme() -> EffectProgram {
    Document::read(include_bytes!(
        "../../../../wc3/tests/fixtures/pkb/controls.pkb"
    ))
    .unwrap()
    .compile_effect(1, &EffectCompileOptions::default())
    .unwrap()
}
fn model() -> (Model<'static>, NodeId, FeatureId) {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("external").unwrap();
    let mut emitter: Popcorn<'_> = builder.construct();
    emitter.visibility = builder.add_value(1.);
    emitter.life_span = builder.add_value(1.);
    emitter.emission_rate = builder.add_value(1.);
    emitter.speed = builder.add_value(1.);
    emitter.color = builder.add_value(Vec3::ONE);
    emitter.opacity = builder.add_value(1.);
    builder
        .add_global_track(DynamicTrack::Scalar(Track {
            target: emitter.emission_rate,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                [1., 3.],
            )
            .unwrap(),
        }))
        .unwrap();
    let feature = builder
        .add_feature(node, NodeFeature::ParticleProgram(Box::new(emitter)))
        .unwrap();
    (builder.finish().unwrap(), node, feature)
}
fn spawn(runtime: &RuntimeProgramParticles, world: &mut World) -> ProgramInstances {
    let basis = world.spawn_empty().id();
    let mut meshes = Assets::<Mesh>::default();
    let mut queue = CommandQueue::default();
    let result = runtime.spawn(&mut Commands::new(&mut queue, world), &mut meshes, basis);
    queue.apply(world);
    result
}
fn tick(
    runtime: &RuntimeProgramParticles,
    states: &mut ProgramInstances,
    world: &mut World,
    model: &Model<'_>,
    time: f64,
    change: EffectChange,
    seed: bool,
) {
    let animation = AnimationProgram::compile(model);
    let selection = AnimationSample {
        clip_time: Time::seconds(time).unwrap(),
        shared_time: Time::seconds(time).unwrap(),
        ..Default::default()
    };
    let sample = animation.sample(selection).unwrap();
    let poses = [Mat4::IDENTITY];
    let visible = [true];
    let frame = EffectFrame {
        pose: None,
        root_visible: true,
        traversal: Default::default(),
        change,
        time,
        selection,
        sample: &sample,
        node_poses: &poses,
        node_visible: &visible,
        basis_world: Mat4::IDENTITY,
    };
    let mut queries = SystemState::<(
        Query<&mut AgeSpriteData>,
        Query<&mut ParticleProgramEmitter>,
        Query<&ParticleProgramControls>,
    )>::new(world);
    let (mut sprites, mut statistics, controls) = queries.get_mut(world).unwrap();
    if seed {
        runtime.seed_owned(states, &EffectStart { frame }, Some(&controls));
    } else {
        runtime.update_bounded(
            states,
            &frame,
            &mut sprites,
            &mut statistics,
            Some(&controls),
            None,
        );
    }
}
fn size(states: &ProgramInstances) -> f32 {
    let core = states.states[0].instance.as_ref().unwrap();
    let particle = core.particles(EffectLayerId(0)).next().unwrap();
    f32::from_bits(particle.field(EffectFieldId(3)).unwrap().words()[0])
}
fn identity(states: &ProgramInstances) -> (u32, u32, u64) {
    let particle = states.states[0]
        .instance
        .as_ref()
        .unwrap()
        .particles(EffectLayerId(0))
        .next()
        .unwrap();
    (
        particle.id.slot,
        particle.id.generation,
        particle.birth_time.to_bits(),
    )
}

#[test]
fn shared_handles_preserve_independent_values_and_restore_current_baselines() {
    let program = programme();
    let (model, node, feature) = model();
    let effect = ParticleEffect::new(program.clone(), |_| Some(Handle::default()));
    let runtime = RuntimeProgramParticles::compile(
        &model,
        &HashMap::from([(node, 0)]),
        &HashMap::from([(feature, effect)]),
    );
    let mut world = World::new();
    let mut a = spawn(&runtime, &mut world);
    let mut b = spawn(&runtime, &mut world);
    let ae = a.states[0].entity;
    let be = b.states[0].entity;
    let rate = world
        .get::<ParticleProgramControls>(ae)
        .unwrap()
        .attribute("Game.EmissionRateMultiplier")
        .unwrap();
    let custom = world
        .get::<ParticleProgramControls>(ae)
        .unwrap()
        .attribute("Custom.SizeMultiplier")
        .unwrap();
    world
        .get_mut::<ParticleProgramControls>(ae)
        .unwrap()
        .set(&rate, 4f32)
        .unwrap();
    world
        .get_mut::<ParticleProgramControls>(be)
        .unwrap()
        .set(&rate, 6f32)
        .unwrap();
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        0.,
        EffectChange::Continuous,
        false,
    );
    tick(
        &runtime,
        &mut b,
        &mut world,
        &model,
        0.,
        EffectChange::Continuous,
        true,
    );
    assert_eq!((size(&a), size(&b)), (8., 12.));
    let first = identity(&a);
    let owned_first = identity(&b);
    world
        .get_mut::<ParticleProgramControls>(be)
        .unwrap()
        .set(&rate, 9f32)
        .unwrap();
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        0.25,
        EffectChange::Continuous,
        false,
    );
    tick(
        &runtime,
        &mut b,
        &mut world,
        &model,
        0.25,
        EffectChange::Continuous,
        false,
    );
    assert_eq!((size(&a), size(&b)), (8., 18.));
    assert_eq!(identity(&b), owned_first);
    world
        .get_mut::<ParticleProgramControls>(ae)
        .unwrap()
        .remove(&rate)
        .unwrap();
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        0.5,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(
        size(&a),
        4.,
        "removal reveals current authored rate2, not initial rate1"
    );
    assert_eq!(identity(&a), first);
    assert_eq!(size(&b), 18.);
    world
        .get_mut::<ParticleProgramControls>(ae)
        .unwrap()
        .set(&custom, 5f32)
        .unwrap();
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        0.75,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(size(&a), 12.5);
    world
        .get_mut::<ParticleProgramControls>(ae)
        .unwrap()
        .clear();
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        1.,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(size(&a), 6., "custom removal restores declared default2");

    // No revision cache: a same-programme replacement's different table applies immediately.
    let replacement = world.get::<ParticleProgramControls>(be).unwrap().clone();
    world.entity_mut(ae).insert(replacement);
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        1.1,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(size(&a), 18.);
    world.entity_mut(ae).remove::<ParticleProgramControls>();
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        1.2,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(
        size(&a),
        6.,
        "component removal cannot leave a stale live value"
    );
    // Reapply an active override before replacing it directly with foreign controls.
    let replacement = world.get::<ParticleProgramControls>(be).unwrap().clone();
    world.entity_mut(ae).insert(replacement);
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        1.25,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(size(&a), 18.);
    let mut foreign = ParticleProgramControls::new(&programme());
    let foreign_rate = foreign.attribute("Game.EmissionRateMultiplier").unwrap();
    foreign.set(&foreign_rate, 99f32).unwrap();
    world.entity_mut(ae).insert(foreign);
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        1.3,
        EffectChange::Continuous,
        false,
    );
    let statistics = world.get::<ParticleProgramEmitter>(ae).unwrap();
    assert_eq!(
        statistics.control_error,
        Some(ParticleProgramControlError::WrongProgram)
    );
    assert!(statistics.error.is_none());
    assert_eq!(size(&a), 6.);
    assert_eq!(identity(&a), first);
    let replacement = world.get::<ParticleProgramControls>(be).unwrap().clone();
    world.entity_mut(ae).insert(replacement);
    tick(
        &runtime,
        &mut a,
        &mut world,
        &model,
        1.4,
        EffectChange::Continuous,
        false,
    );
    let statistics = world.get::<ParticleProgramEmitter>(ae).unwrap();
    assert!(statistics.control_error.is_none());
    assert!(statistics.error.is_none());
    assert_eq!(size(&a), 18.);
    assert_eq!(identity(&a), first);

    let fixed = ParticleEffect::new(program, |_| Some(Handle::default()))
        .with_attribute("Game.EmissionRateMultiplier", 7f32)
        .unwrap();
    let fixed = RuntimeProgramParticles::compile(
        &model,
        &HashMap::from([(node, 0)]),
        &HashMap::from([(feature, fixed)]),
    );
    let mut c = spawn(&fixed, &mut world);
    let ce = c.states[0].entity;
    world
        .get_mut::<ParticleProgramControls>(ce)
        .unwrap()
        .set(&rate, 4f32)
        .unwrap();
    tick(
        &fixed,
        &mut c,
        &mut world,
        &model,
        0.,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(size(&c), 8.);
    world
        .get_mut::<ParticleProgramControls>(ce)
        .unwrap()
        .remove(&rate)
        .unwrap();
    tick(
        &fixed,
        &mut c,
        &mut world,
        &model,
        0.5,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(
        size(&c),
        14.,
        "prepared fixed value outranks current authored rate"
    );
}

#[test]
fn typed_controls_do_not_force_paused_invocations_or_clear_real_faults() {
    let (model, node, feature) = model();
    let runtime = RuntimeProgramParticles::compile(
        &model,
        &HashMap::from([(node, 0)]),
        &HashMap::from([(
            feature,
            ParticleEffect::new(programme(), |_| Some(Handle::default())),
        )]),
    );
    let mut world = World::new();
    let mut states = spawn(&runtime, &mut world);
    let entity = states.states[0].entity;
    let rate = world
        .get::<ParticleProgramControls>(entity)
        .unwrap()
        .attribute("Game.EmissionRateMultiplier")
        .unwrap();
    let custom = world
        .get::<ParticleProgramControls>(entity)
        .unwrap()
        .attribute("Custom.SizeMultiplier")
        .unwrap();
    tick(
        &runtime,
        &mut states,
        &mut world,
        &model,
        0.,
        EffectChange::Continuous,
        false,
    );
    let original = identity(&states);
    world
        .get_mut::<ParticleProgramControls>(entity)
        .unwrap()
        .set(&rate, 5f32)
        .unwrap();
    tick(
        &runtime,
        &mut states,
        &mut world,
        &model,
        0.,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(
        size(&states),
        2.,
        "attribute installation is not an evolution invocation"
    );
    tick(
        &runtime,
        &mut states,
        &mut world,
        &model,
        0.1,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(size(&states), 10.);
    assert_eq!(identity(&states), original);
    world
        .get_mut::<ParticleProgramControls>(entity)
        .unwrap()
        .set(&custom, f32::NAN)
        .unwrap();
    tick(
        &runtime,
        &mut states,
        &mut world,
        &model,
        0.2,
        EffectChange::Continuous,
        false,
    );
    let failure = states.states[0].error.clone().unwrap();
    assert_eq!(failure.layer, Some(EffectLayerId(0)));
    assert_eq!(failure.offset, Some(35));
    let fault_time = states.states[0].instance.as_ref().unwrap().time();
    world
        .get_mut::<ParticleProgramControls>(entity)
        .unwrap()
        .set(&custom, 3f32)
        .unwrap();
    tick(
        &runtime,
        &mut states,
        &mut world,
        &model,
        0.3,
        EffectChange::Continuous,
        false,
    );
    assert_eq!(states.states[0].error, Some(failure));
    assert_eq!(
        states.states[0].instance.as_ref().unwrap().time(),
        fault_time
    );
    tick(
        &runtime,
        &mut states,
        &mut world,
        &model,
        0.,
        EffectChange::Reset,
        false,
    );
    assert!(states.states[0].error.is_none());
    assert_eq!(size(&states), 15.);
    assert_ne!(identity(&states), original);
}

#[test]
fn handles_follow_programme_identity_and_setters_preserve_explicit_values() {
    let program = programme();
    assert!(program.same_program(&program.clone()));
    let other = programme();
    assert!(!program.same_program(&other));
    let mut a = ParticleProgramControls::new(&program);
    let mut b = ParticleProgramControls::new(&program);
    let foreign = ParticleProgramControls::new(&other);
    let rate = a.attribute("Game.EmissionRateMultiplier").unwrap();
    assert_eq!(rate.kind(), ValueType::Float);
    assert!(a.attribute("missing").is_err());
    assert_eq!(
        foreign.get(&rate),
        Err(ParticleProgramControlError::WrongProgram)
    );
    assert!(matches!(
        a.set(&rate, 1i32),
        Err(ParticleProgramControlError::TypeMismatch { .. })
    ));
    a.set(&rate, 4f32).unwrap();
    b.set(&rate, 9f32).unwrap();
    assert_eq!(a.get(&rate).unwrap(), Some(Value::from(4f32)));
    assert_eq!(b.get(&rate).unwrap(), Some(Value::from(9f32)));
    for _ in 0..10 {
        a.set(&rate, 4f32).unwrap();
    }
    assert_eq!(a.values().len(), 1);
    let attribute_debug = format!("{rate:?}");
    assert!(attribute_debug.starts_with("ParticleProgramAttribute { id:"));
    assert!(!attribute_debug.contains("program"));
    let controls_debug = format!("{a:?}");
    assert!(controls_debug.starts_with("ParticleProgramControls { attribute_count:"));
    assert!(controls_debug.contains("overrides"));
    assert!(!controls_debug.contains("program"));
    assert!(a.remove(&rate).unwrap());
    assert!(!a.remove(&rate).unwrap());
    a.clear();
    a.clear();
    assert_eq!(a.get(&rate).unwrap(), None);
    assert_eq!(
        foreign.get(&rate),
        Err(ParticleProgramControlError::WrongProgram)
    );
}
