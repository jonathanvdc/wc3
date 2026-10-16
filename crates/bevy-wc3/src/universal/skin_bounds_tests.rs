use super::*;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::{VisibilityPlugin, VisibilitySystems};
use bevy::mesh::PrimitiveTopology;
use bevy::transform::TransformSystems;

fn mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0., 0., 0.], [2., 2., 2.], [1., 1., 1.]],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[0; 4]; 3]),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[0.5, 0., 0., 0.]; 3]);
    mesh.insert_attribute(
        EXTRA_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[1; 4]; 3]),
    );
    mesh.insert_attribute(EXTRA_JOINT_WEIGHT, vec![[0.5, 0., 0., 0.]; 3]);
    compile_skin_bounds(&mut mesh).unwrap();
    mesh
}
#[test]
fn eight_influences_include_only_positive_weight_contributors() {
    let mesh = mesh();
    let bounds = mesh.skinned_mesh_bounds().unwrap();
    assert_eq!(
        bounds.aabb_index_to_joint_index,
        vec![JointIndex(0), JointIndex(1)]
    );
    assert_eq!(bounds.aabbs.len(), 2);
    assert_eq!(bounds.aabbs[1].center, Vec3::ONE);
}
#[test]
fn static_rigs_ignore_unrelated_assets_and_hidden_motion_updates_on_reveal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, VisibilityPlugin));
    app.add_message::<AssetEvent<Mesh>>();
    app.add_message::<AssetEvent<SkinnedMeshInverseBindposes>>();
    let mut meshes = Assets::<Mesh>::default();
    let source = meshes.add(mesh());
    let unrelated = meshes.add(mesh());
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let bind = binds.add(SkinnedMeshInverseBindposes::from(vec![Mat4::IDENTITY; 2]));
    let joint0 = app.world_mut().spawn(Transform::default()).id();
    let joint1 = app.world_mut().spawn(Transform::default()).id();
    let part = app
        .world_mut()
        .spawn((
            Mesh3d(source),
            Transform::default(),
            Visibility::Inherited,
            Aabb::from_min_max(Vec3::ZERO, Vec3::splat(2.)),
            SkinBounds::default(),
            SkinnedMesh {
                inverse_bindposes: bind,
                joints: vec![joint0, joint1],
            },
        ))
        .id();
    app.insert_resource(meshes).insert_resource(binds);
    app.add_systems(
        PostUpdate,
        update_skin_bounds
            .after(TransformSystems::Propagate)
            .after(VisibilitySystems::CalculateBounds)
            .after(VisibilitySystems::VisibilityPropagate)
            .before(VisibilitySystems::CheckVisibility),
    );
    app.update();
    assert_eq!(app.world().get::<SkinBounds>(part).unwrap().updates, 1);
    app.update();
    assert_eq!(app.world().get::<SkinBounds>(part).unwrap().updates, 1);
    app.world_mut()
        .write_message(AssetEvent::<Mesh>::Modified { id: unrelated.id() });
    app.update();
    assert_eq!(app.world().get::<SkinBounds>(part).unwrap().updates, 1);
    *app.world_mut().get_mut::<Visibility>(part).unwrap() = Visibility::Hidden;
    app.world_mut()
        .get_mut::<Transform>(joint1)
        .unwrap()
        .translation
        .x = 10.;
    app.update();
    assert!(app.world().get::<SkinBounds>(part).unwrap().dirty);
    assert_eq!(app.world().get::<SkinBounds>(part).unwrap().updates, 1);
    *app.world_mut().get_mut::<Visibility>(part).unwrap() = Visibility::Inherited;
    app.update();
    assert!(!app.world().get::<SkinBounds>(part).unwrap().dirty);
    assert_eq!(app.world().get::<SkinBounds>(part).unwrap().updates, 2);
    let aabb = app.world().get::<Aabb>(part).unwrap();
    assert!((aabb.max().x - 12.).abs() < 1e-5);
    let handle = app.world().get::<Mesh3d>(part).unwrap().0.clone();
    app.world_mut()
        .resource_mut::<Assets<Mesh>>()
        .get_mut(&handle)
        .unwrap()
        .insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0., 0., 0.], [20., 2., 2.], [1., 1., 1.]],
        );
    app.world_mut()
        .write_message(AssetEvent::<Mesh>::Modified { id: handle.id() });
    app.update();
    assert_eq!(app.world().get::<SkinBounds>(part).unwrap().updates, 3);
    assert!((app.world().get::<Aabb>(part).unwrap().max().x - 30.).abs() < 1e-5);
}
