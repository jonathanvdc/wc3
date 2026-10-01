use super::*;
use crate::animation::AnimatedLayer;
use crate::animation::{animate_layers, animate_nodes, Wc3Animation};
use crate::instance::{Wc3ModelOwner, Wc3OwnedModels};
use crate::particle_emitter2::Particle2State;
use crate::ribbon_emitter::{RibbonInstances, RibbonLayer, RibbonState};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::{CommandQueue, World};
use bevy::mesh::skinning::SkinnedMesh;
use wc3::model::animation::{Animatable, Track, ValueKeyframe};
use wc3::model::materials::Layer;
use wc3::model::materials::LayerFilterMode;
use wc3::model::mdl::Read as _;
use wc3::model::{Model, V1800};

#[test]
fn child_instances_inherit_visibility_and_cleanup_without_sharing_animation() {
    let source = Wc3Model::decode(include_bytes!(
        "../../../wc3/tests/fixtures/mdl/quad_model.mdx"
    ))
    .unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, VisibilityPlugin));
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut materials, &mut bindposes, &source, |_| {
        None
    })
    .unwrap();
    let owner = app
        .world_mut()
        .spawn((Transform::from_xyz(100.0, 0.0, 0.0), Visibility::Inherited))
        .id();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let following = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    commands.entity(following).insert((
        ChildOf(owner),
        Wc3ModelOwner(owner),
        Transform::from_xyz(5.0, 0.0, 0.0),
    ));
    let detached = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    commands
        .entity(detached)
        .insert((Wc3ModelOwner(owner), Transform::from_xyz(20.0, 0.0, 0.0)));
    // Include effects in the same ownership/visibility check.
    let particles =
        Wc3Model::decode_mdl(include_str!("../../tests/fixtures/particle_capture.mdl")).unwrap();
    let particle_prepared = prepare_model(
        &mut meshes,
        &mut materials,
        &mut bindposes,
        &particles,
        |_| None,
    )
    .unwrap();
    let effects = spawn_prepared_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &particle_prepared,
    );
    commands
        .entity(effects)
        .insert((ChildOf(following), Wc3ModelOwner(following)));
    queue.apply(app.world_mut());
    app.insert_resource(meshes);
    app.insert_resource(materials);
    app.insert_resource(bindposes);
    app.add_systems(Update, (animate_nodes, animate_layers));
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3OwnedModels>(owner)
            .unwrap()
            .iter()
            .count(),
        2
    );
    let following_nodes = app.world().get::<Wc3NodeEntities>(following).unwrap();
    let following_bone = following_nodes.get(0).unwrap();
    let detached_bone = app
        .world()
        .get::<Wc3NodeEntities>(detached)
        .unwrap()
        .get(0)
        .unwrap();
    assert_ne!(following_bone, detached_bone);
    assert_eq!(
        app.world()
            .get::<GlobalTransform>(following_bone)
            .unwrap()
            .translation()
            .x,
        105.0
    );
    assert_eq!(
        app.world()
            .get::<GlobalTransform>(detached_bone)
            .unwrap()
            .translation()
            .x,
        20.0
    );
    let mut layers = app
        .world_mut()
        .query::<(Entity, &AnimatedLayer, &SkinnedMesh, &ChildOf)>();
    let geometry: Vec<_> = layers
        .iter(app.world())
        .map(|(entity, layer, skin, parent)| {
            assert_eq!(parent.parent(), layer.root);
            assert_eq!(
                skin.joints,
                if layer.root == following {
                    vec![following_bone]
                } else {
                    vec![detached_bone]
                }
            );
            entity
        })
        .collect();
    assert_eq!(geometry.len(), 2);
    app.world_mut()
        .get_mut::<Wc3Animation>(following)
        .unwrap()
        .elapsed_ms = 500.0;
    assert_eq!(
        app.world()
            .get::<Wc3Animation>(detached)
            .unwrap()
            .elapsed_ms,
        0.0
    );
    *app.world_mut().get_mut::<Visibility>(owner).unwrap() = Visibility::Hidden;
    app.update();
    for entity in [following, effects, following_bone] {
        assert!(!app
            .world()
            .get::<InheritedVisibility>(entity)
            .unwrap()
            .get());
    }
    for entity in &geometry {
        let root = app.world().get::<AnimatedLayer>(*entity).unwrap().root;
        assert_eq!(
            app.world()
                .get::<InheritedVisibility>(*entity)
                .unwrap()
                .get(),
            root == detached
        );
    }
    let mut emitter_query = app.world_mut().query::<(Entity, &Particle2State)>();
    let emitters: Vec<_> = emitter_query
        .iter(app.world())
        .map(|(entity, state)| {
            assert_eq!(state.root, effects);
            assert!(!app
                .world()
                .get::<InheritedVisibility>(entity)
                .unwrap()
                .get());
            entity
        })
        .collect();
    assert_eq!(emitters.len(), 2);
    assert!(app
        .world()
        .get::<InheritedVisibility>(detached)
        .unwrap()
        .get());
    *app.world_mut().get_mut::<Visibility>(owner).unwrap() = Visibility::Inherited;
    app.update();
    assert!(app
        .world()
        .get::<InheritedVisibility>(effects)
        .unwrap()
        .get());
    app.world_mut().entity_mut(owner).despawn();
    for entity in geometry.into_iter().chain(emitters).chain([
        owner,
        following,
        detached,
        effects,
        following_bone,
        detached_bone,
    ]) {
        assert!(
            app.world().get_entity(entity).is_err(),
            "orphaned {entity:?}"
        );
    }
    let mut transforms = app.world_mut().query::<&Transform>();
    assert_eq!(transforms.iter(app.world()).count(), 0);
}

#[test]
fn prepared_assets_share_static_layers_and_isolate_animated_layers() {
    let bytes = include_bytes!("../../../wc3/tests/fixtures/mdl/quad_model.mdx");
    let mut source = Wc3Model::decode(bytes).unwrap();
    let mut material_records = source.model.materials();
    assert!(!material_records.is_empty());
    assert!(!material_records[0].layers.is_empty());
    material_records[0].layers[0].alpha.set_track(
        Track::linear(
            vec![
                ValueKeyframe {
                    frame: 0,
                    value: 1.0,
                },
                ValueKeyframe {
                    frame: 100,
                    value: 0.0,
                },
            ],
            None,
        )
        .unwrap(),
    );
    material_records[0].layers.push(Layer::new());
    source.model.set_materials(&material_records);

    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut materials, &mut bindposes, &source, |_| {
        None
    })
    .unwrap();
    let mesh_count = meshes.len();
    let static_count = materials.len();
    assert_eq!(static_count, 0);
    let bindpose_count = bindposes.len();
    let world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    assert_eq!(meshes.len(), mesh_count);
    assert_eq!(bindposes.len(), bindpose_count);
    assert!(materials.len() >= static_count + 2);
    assert!(!prepared.geosets.is_empty());
}

#[test]
fn ribbons_share_sections_across_layers_and_keep_instances_bindings_and_ownership_separate() {
    use crate::animation::advance_animation;
    use crate::ribbon_emitter::update_ribbons;
    use crate::texture_bindings::Wc3TextureSlot;
    use bevy::transform::TransformSystems;
    use std::sync::Arc;
    use std::time::Duration;

    let mut source =
        Wc3Model::decode_mdl(include_str!("../../tests/fixtures/ribbon_capture.mdl")).unwrap();
    let mut definitions = source.model.materials();
    let mut second = definitions[0].layers[0].clone();
    second.alpha = 0.5.into();
    second.filter_mode = LayerFilterMode::Additive;
    definitions[0].layers.push(second);
    source.model.set_materials(&definitions);
    let mut app = App::new();
    app.add_plugins((TransformPlugin, VisibilityPlugin));
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, (advance_animation, animate_nodes).chain());
    app.add_systems(
        PostUpdate,
        update_ribbons.after(TransformSystems::Propagate),
    );
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut materials, &mut bindposes, &source, |_| {
        None
    })
    .unwrap();
    let mut images = Assets::<Image>::default();
    let chosen = images.add(Image::default());
    let mut bindings = Wc3TextureBindings::default();
    bindings.set_slot(Wc3TextureSlot::Bitmap(0), chosen.clone());
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let fast = spawn_prepared_model_with_bindings(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        bindings,
    );
    let normal = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    queue.apply(app.world_mut());
    app.insert_resource(meshes);
    app.insert_resource(materials);
    app.insert_resource(bindposes);
    app.world_mut().get_mut::<Wc3Animation>(fast).unwrap().speed = 2.0;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(0.25));
    app.update();
    let mut query = app
        .world_mut()
        .query::<(Entity, &RibbonLayer, &RibbonInstances, &ChildOf)>();
    let layers: Vec<_> = query
        .iter(app.world())
        .map(|(entity, layer, data, parent)| {
            assert_eq!(
                parent.parent(),
                app.world().get::<RibbonState>(layer.emitter).unwrap().root
            );
            (entity, layer.emitter, data.clone(), parent.parent())
        })
        .collect();
    assert_eq!(layers.len(), 4);
    let fast_layers: Vec<_> = layers
        .iter()
        .filter(|(_, _, _, root)| *root == fast)
        .collect();
    let normal_layers: Vec<_> = layers
        .iter()
        .filter(|(_, _, _, root)| *root == normal)
        .collect();
    assert_eq!(fast_layers[0].2.live_indices.len(), 9);
    assert_eq!(normal_layers[0].2.live_indices.len(), 4);
    assert!(Arc::ptr_eq(
        &fast_layers[0].2.records,
        &fast_layers[1].2.records
    ));
    assert!(!Arc::ptr_eq(
        &fast_layers[0].2.records,
        &normal_layers[0].2.records
    ));
    for (_, _, data, _) in &fast_layers {
        assert_eq!(data.texture, Some(chosen.clone()));
    }
    assert!(fast_layers
        .iter()
        .any(|(_, _, data, _)| data.uniform.color[3] == 0.4));
    let snapshot = fast_layers[0].2.records.clone();
    app.world_mut()
        .get_mut::<Wc3Animation>(fast)
        .unwrap()
        .playing = false;
    let replacement = images.add(Image::default());
    app.world_mut()
        .get_mut::<Wc3TextureBindings>(fast)
        .unwrap()
        .set_slot(Wc3TextureSlot::Bitmap(0), replacement.clone());
    *app.world_mut().get_mut::<Visibility>(fast).unwrap() = Visibility::Hidden;
    app.update();
    for (entity, _, _, _) in &fast_layers {
        let data = app.world().get::<RibbonInstances>(*entity).unwrap();
        assert!(Arc::ptr_eq(&snapshot, &data.records));
        assert_eq!(data.texture, Some(replacement.clone()));
        assert!(!app
            .world()
            .get::<InheritedVisibility>(*entity)
            .unwrap()
            .get());
    }
    app.world_mut().entity_mut(fast).despawn();
    for (entity, emitter, _, _) in &fast_layers {
        assert!(app.world().get_entity(*entity).is_err());
        assert!(app.world().get_entity(*emitter).is_err());
    }
    assert!(app.world().get_entity(normal).is_ok());
}

#[test]
fn pre2_emitter_is_spawned_per_instance_and_owned_by_root() {
    let model =
        Model::<V1800>::decode_mdl(include_str!("../../tests/fixtures/attachment_parent.mdl"))
            .unwrap();
    let source = Wc3Model {
        source_version: 1800,
        model,
    };
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut materials, &mut bindposes, &source, |_| {
        None
    })
    .unwrap();
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let root = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    queue.apply(&mut world);
    let mut query = world.query::<(Entity, &Particle2State)>();
    let emitters: Vec<_> = query.iter(&world).collect();
    assert_eq!(emitters.len(), 1);
    assert_eq!(emitters[0].1.root, root);
    assert_eq!(
        world.entity(emitters[0].0).get::<ChildOf>(),
        Some(&ChildOf(root))
    );
}

#[test]
fn slot_binding_selects_replaceable_bitmap_for_geosets() {
    let mut source = Wc3Model::decode(include_bytes!(
        "../../../wc3/tests/fixtures/mdl/quad_model.mdx"
    ))
    .unwrap();
    let mut bitmaps = source.model.textures();
    bitmaps[0].path.set_text("").unwrap();
    bitmaps[0].replaceable_id = 1;
    source.model.set_textures(&bitmaps);
    let mut images = Assets::<Image>::default();
    let selected = images.add(Image::default());
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut materials, &mut bindposes, &source, |_| {
        None
    })
    .unwrap();
    let mut bindings = Wc3TextureBindings::default();
    bindings.set_replaceable(1, selected.clone());
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_prepared_model_with_bindings(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        bindings,
    );
    queue.apply(&mut world);
    let mut query = world.query::<&MeshMaterial3d<Wc3LayerMaterial>>();
    let handle = query.iter(&world).next().unwrap();
    assert_eq!(
        materials.get(&handle.0).unwrap().base.base_color_texture,
        Some(selected)
    );
}

#[test]
fn geoset_tints_isolate_shared_materials_and_model_instances() {
    let mut source =
        Wc3Model::decode_mdl(include_str!("../../tests/fixtures/geoset_capture.mdl")).unwrap();
    // A third geoset has nonwhite color data with the color flag disabled.
    let mut geosets = source.model.geosets();
    geosets.push(geosets[0].clone());
    let mut skipped = geosets[0].clone();
    skipped.set_level_of_detail(1);
    geosets.insert(0, skipped);
    source.model.set_geosets(&geosets);
    let mut animations = source.model.geoset_animations();
    let mut disabled = animations[0].clone();
    disabled.geoset_id = 2;
    disabled.flags.set_color(false);
    animations.push(disabled.clone());
    for animation in &mut animations {
        animation.geoset_id += 1;
    }
    let mut invalid = disabled;
    invalid.geoset_id = u32::MAX;
    animations.push(invalid);
    source.model.set_geoset_animations(&animations);
    let mut records = source.model.materials();
    let mut extra_layer = records[0].layers[0].clone();
    extra_layer.alpha = Animatable::Static(0.5);
    records[0].layers.push(extra_layer);
    source.model.set_materials(&records);
    let mut app = App::new();
    app.add_systems(Update, animate_layers);
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut materials, &mut bindposes, &source, |_| {
        None
    })
    .unwrap();
    assert_eq!(
        prepared
            .geosets
            .iter()
            .map(|geoset| geoset.geoset_id)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let first = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    let second = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    queue.apply(app.world_mut());
    app.insert_resource(materials);
    app.world_mut()
        .entity_mut(first)
        .get_mut::<Wc3Animation>()
        .unwrap()
        .elapsed_ms = 500.0;
    app.update();
    let mut query = app
        .world_mut()
        .query::<(&AnimatedLayer, &MeshMaterial3d<Wc3LayerMaterial>)>();
    let layers: Vec<_> = query
        .iter(app.world())
        .map(|(layer, handle)| (layer.root, handle.0.clone()))
        .collect();
    assert_eq!(layers.len(), 12);
    for (index, (_, handle)) in layers.iter().enumerate() {
        for (_, other) in &layers[index + 1..] {
            assert_ne!(handle, other);
        }
    }
    let colors: Vec<_> = layers
        .iter()
        .map(|(root, handle)| {
            (
                *root,
                app.world()
                    .resource::<Assets<Wc3LayerMaterial>>()
                    .get(handle)
                    .unwrap()
                    .base
                    .base_color
                    .to_linear(),
            )
        })
        .collect();
    let first_colors: Vec<_> = colors
        .iter()
        .filter(|(root, _)| *root == first)
        .map(|(_, color)| *color)
        .collect();
    let second_colors: Vec<_> = colors
        .iter()
        .filter(|(root, _)| *root == second)
        .map(|(_, color)| *color)
        .collect();
    assert!(first_colors.contains(&LinearRgba::new(1.0, 0.0, 0.0, 1.0)));
    assert!(first_colors.contains(&LinearRgba::new(0.0, 0.5, 0.5, 1.0)));
    assert!(first_colors.contains(&LinearRgba::WHITE));
    assert!(first_colors.contains(&LinearRgba::new(0.0, 0.5, 0.5, 0.5)));
    assert!(second_colors.contains(&LinearRgba::new(0.0, 0.0, 1.0, 1.0)));
}
