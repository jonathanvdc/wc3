//! Programme dependencies belong to authored features, independently of mesh selection.
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::{Assets, Commands, Handle, Image, Mesh, Transform, Vec3, World};
use bevy_wc3::{
    compile_model_with_options, spawn_model, CompileOptions, CompiledModel, ModelMaterial,
    ModelRig, ParticleEffect, RenderDiagnosticKind, RenderDiagnosticSource,
};
use std::collections::HashMap;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::io;
use wc3::model::profiles::{mdlx::Popcorn, Universal};
use wc3::model::{FeatureId, Model, NodeFeature};

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

fn source() -> (Model<'static>, FeatureId, FeatureId) {
    let mut model = io::mdlx::decode_mdl(
        include_str!("fixtures/light_capture.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen::<Universal>();
    let node = model.nodes().iter().next().unwrap().0;
    let wrong_family = model
        .features()
        .iter()
        .find_map(|(feature, value)| {
            matches!(value.payload, NodeFeature::Light(_)).then_some(feature)
        })
        .unwrap();
    let feature = model
        .edit(|builder| {
            let mut emitter: Popcorn<'_> = builder.construct();
            emitter.visibility = builder.add_value(1.);
            emitter.life_span = builder.add_value(1.);
            emitter.emission_rate = builder.add_value(1.);
            emitter.speed = builder.add_value(1.);
            emitter.color = builder.add_value(Vec3::ONE);
            emitter.opacity = builder.add_value(1.);
            builder.add_feature(node, NodeFeature::ParticleProgram(Box::new(emitter)))
        })
        .unwrap();
    assert!(
        model.instances().iter().next().is_some(),
        "hidden-draw controls need geometry"
    );
    (model, feature, wrong_family)
}

fn options(model: &Model<'_>, hidden: bool) -> CompileOptions {
    let mut options = CompileOptions::default();
    if hidden {
        options.draws.hidden_instances = model.instances().iter().map(|(id, _)| id).collect();
    }
    options
}

fn assert_rejected(key: FeatureId, reason: &str, model: &Model<'_>) {
    for hidden in [false, true] {
        let mut options = options(model, hidden);
        options.particle_effects.insert(key, effect());
        let mut image_requests = 0;
        let result = compile_model_with_options(model, options, |_| {
            image_requests += 1;
            Some(Handle::default())
        });
        let error = match result {
            Ok(_) => panic!("invalid programme key compiled with hidden={hidden}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains(reason), "{error}");
        assert_eq!(image_requests, 0, "binding checks precede image resolution");
    }
}

#[test]
fn foreign_programme_binding_is_rejected_before_images_even_for_hidden_geometry() {
    let (model, _, _) = source();
    let (foreign_model, foreign, _) = source();
    assert!(foreign_model.feature(foreign).is_some());
    assert!(model.feature(foreign).is_none());
    assert_rejected(foreign, "absent or foreign feature", &model);
}

#[test]
fn wrong_family_programme_binding_is_rejected_before_images_even_for_hidden_geometry() {
    let (model, _, light) = source();
    assert_rejected(
        light,
        "does not match an authored particle programme",
        &model,
    );
}

fn has_live_programme(compiled: &CompiledModel, feature: FeatureId) -> bool {
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut materials = Assets::<ModelMaterial>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut queue, &world),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    queue.apply(&mut world);
    world
        .get::<ModelRig>(root)
        .unwrap()
        .particle_program(feature)
        .is_some()
}

#[test]
fn valid_authored_binding_retains_its_emitter_even_when_geometry_is_hidden() {
    let (model, feature, _) = source();
    let effect = effect();
    for hidden in [false, true] {
        let mut options = options(&model, hidden);
        options.particle_effects = HashMap::from([(feature, effect.clone())]);
        let mut image_requests = 0;
        let compiled = compile_model_with_options(&model, options, |_| {
            image_requests += 1;
            Some(Handle::<Image>::default())
        })
        .unwrap();
        if !hidden {
            assert!(
                image_requests > 0,
                "visible control must exercise image resolution"
            );
        }
        assert!(has_live_programme(&compiled, feature));
        assert!(!compiled.diagnostics().iter().any(|diagnostic| {
            diagnostic.kind == RenderDiagnosticKind::Resource
                && diagnostic.source == RenderDiagnosticSource::Feature(feature)
                && diagnostic.message.contains("no prepared interpreter")
        }));
    }
}

#[test]
fn missing_prepared_programme_remains_an_optional_feature_diagnostic() {
    let (model, feature, _) = source();
    for hidden in [false, true] {
        let compiled =
            compile_model_with_options(
                &model,
                options(&model, hidden),
                |_| Some(Handle::default()),
            )
            .unwrap();
        assert!(!has_live_programme(&compiled, feature));
        assert!(compiled.diagnostics().iter().any(|diagnostic| {
            diagnostic.kind == RenderDiagnosticKind::Resource
                && diagnostic.source == RenderDiagnosticSource::Feature(feature)
                && diagnostic.message.contains("no prepared interpreter")
        }));
    }
}
