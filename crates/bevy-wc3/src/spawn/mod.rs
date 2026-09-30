use bevy::camera::visibility::DynamicSkinnedMeshBounds;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;
use wc3::model::materials::LayerFilterMode;

use crate::animation::AnimatedLayer;
use crate::material::Wc3LayerMaterial;
use crate::model::{ModelError, Wc3Model};
use crate::particle2::Particle2State;
use crate::particle_render::ParticleInstances;
use crate::texture_bindings::{ParticleTextureSlot, Wc3TextureBindings};

mod prepare;
mod rig;

pub(crate) use prepare::prepare_resolved_model;
pub use prepare::{prepare_model, PreparedModel};
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
    let root = commands.spawn(Transform::default()).id();
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
    commands.entity(root).insert(bindings.clone());
    spawn_animation_root(commands, &prepared.model, root);
    let rig = spawn_rig(commands, &prepared.model, root, &prepared.inverse_bindposes);
    let node_entities = &rig.by_object_id;
    for (emitter_id, definition) in prepared.model.particle_emitters2().into_iter().enumerate() {
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        let texture = bindings.particle(emitter_id);
        let mesh = meshes.add(Rectangle::new(2.0, 2.0));
        let render = ParticleInstances {
            particles: Vec::new(),
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
            Visibility::Visible
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
            } else {
                commands.entity(root).add_child(entity);
            }
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
    use bevy::ecs::world::{CommandQueue, World};
    use wc3::model::animation::{Track, ValueKeyframe};
    use wc3::model::materials::Layer;
    use wc3::model::mdl::Read as _;
    use wc3::model::{Model, V1800};

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
