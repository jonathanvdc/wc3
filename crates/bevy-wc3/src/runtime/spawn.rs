use super::super::events::{EventDispatchStats, EventState};
use super::super::lod::LodState;
use super::super::mesh_animation::MeshMaterialState;
use super::super::pose::{
    as_transform, evaluate_pose, hierarchical, local_transform, EvaluatedPose,
};
use super::super::skin_bounds::SkinBounds;
use super::super::transition::TransitionState;
use super::super::{ModelMaterial, PreparedModel, WorldLiquidSurface};
use super::current_affine::NodePoseSource;
use super::evaluation::EvaluationState;
use super::model_driver::ModelParticleDriver;
use super::mounts::MountedModel;
use super::{ModelDraw, ModelNode, ModelRig, Playback, PlaybackError, PoseOverrides};
use crate::bounds::transform_bounds;
use crate::effects::model_children::ParticleModelSubtree;
use crate::effects::model_ownership::queue_model_birth;
use crate::effects::RuntimeParticles;
use crate::layers::ModelOwnedRenderEntity;
use crate::physics::PhysicsState;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Spawn a directly compiled semantic model beneath an application-owned root.
///
/// Register [`crate::Wc3BevyPlugin`] to advance playback. The root owns its rig and
/// draws through Bevy hierarchy lifetime; meshes and inverse binds are shared.
pub fn spawn_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ModelMaterial>,
    model: &PreparedModel,
    transform: Transform,
) -> Entity {
    let root = commands
        .spawn((Name::new("Model"), transform, Visibility::default()))
        .id();
    populate_model(commands, meshes, materials, model, root);
    root
}

pub(crate) fn populate_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ModelMaterial>,
    model: &PreparedModel,
    root: Entity,
) {
    populate_model_with_context(
        commands,
        meshes,
        materials,
        model,
        root,
        false,
        Playback::default(),
    );
}

pub(crate) fn populate_model_with_context(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ModelMaterial>,
    model: &PreparedModel,
    root: Entity,
    owned: bool,
    initial_playback: Playback,
) {
    if owned {
        commands.entity(root).insert(ParticleModelSubtree);
    }
    if model.particles.has_model_actors() {
        commands.entity(root).insert(ModelParticleDriver::default());
    } else {
        commands.entity(root).remove::<ModelParticleDriver>();
    }
    let basis_entity = commands
        .spawn((
            Name::new("Model coordinate frame"),
            model.basis,
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    let mut particles = model
        .particles
        .spawn(commands, meshes, materials, basis_entity);
    if owned {
        if let Ok(selection) = initial_playback.selection() {
            RuntimeParticles::start_owned_clock(&mut particles, selection);
        }
    }
    let mut nodes = Vec::with_capacity(model.nodes.len());
    let mut poses = vec![Mat4::IDENTITY; model.nodes.len()];
    let mut evaluated = vec![EvaluatedPose::default(); model.nodes.len()];
    let mut visible = vec![true; model.nodes.len()];
    for &index in model.order.iter() {
        let node = &model.nodes[index];
        evaluated[index] = evaluate_pose(
            node,
            node.rest,
            node.parent
                .map_or_else(EvaluatedPose::default, |parent| evaluated[parent]),
            None,
        );
        poses[index] = evaluated[index].matrix;
        visible[index] = node.visible && node.parent.is_none_or(|parent| visible[parent]);
    }
    for (index, node) in model.nodes.iter().enumerate() {
        nodes.push(
            commands
                .spawn((
                    Name::new(node.name.clone()),
                    ModelNode { id: node.id },
                    NodePoseSource { rig: root, index },
                    if hierarchical(node) {
                        local_transform(node.rest)
                    } else {
                        as_transform(evaluated[index])
                    },
                    if visible[index] {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    ChildOf(basis_entity),
                ))
                .id(),
        );
    }
    for (index, node) in model.nodes.iter().enumerate() {
        if hierarchical(node) {
            if let Some(parent) = node.parent {
                commands.entity(nodes[index]).insert(ChildOf(nodes[parent]));
            }
        }
    }
    let feature_nodes = model
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, nodes[i]))
        .collect();
    let world_parents = model.world.spawn_gates(commands, root, &feature_nodes);
    let feature_instances =
        model
            .features
            .spawn(commands, &feature_nodes, &world_parents.features, owned);
    let lod = model
        .draws
        .iter()
        .filter_map(|draw| draw.lod)
        .chain(model.world.minimum_lod())
        .min()
        .unwrap_or(0);
    let mut material_cache = HashMap::new();
    let mut animated_materials = Vec::new();
    let mut draw_entities = Vec::new();
    for draw in model.draws.iter() {
        let material = material_cache
            .entry((
                draw.material_id,
                draw.pass,
                draw.mesh_state,
                draw.world_state,
            ))
            .or_insert_with(|| {
                if draw.animation.is_animated() {
                    let handle = materials.add(draw.material.clone());
                    animated_materials.push((handle.clone(), draw.animation.clone()));
                    handle
                } else {
                    model
                        .material_cache
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .entry((
                            draw.material_id,
                            draw.pass,
                            draw.mesh_state,
                            draw.world_state,
                        ))
                        .or_insert_with(|| materials.add(draw.material.clone()))
                        .clone()
                }
            })
            .clone();
        let mut entity = commands.spawn((
            ModelDraw {
                instance: draw.instance,
                source: draw.source,
                mesh: draw.mesh_id,
                material: draw.material_id,
                pass: draw.pass,
                lod: draw.lod,
            },
            ModelOwnedRenderEntity,
            Mesh3d(draw.mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            if draw.lod.is_none_or(|level| level == lod) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
            ChildOf(
                world_parents
                    .placements
                    .get(&draw.instance)
                    .copied()
                    .unwrap_or(nodes[draw.node]),
            ),
        ));
        if !draw.material.extension.casts_shadows() {
            entity.insert(NotShadowCaster);
        }
        if !draw.material.extension.receives_shadows() {
            entity.insert(NotShadowReceiver);
        }
        if let Some(skin) = draw.skin {
            let skin = &model.skins[skin];
            entity.insert((
                SkinnedMesh {
                    inverse_bindposes: skin.binds.clone(),
                    joints: skin.joints.iter().map(|&i| nodes[i]).collect(),
                },
                SkinBounds::default(),
            ));
        }
        draw_entities.push((entity.id(), draw.lod, material));
    }
    for liquid in model.liquids.iter() {
        let mut entity = commands.spawn((
            WorldLiquidSurface {
                instance: liquid.instance,
                ordinal: liquid.ordinal,
                liquid_type: liquid.liquid_type,
                material: liquid.material_id,
            },
            ModelOwnedRenderEntity,
            Mesh3d(liquid.mesh.clone()),
            Transform::default(),
            Visibility::Inherited,
            ChildOf(
                world_parents
                    .placements
                    .get(&liquid.instance)
                    .copied()
                    .unwrap_or(nodes[liquid.node]),
            ),
        ));
        if let Some((surface, animation)) = &liquid.surface {
            if !surface.extension.casts_shadows() {
                entity.insert(NotShadowCaster);
            }
            if !surface.extension.receives_shadows() {
                entity.insert(NotShadowReceiver);
            }
            let id = liquid
                .material_id
                .expect("compiled liquid surface has material identity");
            let key = (id, liquid.pass, MeshMaterialState::Plain, [0; 4]);
            let material = material_cache
                .entry(key)
                .or_insert_with(|| {
                    if animation.is_animated() {
                        let handle = materials.add(surface.clone());
                        animated_materials.push((handle.clone(), animation.clone()));
                        handle
                    } else {
                        model
                            .material_cache
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .entry(key)
                            .or_insert_with(|| materials.add(surface.clone()))
                            .clone()
                    }
                })
                .clone();
            entity.insert(MeshMaterial3d(material.clone()));
            draw_entities.push((entity.id(), None, material));
        }
    }
    let mut mounted_roots = Vec::with_capacity(model.mounts.len());
    for mount in model.mounts.iter() {
        let mut child_model = mount.model.clone();
        // The mount transform is already authored in the host model frame.
        // Bounds compiled in application coordinates must return to the child
        // authored frame along with the removed basis conversion.
        let inverse_basis = child_model.basis.to_matrix().inverse();
        child_model.rest_bounds = child_model
            .rest_bounds
            .map(|bounds| transform_bounds(bounds, inverse_basis));
        child_model.basis = Transform::IDENTITY;
        let child = if owned {
            let child = commands
                .spawn((Name::new("Model"), mount.transform, Visibility::default()))
                .id();
            let mut playback = child_model
                .clips
                .first()
                .map_or_else(Playback::default, |(clip, _)| Playback::for_clip(*clip));
            playback.speed = initial_playback.speed;
            playback.paused = initial_playback.paused;
            playback.driving_camera = initial_playback.driving_camera;
            if let Some(clip) = initial_playback
                .clip
                .and_then(|clip| mount.clips.get(&clip).copied())
            {
                playback = initial_playback.clone();
                playback.clip = Some(clip);
            }
            populate_model_with_context(
                commands,
                meshes,
                materials,
                &child_model,
                child,
                true,
                playback,
            );
            queue_model_birth(commands, child);
            child
        } else {
            spawn_model(commands, meshes, materials, &child_model, mount.transform)
        };
        mounted_roots.push(child);
        let parent = mount
            .socket
            .and_then(|point| model.features.attachment_entity(point, &feature_instances))
            .unwrap_or(nodes[mount.parent]);
        commands.entity(child).insert((
            ChildOf(parent),
            MountedModel {
                parent: root,
                clips: Arc::new(mount.clips.clone()),
                inherited_physics: None,
                inherited_from: None,
            },
        ));
        if !owned {
            if let Some((clip, _)) = child_model.clips.first() {
                commands.entity(child).insert(Playback::for_clip(*clip));
            }
        }
    }
    commands.entity(root).insert((
        ModelRig {
            events: EventState::default(),
            event_scratch: Vec::new(),
            model: model.clone(),
            nodes,
            mounts: mounted_roots,
            poses,
            evaluated,
            visible,
            basis_entity,
            draws: draw_entities,
            materials: animated_materials,
            features: feature_instances,
            particles,
            pose_initialized: false,
            transition: TransitionState::default(),
            evaluation: EvaluationState::default(),
            physics: PhysicsState::default(),
            physics_locals: Vec::new(),
        },
        Playback {
            lod,
            ..initial_playback
        },
        PlaybackError::default(),
        EventDispatchStats::default(),
        LodState::default(),
    ));
    commands
        .entity(root)
        .insert_if_new(PoseOverrides::default());
}
