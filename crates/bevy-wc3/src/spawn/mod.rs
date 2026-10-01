use bevy::camera::visibility::DynamicSkinnedMeshBounds;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;
use wc3::model::materials::LayerFilterMode;

use crate::animation::AnimatedLayer;
use crate::attachment::spawn_attachments;
use crate::material::Wc3LayerMaterial;
use crate::model::{ModelError, Wc3Model};
use crate::particle_emitter::ParticleState;
use crate::particle_emitter2::{Particle2State, ParticleInstances, ParticleTextureSlot};
use crate::ribbon_emitter::{RibbonInstances, RibbonLayer, RibbonState};
use crate::texture_bindings::Wc3TextureBindings;

mod prepare;
mod rig;

pub(crate) use prepare::prepare_resolved_model;
pub use prepare::{prepare_model, prepare_model_with_resources, PreparedModel};
pub use rig::Wc3NodeEntities;
use rig::{spawn_animation_root, spawn_rig};

/// Spawn an independently animated instance from prepared assets.
pub fn spawn_prepared_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
) -> Entity {
    spawn_prepared_model_with_bindings(
        commands,
        meshes,
        materials,
        prepared,
        Wc3TextureBindings::default(),
    )
}

/// Spawn a prepared model with consumer-defined texture choices.
pub fn spawn_prepared_model_with_bindings(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
    bindings: Wc3TextureBindings,
) -> Entity {
    let root = commands
        .spawn((Transform::default(), Visibility::default()))
        .id();
    spawn_prepared_into(commands, meshes, materials, prepared, root, bindings);
    root
}

pub(crate) fn spawn_prepared_into(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
    root: Entity,
    bindings: Wc3TextureBindings,
) {
    let bindings = bindings.with_defaults(prepared.textures.clone());
    commands
        .entity(root)
        .insert((bindings.clone(), prepared.models.clone()));
    // Preserve consumer visibility when spawning into an existing root.
    commands.entity(root).insert_if_new(Visibility::default());
    spawn_animation_root(commands, &prepared.model, root);
    let rig = spawn_rig(commands, &prepared.model, root, &prepared.inverse_bindposes);
    let node_entities = &rig.by_object_id;
    spawn_attachments(commands, prepared, root, node_entities);
    for (index, definition) in prepared.model.particle_emitters().into_iter().enumerate() {
        let Some(model) = prepared.model_resources().particle(index) else {
            continue;
        };
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        commands.spawn((
            ParticleState::new(root, node, definition, model),
            ChildOf(root),
        ));
    }
    for (emitter_id, definition) in prepared.model.particle_emitters2().into_iter().enumerate() {
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        let texture = bindings.particle(emitter_id);
        let mesh = meshes.add(Rectangle::new(2.0, 2.0));
        let render = ParticleInstances {
            records: Default::default(),
            live_indices: Default::default(),
            uniform: Default::default(),
            texture,
            filter: definition.filter_mode,
            priority_plane: definition.priority_plane,
            sort_far: definition.node.flags.sort_prims_far_z(),
        };
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                render,
                Particle2State::new(root, node, definition),
                ParticleTextureSlot(emitter_id),
                Transform::default(),
                NoFrustumCulling,
            ))
            .id();
        commands.entity(root).add_child(entity);
    }
    let source_materials = prepared.model.materials();
    let texture_animations = prepared.model.texture_animations();
    for definition in prepared.model.ribbon_emitters() {
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        let Some(material) = source_materials.get(definition.material_id as usize) else {
            warn!(
                "Ribbon {} references missing material {}",
                definition.node.object_id, definition.material_id
            );
            continue;
        };
        let emitter = commands
            .spawn((
                RibbonState::new(root, node, definition.clone()),
                ChildOf(root),
            ))
            .id();
        let mesh = meshes.add(Rectangle::new(2.0, 2.0));
        for (layer_index, layer) in material.layers.iter().enumerate() {
            let mut layer = layer.clone();
            // Prepared layers normalize Classic and HD diffuse texture bindings.
            layer.texture_id = prepared.layers[definition.material_id as usize][layer_index]
                .texture_id
                .clone();
            let texture = bindings.bitmap(layer.texture_id.value().copied().unwrap_or(0) as usize);
            let flags = layer.shading_flags;
            commands.spawn((
                Mesh3d(mesh.clone()),
                RibbonInstances {
                    records: Default::default(),
                    live_indices: Default::default(),
                    uniform: Default::default(),
                    texture,
                    filter: layer.filter_mode,
                    priority_plane: material.priority_plane,
                    emitter,
                    layer_index,
                    sort_far: material.render_mode.sort_primitives_far_z(),
                    sort_near: material.render_mode.sort_primitives_near_z(),
                    no_depth_test: flags.no_depth_test(),
                    no_depth_set: flags.no_depth_set(),
                    two_sided: flags.two_sided() || material.render_mode.two_sided(),
                },
                RibbonLayer {
                    emitter,
                    texture_animation: texture_animations
                        .get(layer.texture_animation_id as usize)
                        .cloned(),
                    definition: layer,
                },
                Transform::default(),
                NoFrustumCulling,
                ChildOf(root),
            ));
        }
    }
    let layer_handles: Vec<Vec<_>> = prepared
        .layers
        .iter()
        .map(|layers| {
            layers
                .iter()
                .map(|layer| {
                    let mut value = layer.material.clone();
                    let texture_id = layer.texture_id.value().copied().unwrap_or(0);
                    value.base.base_color_texture = bindings.bitmap(texture_id as usize);
                    materials.add(value)
                })
                .collect()
        })
        .collect();
    for geoset in &prepared.geosets {
        let geoset_alpha = prepared.geoset_alphas[geoset.geoset_id].clone();
        let initial_visibility = if geoset_alpha
            .as_ref()
            .and_then(|alpha| alpha.value())
            .is_some_and(|alpha| *alpha <= 0.0)
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        for (layer, material) in prepared.layers[geoset.material_id]
            .iter()
            .zip(&layer_handles[geoset.material_id])
        {
            let material = if geoset_alpha.is_some() {
                let mut material = materials
                    .get(material)
                    .cloned()
                    .unwrap_or_else(|| layer.material.clone());
                let alpha = layer.alpha.value().copied().unwrap_or(1.0)
                    * geoset_alpha
                        .as_ref()
                        .and_then(|alpha| alpha.value().copied())
                        .unwrap_or(1.0);
                material.base.base_color = Color::srgba(1.0, 1.0, 1.0, alpha);
                if geoset_alpha
                    .as_ref()
                    .and_then(|alpha| alpha.value().copied())
                    .is_some_and(|alpha| (0.0..1.0).contains(&alpha))
                    && matches!(
                        layer.material.extension.filter,
                        LayerFilterMode::None | LayerFilterMode::Transparent
                    )
                {
                    material.base.alpha_mode = AlphaMode::AlphaToCoverage;
                }
                materials.add(material)
            } else {
                material.clone()
            };
            let mut entity = commands.spawn((
                Mesh3d(geoset.mesh.clone()),
                MeshMaterial3d(material),
                initial_visibility,
            ));
            {
                entity.insert(AnimatedLayer {
                    root,
                    alpha: layer.alpha.clone(),
                    geoset_alpha: geoset_alpha.clone(),
                    texture_id: layer.texture_id.clone(),
                });
            }
            let entity = entity.id();
            if !rig.joints.is_empty() {
                commands.entity(entity).insert((
                    SkinnedMesh {
                        inverse_bindposes: rig.inverse_bindposes.clone(),
                        joints: rig.joints.clone(),
                    },
                    DynamicSkinnedMeshBounds,
                ));
            }
            // Joint matrices already contain world transforms; skinning ignores
            // this mesh transform. Parenting supplies visibility and ownership.
            commands.entity(root).add_child(entity);
        }
    }
}

/// Spawn an independently animated instance. The resolver supplies literal
/// bitmap paths; replaceable IDs can be bound on the returned root entity.
pub fn spawn_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<Entity, ModelError> {
    let prepared = prepare_model(
        meshes,
        materials,
        inverse_bindposes,
        source,
        resolve_texture,
    )?;
    Ok(spawn_prepared_model(commands, meshes, materials, &prepared))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{animate_layers, animate_nodes, Wc3Animation};
    use crate::instance::{Wc3ModelOwner, Wc3OwnedModels};
    use bevy::camera::visibility::VisibilityPlugin;
    use bevy::ecs::world::{CommandQueue, World};
    use wc3::model::animation::{Track, ValueKeyframe};
    use wc3::model::materials::Layer;
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
            Wc3Model::decode_mdl(include_str!("../../tests/fixtures/particle_capture.mdl"))
                .unwrap();
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
}
