use super::prepared::ParticleEffect;
use super::runtime::{ParticleProgramEmitter, RuntimeProgramParticles};
use crate::effects::age_sprites_gpu::AgeSpriteData;
use crate::effects::emission::EffectChange;
use crate::effects::EffectFrame;
use crate::{RenderDiagnosticKind, TextureColorSpace, TextureRole};
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::profiles::{mdlx::Popcorn, Universal};
use wc3::model::{
    AnimationProgram, AnimationSample, ImageResource, Model, ModelMetadata, NodeFeature,
};

const SPRITE: &[u8] = include_bytes!("../../../../wc3/tests/fixtures/pkb/sprite.pkb");
fn effect() -> ParticleEffect {
    let document = Document::read(SPRITE).unwrap();
    let program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    ParticleEffect::new(program, |request| {
        assert_eq!(request.renderer, 0);
        assert_eq!(request.role, TextureRole::Color);
        assert_eq!(request.color_space, TextureColorSpace::Srgb);
        assert_eq!(
            request.resource,
            ImageResource::Path(b"sprite.png".as_slice().into())
        );
        Some(Handle::default())
    })
}
#[test]
fn universal_external_effect_lifecycle_preserves_paused_records_and_retires_geometry() {
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
    let effect = effect();
    assert!(
        effect.diagnostics().is_empty(),
        "{:?}",
        effect.diagnostics()
    );
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
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let poses = [Mat4::IDENTITY];
    let visibility = [true];
    let mut queries = SystemState::<(
        Query<&mut AgeSpriteData>,
        Query<&mut ParticleProgramEmitter>,
    )>::new(&mut world);
    let mut retained = None;
    for (step, (time, root_visible, change, expected)) in [
        (0., false, EffectChange::Continuous, 0),
        (0., false, EffectChange::Continuous, 0),
        (0., true, EffectChange::Continuous, 1),
        (0.25, true, EffectChange::Continuous, 1),
        (0.25, true, EffectChange::Continuous, 1),
        (0.25, true, EffectChange::Continuous, 1),
        (2.25, true, EffectChange::Continuous, 0),
        (0., true, EffectChange::Reset, 1),
        (0.25, true, EffectChange::Continuous, 1),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut sprites, mut statistics) = queries.get_mut(&mut world).unwrap();
        definitions.update(
            &mut instances,
            &EffectFrame {
                pose: None,
                root_visible,
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
        assert_eq!(output.sprites, expected, "time {time}");
        assert_eq!(output.omitted, 0);
        assert!(output.error.is_none(), "{:?}", output.error);
        let data = sprites.single().unwrap();
        assert_eq!(data.records.len(), expected);
        assert_eq!(data.live_indices.len(), expected);
        if time == 0.25 && retained.is_none() {
            assert_eq!(data.records[0].position, [0., 0., 0., 1.]);
            assert_eq!(data.records[0].size_spin, [400., 400., 1., 0.]);
            retained = Some(data.records.clone());
        } else if step == 4 || step == 5 {
            assert!(
                Arc::ptr_eq(retained.as_ref().unwrap(), &data.records),
                "paused extraction must retain its immutable source"
            );
        }
    }
}

#[test]
fn unknown_material_families_remain_explicit_without_loading_their_images() {
    let mut bytes = SPRITE.to_vec();
    let name = b"Default_Billboard.pkma";
    let position = bytes
        .windows(name.len())
        .position(|part| part == name)
        .unwrap();
    bytes[position..position + name.len()].copy_from_slice(b"Unknown_Billboard.pkma");
    let document = Document::read(&bytes).unwrap();
    let program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    assert_eq!(ParticleEffect::texture_requests(&program).count(), 0);
    let prepared = ParticleEffect::new(program, |_| {
        panic!("unsupported material requested an image")
    });
    assert!(prepared
        .diagnostics()
        .iter()
        .any(|issue| issue.kind == RenderDiagnosticKind::Unsupported));
}
#[test]
fn unknown_external_attributes_fail_during_preparation() {
    assert!(effect().with_attribute("Unknown", 1.0f32).is_err());
}

#[test]
#[ignore = "requires the separately acquired HD LichAttack dependency"]
fn explicit_external_attributes_override_authored_model_controls() {
    use std::path::Path;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dependencies/files/war3.w3mod/_hd.w3mod/sharedfx/lich/lichattack--f9ecf5d6868f.pkb");
    let bytes = std::fs::read(path).unwrap();
    let program = Document::read(&bytes)
        .unwrap()
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    let prepared = ParticleEffect::new(program, |_| Some(Handle::default()));
    assert!(prepared
        .clone()
        .with_attribute("Game.EmissionRateMultiplier", Vec4::ONE)
        .is_err());
    assert!(prepared
        .clone()
        .with_attribute("Game.TeamColor", Vec4::ONE)
        .is_ok());
    let baseline = live_programme_sprites(prepared.clone());
    assert!(
        baseline > 0,
        "the unmodified effect must generate visible particles"
    );
    let overridden = prepared
        .with_attribute("Game.EmissionRateMultiplier", 0.0f32)
        .unwrap();
    assert_eq!(
        live_programme_sprites(overridden),
        0,
        "an explicit zero multiplier must survive authored rate rebinding"
    );
}

fn live_programme_sprites(effect: ParticleEffect) -> usize {
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
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let poses = [Mat4::IDENTITY];
    let visibility = [true];
    let mut queries = SystemState::<(
        Query<&mut AgeSpriteData>,
        Query<&mut ParticleProgramEmitter>,
    )>::new(&mut world);
    let mut total = 0;
    for step in 0..=60 {
        let (mut sprites, mut statistics) = queries.get_mut(&mut world).unwrap();
        definitions.update(
            &mut instances,
            &EffectFrame {
                pose: None,
                root_visible: true,
                traversal: Default::default(),
                change: EffectChange::Continuous,
                time: f64::from(step) / 60.,
                selection: Default::default(),
                sample: &sample,
                node_poses: &poses,
                node_visible: &visibility,
                basis_world: Mat4::IDENTITY,
            },
            &mut sprites,
            &mut statistics,
        );
        let state = statistics.single().unwrap();
        assert!(state.error.is_none(), "{:?}", state.error);
        total += state.sprites;
    }
    total
}
