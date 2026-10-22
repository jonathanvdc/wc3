//! Public application admission policy and observable unavailable preparation.
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::shader::Shader;
use bevy_wc3::{
    compile_model_with_options, spawn_model, CompileOptions, ModelMaterial, ModelRig,
    PhysicsContactError, PhysicsContactPolicy, PhysicsSimulation, Wc3BevyPlugin,
};
use wc3::model::profiles::Universal;
use wc3::model::systems::model::System as ModelSystem;
use wc3::model::systems::model_physics::{Body, Physics, Shape, ShapeGeometry};
use wc3::model::systems::System as SemanticSystem;
use wc3::model::values::model::{PhysicsBodyKind, PhysicsKind};
use wc3::model::{Model, ModelMetadata};

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
