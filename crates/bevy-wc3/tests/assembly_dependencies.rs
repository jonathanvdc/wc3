//! Resolved dependencies retain source ownership through the ordinary assembly compiler.
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy_wc3::assembly::{
    compile_assembly, compile_assembly_with_dependencies, Assembly, Models, Part, RgbaImage,
    Source, SourceDependencies,
};
use bevy_wc3::{
    spawn_model, ChildModelBinding, ChildModelKey, ModelError, ModelMaterial, ModelRig,
    PartBinding, ParticleEffect, ParticleProgramControls, RenderDiagnosticKind,
};
use std::collections::HashMap;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::io;
use wc3::model::profiles::{mdlx::Popcorn, Universal};
use wc3::model::{
    Attachment, ChildModel, FeatureId, ImageResource, Model, ModelMount, MountTarget, NodeFeature,
    NodeId, PlacementAppearance, ResourceLocator,
};

fn resource(name: &'static str) -> ResourceLocator<'static> {
    ResourceLocator::Named {
        namespace: "assembly-test".into(),
        name: name.into(),
    }
}
fn pixels(_: &ImageResource<'static>) -> Result<RgbaImage, ModelError> {
    RgbaImage::new(1, 1, vec![255; 4]).map_err(|error| ModelError::new(error.to_string()))
}
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
fn source() -> (Model<'static>, NodeId, FeatureId, FeatureId, FeatureId) {
    let mut model = io::mdlx::decode_mdl(
        include_str!("fixtures/light_capture.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen::<Universal>();
    let node = model.nodes().iter().next().unwrap().0;
    let light = model
        .features()
        .iter()
        .find_map(|(id, value)| matches!(value.payload, NodeFeature::Light(_)).then_some(id))
        .unwrap();
    let (programme, child) = model
        .edit(|builder| {
            let mut emitter: Popcorn<'_> = builder.construct();
            emitter.visibility = builder.add_value(1.);
            emitter.life_span = builder.add_value(1.);
            emitter.emission_rate = builder.add_value(1.);
            emitter.speed = builder.add_value(1.);
            emitter.color = builder.add_value(Vec3::ONE);
            emitter.opacity = builder.add_value(1.);
            let programme =
                builder.add_feature(node, NodeFeature::ParticleProgram(Box::new(emitter)))?;
            let child_resource = builder.add_model_resource(resource("child"));
            let child = builder.add_feature(
                node,
                NodeFeature::Attachment(Attachment::Child(ChildModel {
                    resource: child_resource,
                    socket: None,
                    appearance: PlacementAppearance::Unmodified,
                })),
            )?;
            Ok((programme, child))
        })
        .unwrap();
    (model, node, programme, light, child)
}
fn part(source: Source, node: NodeId) -> Part {
    Part::new(
        source,
        PartBinding::Mounted(ModelMount::new(MountTarget::Node(node))),
    )
}

#[test]
fn empty_dependencies_preserve_existing_features_diagnostics_and_images() {
    let (model, _, _, _, _) = source();
    let models: Models<'_> = HashMap::from([(resource("root"), model)]);
    let plan = Assembly::new(Source::new(resource("root")));
    let mut old_images = Assets::default();
    let old = compile_assembly(&plan, &models, &mut old_images, pixels).unwrap();
    let mut new_images = Assets::default();
    let mut calls = Vec::new();
    let new = compile_assembly_with_dependencies(
        &plan,
        &models,
        &mut new_images,
        |key, model| {
            calls.push(key.clone());
            assert!(model.nodes().iter().next().is_some());
            Ok(SourceDependencies::default())
        },
        pixels,
    )
    .unwrap();
    assert_eq!(calls, [resource("root")]);
    assert_eq!(
        format!("{:?}", old.diagnostics()),
        format!("{:?}", new.diagnostics())
    );
    assert!(new.diagnostics().iter().any(|issue| {
        issue.kind == RenderDiagnosticKind::Resource
            && issue.message.contains("no prepared interpreter")
    }));
    assert_eq!(old_images.len(), new_images.len());
    assert!(
        !new_images.is_empty(),
        "the control exercises ordinary model image resolution"
    );
    assert_eq!(
        format!("{:?}", old.rest_bounds()),
        format!("{:?}", new.rest_bounds())
    );
}

#[test]
fn every_source_binding_is_checked_before_any_image_even_when_hidden() {
    let (root, root_node, _, _, _) = source();
    let (model, _, _, light, child) = source();
    let (foreign_model, _, foreign, _, _) = source();
    assert!(foreign_model.feature(foreign).is_some());
    let models: Models<'_> = [(resource("root"), root), (resource("part"), model)].into();
    for hidden in [false, true] {
        for invalid in 0..3 {
            let mut selected = Source::new(resource("part"));
            if hidden {
                selected.draws.hidden_instances = models[&selected.model]
                    .instances()
                    .iter()
                    .map(|(id, _)| id)
                    .collect();
            }
            let mut plan = Assembly::new(Source::new(resource("root")));
            plan.parts.push(part(selected, root_node));
            let mut image_calls = 0;
            let mut assets = Assets::default();
            let result = compile_assembly_with_dependencies(
                &plan,
                &models,
                &mut assets,
                |key, _| {
                    let mut resolved = SourceDependencies::default();
                    if *key == resource("part") {
                        match invalid {
                            0 => {
                                resolved.particle_effects.insert(foreign, effect());
                            }
                            1 => {
                                resolved.particle_effects.insert(light, effect());
                            }
                            _ => {
                                resolved.child_models.insert(
                                    ChildModelKey::particle_emitters(child),
                                    ChildModelBinding::new(Handle::default()),
                                );
                            }
                        }
                    }
                    Ok(resolved)
                },
                |image| {
                    image_calls += 1;
                    pixels(image)
                },
            );
            let error = match result {
                Ok(_) => panic!("invalid source binding compiled"),
                Err(error) => error,
            };
            assert!(
                error.to_string().contains(if invalid == 0 {
                    "absent or foreign"
                } else {
                    "does not match"
                }),
                "{error}"
            );
            assert_eq!(
                image_calls, 0,
                "even the valid root must not decode before part preflight"
            );
            assert!(assets.is_empty());
        }
    }
}

#[test]
fn invalid_plan_and_failed_resolver_do_not_decode_images() {
    let (model, _, _, _, _) = source();
    let models: Models<'_> = [(resource("root"), model)].into();
    let missing = Assembly::new(Source::new(resource("missing")));
    let mut resolutions = 0;
    let mut images = Assets::default();
    assert!(compile_assembly_with_dependencies(
        &missing,
        &models,
        &mut images,
        |_, _| {
            resolutions += 1;
            Ok(SourceDependencies::default())
        },
        |_| panic!("invalid plans cannot decode images"),
    )
    .is_err());
    assert_eq!(resolutions, 0);
    let valid = Assembly::new(Source::new(resource("root")));
    let result = compile_assembly_with_dependencies(
        &valid,
        &models,
        &mut images,
        |_, _| Err(ModelError::new("dependency preparation failed")),
        |_| panic!("failed preparation cannot decode images"),
    );
    let error = match result {
        Ok(_) => panic!("failed resolver compiled"),
        Err(error) => error,
    };
    assert_eq!(error.to_string(), "dependency preparation failed");
    assert!(images.is_empty());
}

#[test]
fn repeated_resources_share_programme_handles_but_keep_actor_controls_independent() {
    let (model, node, programme, _, child) = source();
    let models: Models<'_> = [(resource("root"), model)].into();
    let mut plan = Assembly::new(Source::new(resource("root")));
    plan.parts.push(part(Source::new(resource("root")), node));
    plan.parts.push(part(Source::new(resource("root")), node));
    let shared = effect();
    let mut calls = Vec::new();
    let compiled = compile_assembly_with_dependencies(
        &plan,
        &models,
        &mut Assets::default(),
        |key, _| {
            calls.push(key.clone());
            Ok(SourceDependencies {
                child_models: [(
                    ChildModelKey::attachment(child),
                    ChildModelBinding::new(Handle::default()),
                )]
                .into(),
                particle_effects: [(programme, shared.clone())].into(),
            })
        },
        pixels,
    )
    .unwrap();
    assert_eq!(calls, [resource("root")]);
    assert!(!compiled
        .diagnostics()
        .iter()
        .any(|issue| issue.message.contains("no prepared interpreter")));
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut materials = Assets::<ModelMaterial>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let (a, b) = {
        let mut commands = Commands::new(&mut queue, &world);
        (
            spawn_model(
                &mut commands,
                &mut meshes,
                &mut materials,
                &prepared,
                Transform::default(),
            ),
            spawn_model(
                &mut commands,
                &mut meshes,
                &mut materials,
                &prepared,
                Transform::default(),
            ),
        )
    };
    queue.apply(&mut world);
    let emitter = |world: &World, root| {
        world
            .get::<ModelRig>(root)
            .unwrap()
            .particle_program(programme)
            .unwrap()
    };
    let ae = emitter(&world, a);
    let be = emitter(&world, b);
    let mount = world.get::<ModelRig>(a).unwrap().mounts()[0];
    let ce = emitter(&world, mount);
    assert_ne!(ae, be);
    assert_ne!(ae, ce);
    let attribute = world
        .get::<ParticleProgramControls>(ae)
        .unwrap()
        .attribute("Game.EmissionRateMultiplier")
        .unwrap();
    world
        .get_mut::<ParticleProgramControls>(ae)
        .unwrap()
        .set(&attribute, 4f32)
        .unwrap();
    world
        .get_mut::<ParticleProgramControls>(be)
        .unwrap()
        .set(&attribute, 6f32)
        .unwrap();
    world
        .get_mut::<ParticleProgramControls>(ce)
        .unwrap()
        .set(&attribute, 8f32)
        .unwrap();
    let av = world
        .get::<ParticleProgramControls>(ae)
        .unwrap()
        .get(&attribute)
        .unwrap();
    let bv = world
        .get::<ParticleProgramControls>(be)
        .unwrap()
        .get(&attribute)
        .unwrap();
    let cv = world
        .get::<ParticleProgramControls>(ce)
        .unwrap()
        .get(&attribute)
        .unwrap();
    assert!(av.is_some());
    assert_ne!(av, bv);
    assert_ne!(av, cv);
    assert_ne!(bv, cv);
}

#[test]
fn resolution_is_stable_and_uses_resource_identity_not_model_equivalence() {
    let (root, node, _, _, _) = source();
    let other = root.clone();
    let models: Models<'_> = [(resource("root"), root), (resource("other"), other)].into();
    let mut plan = Assembly::new(Source::new(resource("root")));
    plan.parts.push(part(Source::new(resource("other")), node));
    plan.parts.push(part(Source::new(resource("root")), node));
    plan.parts.push(part(Source::new(resource("other")), node));
    let mut calls = Vec::new();
    compile_assembly_with_dependencies(
        &plan,
        &models,
        &mut Assets::default(),
        |key, _| {
            calls.push(key.clone());
            Ok(SourceDependencies::default())
        },
        pixels,
    )
    .unwrap();
    assert_eq!(calls, [resource("root"), resource("other")]);
}
