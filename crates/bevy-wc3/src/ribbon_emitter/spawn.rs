use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use std::collections::HashMap;

use super::{RibbonInstances, RibbonLayer, RibbonState};
use crate::spawn::PreparedModel;
use crate::texture_bindings::Wc3TextureBindings;

pub(crate) fn spawn_ribbons(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    prepared: &PreparedModel,
    root: Entity,
    node_entities: &HashMap<u32, Entity>,
    bindings: &Wc3TextureBindings,
) {
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
            layer.texture_id = prepared
                .layer_texture_id(definition.material_id as usize, layer_index)
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
}
