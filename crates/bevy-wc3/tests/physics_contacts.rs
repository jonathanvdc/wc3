//! Public contact admission and runtime quota observations through ordinary rigs.
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::shader::Shader;
use bevy::time::TimeUpdateStrategy;
use bevy_wc3::{
    compile_model_with_options, spawn_model, CompileOptions, CompiledModel, ModelMaterial,
    ModelRig, PhysicsContactError, PhysicsContactIssue, PhysicsContactLimits, PhysicsContactPolicy,
    PhysicsContactStats, PhysicsSimulation, Wc3BevyPlugin,
};
use std::time::Duration;
use wc3::model::profiles::Universal;
use wc3::model::systems::model::System as ModelSystem;
use wc3::model::systems::model_physics::{Body, Physics, Shape, ShapeGeometry};
use wc3::model::systems::System as SemanticSystem;
use wc3::model::values::model::{PhysicsBodyKind, PhysicsKind};
use wc3::model::{Model, ModelMetadata, NodeId};

fn cpu_model(compiled: &CompiledModel) -> (App, Entity) {
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut materials = Assets::<ModelMaterial>::default();
    let mut app = App::new();
    // CPU-only plugin path. Asset/mesh support is supplied explicitly, with no
    // renderer/GPU, so the ordinary scheduling and rig observation are exercised.
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        VisibilityPlugin,
        MeshPlugin,
        ImagePlugin::default(),
    ));
    app.init_asset::<Shader>();
    app.add_plugins(Wc3BevyPlugin);
    let mut queue = CommandQueue::default();
    let entity = spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    queue.apply(app.world_mut());
    app.insert_resource(materials);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    (app, entity)
}

#[test]
fn requested_unavailable_contacts_are_observable_and_cleared_when_policy_changes() {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let mut bodies = Vec::new();
    for index in 0..2 {
        let node = builder.add_named_node(format!("body{index}")).unwrap();
        bodies.push(Body {
            node,
            kind: PhysicsBodyKind::Dynamic,
            gravity_scale: 1.,
            inertia_scale: 1.,
            linear_damping: 0.,
            angular_damping: 0.,
            follow_factor: 0.,
            parent_body: None,
            attachment_host: false,
            ragdoll_root: false,
            shapes: vec![Shape {
                transform: Mat4::IDENTITY,
                geometry: ShapeGeometry::Sphere(0.5),
                friction: 0.,
                restitution: 0.,
                density: 1.,
                mass_properties: None,
                game_flags: Default::default(),
                collision_mask: Default::default(),
            }],
        });
    }
    builder
        .add_system(SemanticSystem::M2(ModelSystem::Physics(Physics {
            kind: PhysicsKind::WornItem,
            resource: None,
            bodies,
            joints: Vec::new(),
            collision_pairs: Vec::new(),
            wind_coefficients: None,
            follow_allow_list: Default::default(),
            vegetation: Vec::new(),
        })))
        .unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_model_with_options(
        &model,
        CompileOptions {
            physics_contact_pair_budget: 0,
            ..Default::default()
        },
        |_| None,
    )
    .unwrap();
    let (mut app, entity) = cpu_model(&compiled);
    app.world_mut()
        .entity_mut(entity)
        .insert(PhysicsSimulation {
            contact_policy: PhysicsContactPolicy::UnconnectedDynamics,
            ..Default::default()
        });
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(entity)
            .unwrap()
            .physics_contact_error(),
        Some(PhysicsContactError::PairBudgetExceeded {
            candidates: 1,
            limit: 0
        })
    );
    app.world_mut()
        .get_mut::<PhysicsSimulation>(entity)
        .unwrap()
        .contact_policy = PhysicsContactPolicy::AuthoredPairs;
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(entity)
            .unwrap()
            .physics_contact_error(),
        None
    );
    app.world_mut()
        .get_mut::<PhysicsSimulation>(entity)
        .unwrap()
        .contact_policy = PhysicsContactPolicy::UnconnectedDynamics;
    app.update();
    assert!(app
        .world()
        .get::<ModelRig>(entity)
        .unwrap()
        .physics_contact_error()
        .is_some());
    app.world_mut()
        .entity_mut(entity)
        .remove::<PhysicsSimulation>();
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(entity)
            .unwrap()
            .physics_contact_error(),
        None
    );
}

fn runtime_contact_model() -> (Model<'static, Universal>, NodeId) {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let wall = builder.add_named_node("kinematic hull").unwrap();
    let node = builder.new_node("dynamic sphere");
    builder
        .values_mut()
        .set(node.transform.translation, Vec3::new(0.7, 0.1, 0.1))
        .unwrap();
    let dynamic = builder.add_node(node).unwrap();
    let hull = ShapeGeometry::ConvexHull {
        vertices: (0..8)
            .map(|bits| {
                Vec3::new(
                    if bits & 1 == 0 { -0.5 } else { 0.5 },
                    if bits & 2 == 0 { -0.5 } else { 0.5 },
                    if bits & 4 == 0 { -0.5 } else { 0.5 },
                )
            })
            .collect(),
        planes: [
            Vec4::new(-1., 0., 0., 0.5),
            Vec4::new(1., 0., 0., 0.5),
            Vec4::new(0., -1., 0., 0.5),
            Vec4::new(0., 1., 0., 0.5),
            Vec4::new(0., 0., -1., 0.5),
            Vec4::new(0., 0., 1., 0.5),
        ]
        .into_iter()
        .collect(),
        faces: vec![
            vec![0, 4, 6, 2],
            vec![1, 3, 7, 5],
            vec![0, 1, 5, 4],
            vec![2, 6, 7, 3],
            vec![0, 2, 3, 1],
            vec![4, 5, 7, 6],
        ]
        .into(),
    };
    let bodies = [
        (wall, PhysicsBodyKind::Kinematic, hull),
        (
            dynamic,
            PhysicsBodyKind::Dynamic,
            ShapeGeometry::Sphere(0.25),
        ),
    ]
    .into_iter()
    .map(|(node, kind, geometry)| Body {
        node,
        kind,
        gravity_scale: 1.,
        inertia_scale: 1.,
        linear_damping: 0.,
        angular_damping: 0.,
        follow_factor: 0.,
        parent_body: None,
        attachment_host: false,
        ragdoll_root: false,
        shapes: vec![Shape {
            transform: Mat4::IDENTITY,
            geometry,
            friction: 0.,
            restitution: 0.,
            density: 1.,
            mass_properties: None,
            game_flags: Default::default(),
            collision_mask: Default::default(),
        }],
    })
    .collect();
    builder
        .add_system(SemanticSystem::M2(ModelSystem::Physics(Physics {
            kind: PhysicsKind::WornItem,
            resource: None,
            bodies,
            joints: Vec::new(),
            collision_pairs: vec![(wall, dynamic)],
            wind_coefficients: None,
            follow_allow_list: Default::default(),
            vegetation: Vec::new(),
        })))
        .unwrap();
    (builder.finish().unwrap(), dynamic)
}

fn advance_frame(app: &mut App, seconds: f64) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        seconds,
    )));
    app.update();
}

fn runtime_stats(app: &App, entity: Entity) -> PhysicsContactStats {
    let rig = app.world().get::<ModelRig>(entity).unwrap();
    assert_eq!(
        rig.physics_contact_error(),
        None,
        "valid authored pairs must not become an admission-policy error"
    );
    rig.physics_contact_stats()
}

fn position(app: &App, entity: Entity, node: NodeId) -> Vec3 {
    let node_entity = app
        .world()
        .get::<ModelRig>(entity)
        .unwrap()
        .node(node)
        .unwrap();
    app.world()
        .get::<GlobalTransform>(node_entity)
        .unwrap()
        .translation()
}

#[test]
fn public_runtime_quotas_recover_and_preserve_observations_until_explicit_reset() {
    const STEP: f64 = 1. / 64.;
    for issue in [
        PhysicsContactIssue::PairLimit,
        PhysicsContactIssue::QueryLimit,
        PhysicsContactIssue::SupportLimit,
    ] {
        let (model, dynamic) = runtime_contact_model();
        let compiled =
            compile_model_with_options(&model, CompileOptions::default(), |_| None).unwrap();
        let (mut app, entity) = cpu_model(&compiled);
        let mut limits = PhysicsContactLimits::default();
        match issue {
            PhysicsContactIssue::PairLimit => limits.pair_checks_per_update = 0,
            PhysicsContactIssue::QueryLimit => limits.queries_per_update = 0,
            PhysicsContactIssue::SupportLimit => limits.support_vertices_per_update = 1,
            _ => unreachable!(),
        }
        app.world_mut()
            .entity_mut(entity)
            .insert(PhysicsSimulation {
                timestep: STEP,
                iterations: 1,
                contact_limits: limits,
                ..Default::default()
            });
        app.update(); // Observe the initial pose at time zero.
        assert_eq!(runtime_stats(&app, entity), PhysicsContactStats::default());
        let initial = position(&app, entity, dynamic);
        advance_frame(&mut app, STEP);
        let exhausted = runtime_stats(&app, entity);
        assert_eq!(exhausted.last_issue, Some(issue));
        assert_eq!(exhausted.inconclusive_queries, 0);
        assert!(
            position(&app, entity, dynamic).distance(initial) > 1e-6,
            "contact exhaustion must not suspend ordinary integration"
        );
        match issue {
            PhysicsContactIssue::PairLimit => {
                assert!(exhausted.pair_budget_exhaustions > 0);
                assert_eq!(exhausted.pair_checks, 0);
                assert_eq!(exhausted.queries, 0);
                assert_eq!(exhausted.support_vertices, 0);
            }
            PhysicsContactIssue::QueryLimit => {
                assert!(exhausted.query_budget_exhaustions > 0);
                assert_eq!(exhausted.queries, 0);
                assert_eq!(exhausted.support_vertices, 0);
            }
            PhysicsContactIssue::SupportLimit => {
                assert!(exhausted.support_budget_exhaustions > 0);
                assert!(exhausted.queries > 0);
                assert_eq!(exhausted.support_vertices, 1);
            }
            _ => unreachable!(),
        }
        // Recovery changes only allowances, without resetting the instance.
        app.world_mut()
            .get_mut::<PhysicsSimulation>(entity)
            .unwrap()
            .contact_limits = PhysicsContactLimits::default();
        advance_frame(&mut app, STEP);
        let recovered = runtime_stats(&app, entity);
        assert!(recovered.queries > exhausted.queries);
        assert!(recovered.support_vertices > exhausted.support_vertices);
        assert_eq!(
            recovered.pair_budget_exhaustions,
            exhausted.pair_budget_exhaustions
        );
        assert_eq!(
            recovered.query_budget_exhaustions,
            exhausted.query_budget_exhaustions
        );
        assert_eq!(
            recovered.support_budget_exhaustions,
            exhausted.support_budget_exhaustions
        );
        assert_eq!(recovered.inconclusive_queries, 0);
        assert_eq!(
            recovered.last_issue,
            Some(issue),
            "successful work retains the latest prior issue"
        );
        advance_frame(&mut app, 0.);
        assert_eq!(runtime_stats(&app, entity), recovered);
        app.world_mut()
            .get_mut::<PhysicsSimulation>(entity)
            .unwrap()
            .paused = true;
        advance_frame(&mut app, STEP);
        assert_eq!(runtime_stats(&app, entity), recovered);
        app.world_mut()
            .get_mut::<PhysicsSimulation>(entity)
            .unwrap()
            .reset();
        advance_frame(&mut app, 0.);
        assert_eq!(runtime_stats(&app, entity), PhysicsContactStats::default());
        app.world_mut()
            .get_mut::<PhysicsSimulation>(entity)
            .unwrap()
            .paused = false;
        advance_frame(&mut app, STEP);
        let restarted = runtime_stats(&app, entity);
        assert!(restarted.queries > 0);
        assert!(restarted.support_vertices > 0);
        assert_eq!(restarted.last_issue, None);
    }
}
