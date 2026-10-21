//! Public lookup and programme ownership remain actor-local without ECS traversal.
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy_wc3::{
    compile_model_with_options, spawn_model, CompileOptions, ModelMaterial, ModelPart, ModelRig,
    PartBinding, ParticleEffect, ParticleProgramControlError, ParticleProgramControls,
};
use std::collections::HashMap;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::profiles::{mdlx::Popcorn, Universal};
use wc3::model::{FeatureId, Model, ModelMetadata, ModelMount, MountTarget, NodeFeature};

fn effect() -> ParticleEffect {
    let document =
        Document::read(include_bytes!("../../wc3/tests/fixtures/pkb/controls.pkb")).unwrap();
    ParticleEffect::new(
        document
            .compile_effect(1, &EffectCompileOptions::default())
            .unwrap(),
        |_| Some(Handle::default()),
    )
}
#[test]
fn public_lookup_is_per_rig_and_handles_survive_only_the_same_immutable_programme() {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("socket").unwrap();
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
    let shared = effect();
    let compile = |effect| {
        compile_model_with_options(
            &model,
            CompileOptions {
                particle_effects: HashMap::from([(feature, effect)]),
                ..Default::default()
            },
            |_| None,
        )
        .unwrap()
    };
    let compiled = compile(shared.clone());
    let combined = compiled
        .compose([ModelPart {
            model: &compiled,
            binding: PartBinding::Mounted(ModelMount::new(MountTarget::Node(node))),
            clips: HashMap::new(),
        }])
        .unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut materials = Assets::<ModelMaterial>::default();
    let prepared = combined.prepare(&mut meshes, &mut binds);
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let (a, b) = {
        let mut commands = Commands::new(&mut queue, &world);
        let a = spawn_model(
            &mut commands,
            &mut meshes,
            &mut materials,
            &prepared,
            Transform::default(),
        );
        let b = spawn_model(
            &mut commands,
            &mut meshes,
            &mut materials,
            &prepared,
            Transform::default(),
        );
        (a, b)
    };
    assert!(
        world.get::<ModelRig>(a).is_none(),
        "pending commands do not imply a ready rig"
    );
    queue.apply(&mut world);
    let ae = world
        .get::<ModelRig>(a)
        .unwrap()
        .particle_program(feature)
        .unwrap();
    let be = world
        .get::<ModelRig>(b)
        .unwrap()
        .particle_program(feature)
        .unwrap();
    assert_ne!(ae, be);
    let mounted = world.get::<ModelRig>(a).unwrap().mounts()[0];
    let child = world
        .get::<ModelRig>(mounted)
        .unwrap()
        .particle_program(feature)
        .unwrap();
    assert_ne!(
        ae, child,
        "parent lookup must not recurse into a mounted rig"
    );
    let mut foreign = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let other_node = foreign.add_named_node("other").unwrap();
    let other: Popcorn<'_> = foreign.construct();
    let other_feature: FeatureId = foreign
        .add_feature(other_node, NodeFeature::ParticleProgram(Box::new(other)))
        .unwrap();
    assert!(world
        .get::<ModelRig>(a)
        .unwrap()
        .particle_program(other_feature)
        .is_none());
    let rate = world
        .get::<ParticleProgramControls>(ae)
        .unwrap()
        .attribute("Game.EmissionRateMultiplier")
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
    world
        .get_mut::<ParticleProgramControls>(child)
        .unwrap()
        .set(&rate, 8f32)
        .unwrap();
    let same = compile(shared).prepare(&mut meshes, &mut binds);
    let different = compile(effect()).prepare(&mut meshes, &mut binds);
    // Replacement invalidates entity lifetime; shared programme bindings remain valid.
    world.entity_mut(a).despawn();
    let (same_root, different_root) = {
        let mut commands = Commands::new(&mut queue, &world);
        (
            spawn_model(
                &mut commands,
                &mut meshes,
                &mut materials,
                &same,
                Transform::default(),
            ),
            spawn_model(
                &mut commands,
                &mut meshes,
                &mut materials,
                &different,
                Transform::default(),
            ),
        )
    };
    queue.apply(&mut world);
    assert!(world.get::<ParticleProgramControls>(ae).is_none());
    let se = world
        .get::<ModelRig>(same_root)
        .unwrap()
        .particle_program(feature)
        .unwrap();
    let de = world
        .get::<ModelRig>(different_root)
        .unwrap()
        .particle_program(feature)
        .unwrap();
    world
        .get_mut::<ParticleProgramControls>(se)
        .unwrap()
        .set(&rate, 3f32)
        .unwrap();
    assert_eq!(
        world
            .get_mut::<ParticleProgramControls>(de)
            .unwrap()
            .set(&rate, 3f32),
        Err(ParticleProgramControlError::WrongProgram)
    );
}
