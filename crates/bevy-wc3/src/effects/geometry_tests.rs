use super::*;
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::{RenderLayers, VisibilityPlugin, VisibleEntities};
use bevy::mesh::MeshPlugin;
use bevy::render::view::NoIndirectDrawing;
use std::any::TypeId;

#[test]
fn shared_quad_and_extraction_preserve_asset_identity_and_world_center_without_mesh_uniforms() {
    let mut meshes = Assets::<Mesh>::default();
    let first = effect_quad(&mut meshes);
    let second = effect_quad(&mut meshes);
    assert_eq!(first.id(), second.id());
    assert_eq!(meshes.len(), 1);
    let transform =
        GlobalTransform::from(Transform::from_xyz(3., -4., 5.).with_scale(Vec3::new(-2., 3., 4.)));
    let extracted =
        EffectGeometry::extract_component((&EffectGeometry(first.clone()), &transform)).unwrap();
    assert_eq!(extracted.mesh, first.id());
    assert_eq!(extracted.world_center, Vec3::new(3., -4., 5.));
}

#[test]
fn native_visibility_tracks_layers_parent_hidden_state_and_camera_choices_without_mesh3d() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        VisibilityPlugin,
        MeshPlugin,
    ));
    app.init_asset::<Mesh>();
    app.add_plugins(EffectGeometryPlugin);
    let root = app
        .world_mut()
        .spawn((Transform::default(), Visibility::Inherited))
        .id();
    let effect = app
        .world_mut()
        .spawn((
            EffectGeometry(Handle::default()),
            RenderLayers::layer(1),
            ChildOf(root),
        ))
        .id();
    let first = app
        .world_mut()
        .spawn((Camera3d::default(), RenderLayers::layer(0)))
        .id();
    let second = app
        .world_mut()
        .spawn((Camera3d::default(), RenderLayers::layer(1)))
        .id();
    app.update();
    let contains = |app: &App, view: Entity| {
        app.world()
            .get::<VisibleEntities>(view)
            .unwrap()
            .get(TypeId::of::<EffectGeometry>())
            .contains(&effect)
    };
    assert!(!contains(&app, first));
    assert!(contains(&app, second));
    assert!(!app.world().entity(effect).contains::<Mesh3d>());
    assert!(app.world().entity(effect).contains::<Aabb>());
    assert!(!app.world().entity(first).contains::<NoIndirectDrawing>());
    app.world_mut().entity_mut(root).insert(Visibility::Hidden);
    app.update();
    assert!(!contains(&app, second));
    app.world_mut().entity_mut(root).insert(Visibility::Visible);
    app.world_mut()
        .entity_mut(effect)
        .insert(RenderLayers::layer(0));
    app.update();
    assert!(contains(&app, first));
    assert!(!contains(&app, second));
}

#[test]
fn capsule_mesh_is_shared_and_splits_core_from_caps_for_flat_normals() {
    let mut meshes = Assets::<Mesh>::default();
    let first = effect_capsule(&mut meshes);
    assert_eq!(first.id(), effect_capsule(&mut meshes).id());
    assert_ne!(first.id(), effect_quad(&mut meshes).id());
    assert_eq!(meshes.len(), 2);
    let mesh = meshes.get(&first).unwrap();
    assert_eq!(mesh.count_vertices(), 10);
    assert_eq!(
        mesh.indices().unwrap().iter().collect::<Vec<_>>(),
        CAPSULE_INDICES.map(usize::from)
    );
    let axis = Vec3::new(1., 4., 2.);
    let direction = (Vec3::ZERO - Vec3::new(3., -2., 8.)).normalize();
    let width = axis.cross(direction).normalize();
    let padding = direction.cross(width);
    let position = |corner: [f32; 3]| {
        if corner[2].abs() == 2. {
            (axis * 0.5 + padding) * corner[1]
        } else {
            axis * (corner[1] * 0.5) + width * corner[0]
        }
    };
    let positions = CAPSULE_CORNERS.map(position);
    let core = axis.cross(width).normalize();
    let cap = padding.cross(width).normalize();
    assert!(
        core.dot(cap) < 0.99,
        "oblique core and cap require different normals"
    );
    for (triangle, expected) in CAPSULE_INDICES
        .as_chunks::<3>()
        .0
        .iter()
        .zip([core, core, cap, cap])
    {
        let [a, b, c] = [
            positions[triangle[0] as usize],
            positions[triangle[1] as usize],
            positions[triangle[2] as usize],
        ];
        let actual = (b - a).cross(c - a).normalize();
        assert!(actual.dot(expected) > 1. - 1e-5);
    }
}
#[test]
fn capsule_selector_silhouette_has_four_corners_and_two_tips() {
    let axis = Vec3::Y * 4.;
    let width = -Vec3::X;
    let padding = Vec3::Y;
    let positions = CAPSULE_CORNERS.map(|c| {
        if c[2].abs() == 2. {
            (axis * 0.5 + padding) * c[1]
        } else {
            axis * (c[1] * 0.5) + width * c[0]
        }
    });
    assert_eq!(positions[0], Vec3::new(-1., 2., 0.));
    assert_eq!(positions[1], Vec3::new(-1., -2., 0.));
    assert_eq!(positions[2], Vec3::new(1., -2., 0.));
    assert_eq!(positions[3], Vec3::new(1., 2., 0.));
    assert_eq!(positions[6], Vec3::Y * 3.);
    assert_eq!(positions[9], -Vec3::Y * 3.);
    // Texture coordinates at the four core corners lie on the diagonal; tips
    // supply the other two atlas-local corners before authored UV operations.
    let uv = CAPSULE_CORNERS.map(|c| {
        if c[2].abs() == 2. {
            if c[1] > 0. {
                [1., 0.]
            } else {
                [0., 1.]
            }
        } else {
            [(c[0] + 1.) * 0.5; 2]
        }
    });
    assert_eq!(
        [uv[0], uv[1], uv[2], uv[3], uv[6], uv[9]],
        [[1., 1.], [1., 1.], [0., 0.], [0., 0.], [1., 0.], [0., 1.]]
    );
}
