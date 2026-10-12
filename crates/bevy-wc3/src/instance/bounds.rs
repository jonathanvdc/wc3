//! Skin bounds are refreshed only when their contributing transforms change.
use bevy::camera::primitives::Aabb;
use bevy::mesh::skinning::{
    entity_aabb_from_skinned_mesh_bounds, SkinnedMesh, SkinnedMeshInverseBindposes,
};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct SkinBounds {
    dirty: bool,
}
impl Default for SkinBounds {
    fn default() -> Self {
        Self { dirty: true }
    }
}

type SkinnedParts<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut SkinBounds,
        &'static mut Aabb,
        Ref<'static, Mesh3d>,
        Ref<'static, SkinnedMesh>,
        Ref<'static, GlobalTransform>,
        &'static InheritedVisibility,
    ),
>;

pub(crate) fn update_skin_bounds(
    meshes: Res<Assets<Mesh>>,
    bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    joints: Query<&GlobalTransform>,
    changed_joints: Query<Ref<GlobalTransform>>,
    mut parts: SkinnedParts,
) {
    for (mut bounds, mut aabb, mesh, skin, transform, visibility) in &mut parts {
        let dirty = bounds.dirty
            || meshes.is_changed()
            || bindposes.is_changed()
            || mesh.is_changed()
            || skin.is_changed()
            || transform.is_changed()
            || skin.joints.iter().any(|joint| {
                changed_joints
                    .get(*joint)
                    .is_ok_and(|joint| joint.is_changed())
            });
        if !dirty {
            continue;
        }
        if !visibility.get() {
            if !bounds.dirty {
                bounds.dirty = true;
            }
            continue;
        }
        let Some(mesh) = meshes.get(&mesh.0) else {
            continue;
        };
        let Some(bindposes) = bindposes.get(&skin.inverse_bindposes) else {
            continue;
        };
        if let Ok(next) =
            entity_aabb_from_skinned_mesh_bounds(&joints, mesh, &skin, bindposes, Some(&transform))
        {
            let next: Aabb = next.into();
            if aabb.center != next.center || aabb.half_extents != next.half_extents {
                *aabb = next;
            }
            if bounds.dirty {
                bounds.dirty = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::Wc3Model;
    use crate::instance::spawn::spawn_prepared_model;
    use crate::preparation::prepare_model;
    use bevy::camera::visibility::{VisibilityPlugin, VisibilitySystems};
    use bevy::ecs::world::CommandQueue;
    use bevy::math::Vec3A;
    use bevy::transform::TransformSystems;

    #[test]
    fn unchanged_rigs_keep_bounds_and_hidden_motion_refreshes_on_reveal() {
        let source =
            Wc3Model::decode(include_bytes!("../../tests/fixtures/quad_model.mdx")).unwrap();
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, VisibilityPlugin));
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::default();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, app.world());
        let root = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
        queue.apply(app.world_mut());
        app.insert_resource(meshes).insert_resource(bindposes);
        app.add_systems(
            PostUpdate,
            update_skin_bounds
                .after(TransformSystems::Propagate)
                .after(VisibilitySystems::CalculateBounds)
                .after(VisibilitySystems::VisibilityPropagate)
                .before(VisibilitySystems::CheckVisibility),
        );
        app.update();
        let part = app
            .world_mut()
            .query_filtered::<Entity, With<SkinBounds>>()
            .single(app.world())
            .unwrap();
        let initial = *app.world().get::<Aabb>(part).unwrap();
        assert!(!app.world().get::<SkinBounds>(part).unwrap().dirty);
        app.update();
        let current = *app.world().get::<Aabb>(part).unwrap();
        assert_eq!(initial.center, current.center);
        assert_eq!(initial.half_extents, current.half_extents);
        *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Hidden;
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation
            .x = 30.;
        app.update();
        assert!(app.world().get::<SkinBounds>(part).unwrap().dirty);
        app.world_mut().get_mut::<Transform>(root).unwrap().scale = Vec3::splat(2.);
        let joint = app.world().get::<SkinnedMesh>(part).unwrap().joints[0];
        app.world_mut()
            .get_mut::<Transform>(joint)
            .unwrap()
            .translation
            .y += 4.;
        app.update();
        *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Inherited;
        app.update();
        assert!(!app.world().get::<SkinBounds>(part).unwrap().dirty);
        let bounds = app.world().get::<Aabb>(part).unwrap();
        assert!(bounds.half_extents.abs_diff_eq(initial.half_extents, 1e-5));
        // Bounds are mesh-local; moved/scaled joint and mesh world transforms cancel.
        assert!(bounds
            .center
            .abs_diff_eq(initial.center + Vec3A::Y * 4., 1e-5));
    }
}
