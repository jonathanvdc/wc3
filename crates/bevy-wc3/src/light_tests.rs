use super::*;
use crate::animation::pose::animate_nodes;
use crate::assets::model::Wc3Model;
use crate::instance::spawn::{spawn_prepared_model, Wc3NodeEntities};
use crate::materials::Wc3LayerMaterial;
use crate::preparation::prepare_model;
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use wc3::model::animation::Animatable;

fn scene() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, VisibilityPlugin));
    app.insert_resource(GlobalAmbientLight {
        brightness: 123.0,
        ..default()
    });
    app.add_systems(Update, (animate_nodes, animate_lights).chain());
    let source = Wc3Model::decode_mdl(include_str!("../tests/fixtures/light_capture.mdl")).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut poses = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut poses, &source, |_| None).unwrap();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let first = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    let second = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    queue.apply(app.world_mut());
    app.insert_resource(meshes);
    app.insert_resource(materials);
    app.insert_resource(poses);
    (app, first, second)
}

fn light_entity(app: &mut App, root: Entity, id: u32) -> Entity {
    app.world_mut()
        .query::<(Entity, &Wc3Light)>()
        .iter(app.world())
        .find(|(_, light)| light.root == root && light.definition.node.object_id == id)
        .unwrap()
        .0
}

#[test]
fn lights_sample_tracks_follow_nodes_and_keep_instances_independent() {
    let (mut app, first, second) = scene();
    app.world_mut()
        .get_mut::<Wc3Animation>(first)
        .unwrap()
        .elapsed_ms = 500.0;
    app.world_mut()
        .entity_mut(first)
        .insert(Transform::from_xyz(10.0, 0.0, 0.0));
    app.update();
    let first_point = light_entity(&mut app, first, 2);
    let second_point = light_entity(&mut app, second, 2);
    let point = app.world().get::<PointLight>(first_point).unwrap();
    assert_eq!(point.intensity, 75_000.0);
    assert_eq!(point.range, 10.0);
    assert_eq!(point.color, Color::linear_rgb(1.0, 0.0, 0.0));
    assert!(point.shadow_maps_enabled);
    assert_eq!(
        app.world()
            .get::<PointLight>(second_point)
            .unwrap()
            .intensity,
        50_000.0
    );
    let global = app.world().get::<GlobalTransform>(first_point).unwrap();
    assert!((global.translation() - Vec3::new(10.0, 0.0, 2.0)).length() < 0.001);
    let node = app
        .world()
        .get::<Wc3NodeEntities>(first)
        .unwrap()
        .get(2)
        .unwrap();
    assert_eq!(
        app.world().get::<ChildOf>(first_point).unwrap().parent(),
        node
    );

    app.world_mut()
        .get_mut::<Wc3Animation>(first)
        .unwrap()
        .elapsed_ms = 750.0;
    app.update();
    assert_eq!(
        app.world()
            .get::<PointLight>(first_point)
            .unwrap()
            .intensity,
        0.0
    );
    assert_eq!(
        *app.world().get::<Visibility>(first_point).unwrap(),
        Visibility::Hidden
    );
    // Seeking back restores the same properties; it does not alter the other instance.
    app.world_mut()
        .get_mut::<Wc3Animation>(first)
        .unwrap()
        .elapsed_ms = 0.0;
    app.update();
    assert_eq!(
        app.world()
            .get::<PointLight>(first_point)
            .unwrap()
            .intensity,
        50_000.0
    );
    assert_eq!(
        *app.world().get::<Visibility>(first_point).unwrap(),
        Visibility::Inherited
    );
    app.world_mut().entity_mut(first).insert(Visibility::Hidden);
    app.update();
    assert!(!app
        .world()
        .get::<InheritedVisibility>(first_point)
        .unwrap()
        .get());
    app.world_mut().entity_mut(first).despawn();
    assert!(app.world().get_entity(first_point).is_err());
    assert!(app.world().get_entity(second_point).is_ok());
}

#[test]
fn settings_and_per_light_suppression_preserve_consumer_light_fields() {
    let (mut app, first, _) = scene();
    let entity = light_entity(&mut app, first, 2);
    app.world_mut().entity_mut(first).insert(Wc3LightSettings {
        point_intensity_scale: 20.0,
        range_scale: 2.0,
        shadows_enabled: false,
        ..default()
    });
    app.world_mut()
        .get_mut::<PointLight>(entity)
        .unwrap()
        .radius = 0.25;
    app.update();
    let point = app.world().get::<PointLight>(entity).unwrap();
    assert_eq!(point.intensity, 1_000.0);
    assert_eq!(point.range, 16.0);
    assert_eq!(point.radius, 0.25);
    assert!(!point.shadow_maps_enabled);
    app.world_mut().get_mut::<Wc3Light>(entity).unwrap().enabled = false;
    app.update();
    assert_eq!(
        app.world().get::<PointLight>(entity).unwrap().intensity,
        0.0
    );
    app.world_mut().get_mut::<Wc3Light>(entity).unwrap().enabled = true;
    app.world_mut()
        .get_mut::<Wc3LightSettings>(first)
        .unwrap()
        .enabled = false;
    app.update();
    assert_eq!(
        app.world().get::<PointLight>(entity).unwrap().intensity,
        0.0
    );
}

#[test]
fn directional_light_uses_parent_rotation_and_ambient_creates_no_scene_light() {
    let (mut app, first, _) = scene();
    let directional = light_entity(&mut app, first, 3);
    app.world_mut()
        .get_mut::<Wc3Light>(directional)
        .unwrap()
        .definition
        .intensity = Animatable::Static(2.0);
    app.world_mut()
        .entity_mut(first)
        .insert(Transform::from_rotation(Quat::from_rotation_y(0.7)));
    app.update();
    assert_eq!(
        app.world()
            .get::<DirectionalLight>(directional)
            .unwrap()
            .illuminance,
        20_000.0
    );
    let global = app.world().get::<GlobalTransform>(directional).unwrap();
    assert!((global.back().as_vec3() - Quat::from_rotation_y(0.7) * Vec3::Z).length() < 0.001);
    let ambient = light_entity(&mut app, first, 4);
    assert!(app.world().get::<PointLight>(ambient).is_none());
    assert!(app.world().get::<DirectionalLight>(ambient).is_none());
    assert_eq!(
        app.world().resource::<GlobalAmbientLight>().brightness,
        123.0
    );
}

#[test]
fn invalid_properties_do_not_reach_bevy_lights() {
    let (mut app, first, _) = scene();
    let entity = light_entity(&mut app, first, 2);
    {
        let mut light = app.world_mut().get_mut::<Wc3Light>(entity).unwrap();
        light.definition.attenuation_end = Animatable::Static(f32::NAN);
        light.definition.intensity = Animatable::Static(-1.0);
    }
    app.world_mut().entity_mut(first).insert(Wc3LightSettings {
        fallback_range: 42.0,
        ..default()
    });
    app.update();
    let point = app.world().get::<PointLight>(entity).unwrap();
    assert_eq!(point.range, 42.0);
    assert_eq!(point.intensity, 0.0);
}

#[test]
fn global_intensity_clock_keeps_running_after_nonlooping_sequence_ends() {
    let (mut app, first, _) = scene();
    let entity = light_entity(&mut app, first, 2);
    app.world_mut()
        .get_mut::<Wc3Light>(entity)
        .unwrap()
        .definition
        .visibility = None;
    app.world_mut()
        .get_mut::<Wc3Animation>(first)
        .unwrap()
        .elapsed_ms = 1250.0;
    app.update();
    let point = app.world().get::<PointLight>(entity).unwrap();
    assert_eq!(point.intensity, 62_500.0);
    assert_eq!(point.range, 12.0);
}
