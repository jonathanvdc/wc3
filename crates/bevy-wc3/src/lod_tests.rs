use super::*;
use crate::animation::Wc3Animation;
use crate::assets::Wc3Model;
use crate::instance::spawn::spawn_prepared_model;
use crate::materials::layers::{animate_layers, AnimatedLayer};
use crate::materials::Wc3LayerMaterial;
use crate::preparation::prepare_model;
use crate::schedule::{configure, Wc3Systems};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::visibility::{VisibilityPlugin, VisibilitySystems};
use bevy::camera::{CameraUpdateSystems, Viewport};
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use std::f32::consts::FRAC_PI_2;

#[test]
fn sparse_levels_caps_and_missing_requests_resolve_without_holes() {
    let mut settings = Wc3LodSettings::medium();
    let levels = [0, 2, 5];
    for (requested, expected) in [(0, 0), (1, 2), (2, 2), (3, 5), (99, 5)] {
        assert_eq!(
            select_level(&levels, Wc3Lod::Fixed(requested), &settings, None, None),
            expected
        );
    }
    assert_eq!(
        select_level(&levels, Wc3Lod::Automatic, &settings, Some(200.0), None),
        2
    );
    assert_eq!(
        select_level(&levels, Wc3Lod::Automatic, &settings, Some(50.0), None),
        5
    );
    settings.minimum_level = 1;
    assert_eq!(
        select_level(&levels, Wc3Lod::Fixed(0), &settings, None, None),
        2
    );
    assert_eq!(
        select_level(&levels, Wc3Lod::Automatic, &settings, None, Some(5)),
        2
    );
    settings.minimum_level = 99;
    assert_eq!(
        select_level(&levels, Wc3Lod::Automatic, &settings, Some(1000.0), None),
        5
    );
}

#[test]
fn quality_bias_hysteresis_and_extended_thresholds() {
    let levels = [0, 1, 2, 3, 4];
    let settings = Wc3LodSettings::medium();
    let select =
        |size, previous| select_level(&levels, Wc3Lod::Automatic, &settings, Some(size), previous);
    assert_eq!(select(230.0, Some(0)), 0);
    assert_eq!(select(200.0, Some(0)), 1);
    assert_eq!(select(250.0, Some(1)), 1);
    assert_eq!(select(280.0, Some(1)), 0);
    assert_eq!(select(10.0, Some(0)), 4);
    assert_eq!(select(1000.0, Some(4)), 0);
    assert_eq!(
        select_level(
            &levels,
            Wc3Lod::Automatic,
            &Wc3LodSettings::high(),
            Some(200.0),
            None
        ),
        0
    );
    assert_eq!(
        select_level(
            &levels,
            Wc3Lod::Automatic,
            &Wc3LodSettings::low(),
            Some(200.0),
            None
        ),
        2
    );
    let empty = Wc3LodSettings {
        thresholds: vec![],
        ..settings
    };
    assert_eq!(
        select_level(&levels, Wc3Lod::Automatic, &empty, Some(0.0), Some(4)),
        0
    );
}

#[test]
fn settings_reject_invalid_values() {
    for value in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
        assert!(Wc3LodSettings {
            quality_bias: value,
            ..default()
        }
        .validate()
        .is_err());
    }
    for value in [f32::NAN, 1.0, -0.1] {
        assert!(Wc3LodSettings {
            hysteresis: value,
            ..default()
        }
        .validate()
        .is_err());
    }
    for thresholds in [
        vec![100.0, 200.0],
        vec![100.0, 100.0],
        vec![0.0],
        vec![f32::NAN],
    ] {
        assert!(Wc3LodSettings {
            thresholds,
            ..default()
        }
        .validate()
        .is_err());
    }
}

#[test]
fn projection_handles_zoom_viewport_scale_and_near_plane() {
    let bounds = LodBounds {
        center: Vec3::new(0.0, 0.0, -10.0),
        radius: 1.0,
    };
    let camera = GlobalTransform::IDENTITY;
    let perspective = Projection::Perspective(PerspectiveProjection {
        fov: FRAC_PI_2,
        ..default()
    });
    let pixels = projected_diameter(bounds, &camera, &perspective, 600.0).unwrap();
    assert!((pixels - 600.0 / 9.0).abs() < 0.001);
    assert_eq!(
        projected_diameter(bounds, &camera, &perspective, 1200.0),
        Some(pixels * 2.0)
    );
    let scaled_camera = GlobalTransform::from(Transform::from_scale(Vec3::splat(2.0)));
    assert!(
        (projected_diameter(bounds, &scaled_camera, &perspective, 600.0).unwrap() - pixels).abs()
            < 0.001
    );
    let zoom = Projection::Perspective(PerspectiveProjection {
        fov: 0.5,
        ..default()
    });
    assert!(projected_diameter(bounds, &camera, &zoom, 600.0).unwrap() > pixels);
    let mut ortho = OrthographicProjection::default_3d();
    ortho.area = Rect::new(-10.0, -5.0, 10.0, 5.0);
    let ortho = Projection::Orthographic(ortho);
    assert_eq!(
        projected_diameter(bounds, &camera, &ortho, 600.0),
        Some(120.0)
    );
    let scaled = bounds.world(&GlobalTransform::from(Transform::from_scale(Vec3::new(
        2.0, 3.0, 1.0,
    ))));
    assert_eq!(scaled.radius, 3.0);
    assert!(projected_diameter(scaled, &camera, &ortho, 600.0).unwrap() > 120.0);
    let near = LodBounds {
        center: Vec3::new(0.0, 0.0, -0.5),
        ..bounds
    };
    assert_eq!(
        projected_diameter(near, &camera, &perspective, 600.0),
        Some(f32::INFINITY)
    );
    let behind = LodBounds {
        center: Vec3::new(0.0, 0.0, 10.0),
        ..bounds
    };
    assert_eq!(
        projected_diameter(behind, &camera, &perspective, 600.0),
        None
    );
}

fn scene() -> (App, Entity, Entity) {
    let source = Wc3Model::decode_mdl(include_str!("../tests/fixtures/lod_capture.mdl")).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, VisibilityPlugin));
    configure(&mut app);
    app.init_resource::<Wc3LodSettings>();
    app.add_systems(Update, animate_layers);
    app.add_systems(PostUpdate, update_lod.in_set(Wc3Systems::SelectLod));
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
    assert_eq!(prepared.lod_levels(), [0, 2]);
    assert_eq!(
        prepared
            .geosets
            .iter()
            .map(|g| g.geoset_id)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let first = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    let second = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    queue.apply(app.world_mut());
    app.insert_resource(meshes);
    app.insert_resource(materials);
    app.insert_resource(bindposes);
    (app, first, second)
}

fn visible_groups(app: &mut App, root: Entity) -> Vec<Option<u32>> {
    let mut query = app.world_mut().query::<(&LodGroup, &InheritedVisibility)>();
    let mut levels: Vec<_> = query
        .iter(app.world())
        .filter(|(group, visible)| group.root == root && visible.get())
        .map(|(group, _)| group.level)
        .collect();
    levels.sort();
    levels
}

#[test]
fn lod_groups_preserve_alpha_rigs_independent_instances_and_root_visibility() {
    let (mut app, first, second) = scene();
    app.world_mut().entity_mut(first).insert(Wc3Lod::Fixed(2));
    app.update();
    assert_eq!(visible_groups(&mut app, first), [None, Some(2)]);
    assert_eq!(visible_groups(&mut app, second), [None, Some(0)]);
    let mut query = app.world_mut().query::<(&AnimatedLayer, &SkinnedMesh)>();
    let skins: Vec<_> = query
        .iter(app.world())
        .filter(|(layer, _)| layer.root == first)
        .map(|(_, skin)| skin.joints.clone())
        .collect();
    assert_eq!(skins.len(), 3);
    assert!(skins.iter().all(|joints| joints == &skins[0]));
    app.world_mut()
        .get_mut::<Wc3Animation>(first)
        .unwrap()
        .elapsed_ms = 1000.0;
    app.update();
    let mut query = app
        .world_mut()
        .query::<(&AnimatedLayer, &ChildOf, &Visibility, &InheritedVisibility)>();
    for (layer, parent, visibility, inherited) in query
        .iter(app.world())
        .filter(|(layer, _, _, _)| layer.root == first)
    {
        let group = app.world().get::<LodGroup>(parent.parent()).unwrap();
        if group.level == Some(2) {
            assert_eq!(*visibility, Visibility::Hidden);
            assert!(!inherited.get());
        }
        if group.level == Some(0) {
            assert_eq!(*visibility, Visibility::Inherited);
            assert!(!inherited.get());
        }
        let _ = layer;
    }
    *app.world_mut().get_mut::<Visibility>(first).unwrap() = Visibility::Hidden;
    app.update();
    assert!(visible_groups(&mut app, first).is_empty());
    assert_eq!(visible_groups(&mut app, second), [None, Some(0)]);
    *app.world_mut().get_mut::<Visibility>(first).unwrap() = Visibility::Inherited;
    app.world_mut().entity_mut(first).insert(Wc3Lod::Fixed(0));
    app.update();
    assert_eq!(visible_groups(&mut app, first), [None, Some(0)]);
    let mut groups = app.world_mut().query::<(Entity, &LodGroup)>();
    let owned: Vec<_> = groups
        .iter(app.world())
        .filter(|(_, g)| g.root == first)
        .map(|(e, _)| e)
        .collect();
    app.world_mut().entity_mut(first).despawn();
    assert!(owned
        .iter()
        .all(|&entity| app.world().get_entity(entity).is_err()));
}

fn camera(app: &mut App, distance: f32) -> Entity {
    app.world_mut()
        .spawn((
            Camera3d::default(),
            Camera {
                viewport: Some(Viewport {
                    physical_size: UVec2::new(640, 480),
                    ..default()
                }),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, distance),
            Projection::Perspective(PerspectiveProjection::default()),
        ))
        .id()
}

#[test]
fn automatic_uses_current_transforms_overrides_and_multiview_fallback() {
    let (mut app, first, second) = scene();
    app.insert_resource(Wc3LodSettings::medium());
    let far = camera(&mut app, 100.0);
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        2
    );
    app.world_mut()
        .get_mut::<Transform>(far)
        .unwrap()
        .translation
        .z = 5.0;
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        0
    );
    // Ambiguous active views choose the finest requirement.
    app.world_mut()
        .get_mut::<Transform>(far)
        .unwrap()
        .translation
        .z = 100.0;
    let near = camera(&mut app, 5.0);
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        0
    );
    app.world_mut().entity_mut(first).insert(Wc3NodeCamera(far));
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        2
    );
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(second)
            .unwrap()
            .selected_level(),
        0
    );
    // Settings are a full local override; explicit policy wins over their default.
    app.world_mut().entity_mut(second).insert((
        Wc3Lod::Automatic,
        Wc3LodOverride(Wc3LodSettings {
            minimum_level: 2,
            ..Wc3LodSettings::high()
        }),
    ));
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(second)
            .unwrap()
            .selected_level(),
        2
    );
    app.world_mut().entity_mut(first).remove::<Wc3NodeCamera>();
    app.world_mut().entity_mut(far).insert(Wc3DefaultNodeCamera);
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        2
    );
    app.world_mut().get_mut::<Camera>(far).unwrap().is_active = false;
    app.world_mut().get_mut::<Camera>(near).unwrap().is_active = false;
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        0
    );
}

#[test]
fn lod_selection_precedes_visibility_and_follows_projection_updates() {
    #[derive(Resource, Default)]
    struct Order(Vec<&'static str>);
    let mut app = App::new();
    configure(&mut app);
    app.init_resource::<Order>();
    app.add_systems(
        PostUpdate,
        (
            (|mut order: ResMut<Order>| order.0.push("camera")).in_set(CameraUpdateSystems),
            (|mut order: ResMut<Order>| {
                assert!(order.0.contains(&"camera"));
                order.0.push("lod");
            })
            .in_set(Wc3Systems::SelectLod),
            (|order: Res<Order>| assert!(order.0.contains(&"lod")))
                .in_set(VisibilitySystems::VisibilityPropagate),
        ),
    );
    app.update();
}

#[test]
fn common_only_and_classic_models_have_a_safe_single_level() {
    let mut source =
        Wc3Model::decode_mdl(include_str!("../tests/fixtures/lod_capture.mdl")).unwrap();
    let geosets = source.model.geosets();
    source.model.set_geosets(&geosets[2..]);
    let mut meshes = Assets::<Mesh>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
    assert_eq!(prepared.lod_levels(), [0]);
    assert_eq!(prepared.geosets[0].lod, None);
    let classic = Wc3Model::decode(include_bytes!("../tests/fixtures/quad_model.mdx")).unwrap();
    let prepared = prepare_model(&mut meshes, &mut bindposes, &classic, |_| None).unwrap();
    assert_eq!(prepared.lod_levels(), [0]);
    source.model.set_geosets(&[]);
    let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
    assert_eq!(prepared.lod_levels(), [0]);
    assert!(prepared.geosets.is_empty());
}

#[test]
fn inherited_camera_settings_fallback_and_render_layers() {
    let (mut app, first, _) = scene();
    app.insert_resource(Wc3LodSettings::medium());
    let far = camera(&mut app, 100.0);
    let near = camera(&mut app, 5.0);
    app.world_mut()
        .entity_mut(near)
        .insert(RenderLayers::layer(1));
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        2
    );
    app.world_mut().entity_mut(near).remove::<RenderLayers>();
    let parent = app
        .world_mut()
        .spawn((Transform::default(), Wc3NodeCamera(far)))
        .id();
    app.world_mut().entity_mut(first).insert(ChildOf(parent));
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        2
    );
    // Invalid local settings use built-in settings; an explicit policy survives.
    app.world_mut().entity_mut(first).insert((
        Wc3Lod::Fixed(2),
        Wc3LodOverride(Wc3LodSettings {
            quality_bias: f32::NAN,
            ..default()
        }),
    ));
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        2
    );
    app.world_mut().entity_mut(first).remove::<Wc3Lod>();
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3LodState>(first)
            .unwrap()
            .selected_level(),
        0
    );
}
