use super::*;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::profiles::Universal;
use wc3::model::scene::popcorn::{EffectLayerId, EffectLimits, EffectProgram};
use wc3::model::{AnimationProgram, AnimationSample, ModelMetadata};

// Reuse the checked-in original native curve fixture, changing only its scalar
// sample coordinate to a nonfinite constant. This exercises a real checked
// callback failure, not an error fabricated at the Bevy observation boundary.
fn failing_curve_program() -> EffectProgram {
    let mut bytes = include_bytes!("../../../../wc3/tests/fixtures/pkb/sparse_curve.pkb").to_vec();
    let constant_offset = {
        let document = Document::read(&bytes).unwrap();
        let blob = document.compiler_blobs().next().unwrap().unwrap();
        let script = blob.script().unwrap();
        assert!(!script.constants.is_empty());
        script.constants.as_ptr() as usize - bytes.as_ptr() as usize
    };
    bytes[constant_offset..constant_offset + 4].copy_from_slice(&f32::NAN.to_bits().to_le_bytes());
    Document::read(&bytes)
        .unwrap()
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap()
}

#[test]
fn core_error_context_survives_adapter_observation_and_reset_retries() {
    let program = failing_curve_program();
    let mut direct = program.instantiate(EffectLimits::default(), 0).unwrap();
    let expected = direct
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap_err();
    assert_eq!(expected.layer, Some(EffectLayerId(0)));
    assert_eq!(expected.offset, Some(0));
    assert_eq!(expected.message, "curve sample coordinate must be finite");

    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("external").unwrap();
    let mut emitter: Popcorn<'_> = builder.construct();
    emitter.visibility = builder.add_value(1.);
    emitter.life_span = builder.add_value(1.);
    emitter.emission_rate = builder.add_value(1.);
    emitter.speed = builder.add_value(1.);
    emitter.color = builder.add_value(Vec3::ONE);
    emitter.opacity = builder.add_value(1.);
    let feature = builder
        .add_feature(node, NodeFeature::ParticleProgram(Box::new(emitter)))
        .unwrap();
    let model = builder.finish().unwrap();
    let effect = ParticleEffect::new(program, |_| Some(Handle::default()));
    let definitions = RuntimeProgramParticles::compile(
        &model,
        &HashMap::from([(node, 0)]),
        &HashMap::from([(feature, effect)]),
    );
    let mut world = World::new();
    let basis = world.spawn_empty().id();
    let mut meshes = Assets::<Mesh>::default();
    let mut queue = CommandQueue::default();
    let mut instances = {
        let mut commands = Commands::new(&mut queue, &world);
        definitions.spawn(&mut commands, &mut meshes, basis)
    };
    queue.apply(&mut world);
    let animation = AnimationProgram::compile(&model);
    let sample = animation.sample(AnimationSample::default()).unwrap();
    let poses = [Mat4::IDENTITY];
    let visibility = [true];
    let mut queries = SystemState::<(
        Query<&mut AgeSpriteData>,
        Query<&mut ParticleProgramEmitter>,
    )>::new(&mut world);
    let mut first_particle = None;
    for (index, (time, change)) in [
        (0., EffectChange::Continuous),
        (0., EffectChange::Continuous),
        (0.25, EffectChange::Continuous),
        (0., EffectChange::Reset),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut sprites, mut statistics) = queries.get_mut(&mut world).unwrap();
        definitions.update(
            &mut instances,
            &EffectFrame {
                pose: None,
                root_visible: true,
                traversal: Default::default(),
                change,
                time,
                selection: Default::default(),
                sample: &sample,
                node_poses: &poses,
                node_visible: &visibility,
                basis_world: Mat4::IDENTITY,
            },
            &mut sprites,
            &mut statistics,
        );
        let output = statistics.single().unwrap();
        let actual = output.error.as_ref().unwrap();
        assert_eq!(actual, &expected);
        assert_eq!(actual.to_string(), expected.to_string());
        assert_eq!(output.feature, feature);
        assert_eq!(
            output.sprites, 0,
            "faulted programmes do not publish geometry"
        );
        let core = instances.states[0].instance.as_ref().unwrap();
        assert!(!core.is_running());
        assert_eq!(
            core.time(),
            0.,
            "a retained fault must not continue advancing"
        );
        let particle = core.particles(EffectLayerId(0)).next().unwrap().id;
        if let Some(first) = first_particle {
            if index == 3 {
                assert_ne!(particle, first, "reset retries the failed initializer");
            } else {
                assert_eq!(
                    particle, first,
                    "unchanged observations do not retry execution"
                );
            }
        } else {
            first_particle = Some(particle);
        }
    }
}
