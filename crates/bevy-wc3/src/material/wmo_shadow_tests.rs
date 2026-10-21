//! Complete native shadow-pass fixture, compilation/baking and Bevy visibility.
use super::{
    lighting_quad, records, semantic_wmo, Array, Bundle, Classic, GroupChunk, GroupKey,
    ImportOptions, RootChunk,
};
use crate::{
    compile_model, spawn_model, BakedSource, DrawPurpose, ModelDraw, ModelMaterial, PoseBaker,
    PoseOptions, Wc3BevyPlugin,
};
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::primitives::{CascadesFrusta, Frustum};
use bevy::camera::visibility::{
    CascadesVisibleEntities, NoFrustumCulling, RenderLayers, VisibilityClass, VisibilityPlugin,
    VisibilitySystems, VisibleEntities,
};
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::light::check_dir_light_mesh_visibility;
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, MeshPlugin};
use bevy::prelude::*;
use bevy::shader::Shader;
use std::any::TypeId;
use wc3::model::profiles::Universal;

fn shadow_fixture() -> Bundle<'static, Classic> {
    let mut bundle = lighting_quad(true);
    bundle.root.header_mut().bounds[1][0] = 4.;
    for chunk in bundle.root.chunks_mut() {
        if let RootChunk::Groups(rows) = chunk {
            let mut group = rows.get(0).unwrap().unwrap();
            group.bounds[1][0] = 4.;
            *rows = Array::from_records(&[group]).unwrap();
        }
    }
    let group = bundle.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
    // The caster has its own triangle, outside the ordinary quad. Excluding the
    // camera pass must not discard geometry only present in the shadow batch.
    group.header_mut().bounds_maximum[0] = 4.;
    for chunk in group.chunks_mut() {
        match chunk {
            GroupChunk::Positions(rows) => {
                *rows = Array::from_records(&[
                    [-1., 0., -1.],
                    [1., 0., -1.],
                    [1., 0., 1.],
                    [-1., 0., 1.],
                    [2., 0., -1.],
                    [4., 0., -1.],
                    [3., 0., 1.],
                ])
                .unwrap()
            }
            GroupChunk::Normals(rows) => *rows = Array::from_records(&[[0., 1., 0.]; 7]).unwrap(),
            GroupChunk::TextureCoordinates(rows) => {
                *rows = Array::from_records(&[
                    [0., 1.],
                    [1., 1.],
                    [1., 0.],
                    [0., 0.],
                    [0., 1.],
                    [1., 1.],
                    [0.5, 0.],
                ])
                .unwrap()
            }
            GroupChunk::VertexColors(rows) => {
                *rows = Array::from_records(&[[0, 0, 0, 255]; 7]).unwrap()
            }
            GroupChunk::Indices(rows) => {
                *rows = Array::from_records(&[0u16, 1, 2, 0, 2, 3, 4, 5, 6]).unwrap()
            }
            GroupChunk::Triangles(rows) => {
                *rows = Array::from_records(&[records::Triangle::default(); 3]).unwrap()
            }
            _ => {}
        }
    }
    group.chunks_mut().push(GroupChunk::ShadowBatches(
        Array::from_records(&[records::ShadowBatch {
            start: 6,
            count: 3,
            ..Default::default()
        }])
        .unwrap(),
    ));
    bundle.validate_complete().unwrap();
    bundle
}

#[test]
fn native_shadow_geometry_and_public_baked_provenance_are_retained() {
    let native = shadow_fixture();
    let model = semantic_wmo::import(&native, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let pose = PoseBaker::default()
        .bake(&prepared, &meshes, &binds, PoseOptions::default())
        .unwrap();
    let counts: Vec<_> = pose
        .parts
        .iter()
        .map(|part| {
            let BakedSource::Geometry(source) = part.source else {
                panic!("expected native geometry")
            };
            assert!(part.visible);
            (source.purpose, part.mesh.indices().unwrap().len())
        })
        .collect();
    assert_eq!(
        counts,
        [(DrawPurpose::Surface, 6), (DrawPurpose::ShadowOnly, 3)]
    );
    let shadow = &pose.parts[1];
    assert!(shadow.material.as_ref().unwrap().extension.casts_shadows());
}

#[test]
fn shadow_only_class_removal_preserves_other_application_classes() {
    struct ApplicationVisibility;
    let mut classes = VisibilityClass::default();
    classes.push(TypeId::of::<Mesh3d>());
    classes.push(TypeId::of::<ApplicationVisibility>());
    DrawPurpose::ShadowOnly.configure_visibility(&mut classes);
    assert!(!classes.contains(&TypeId::of::<Mesh3d>()));
    assert!(classes.contains(&TypeId::of::<ApplicationVisibility>()));
}

#[test]
fn native_shadow_pass_is_excluded_from_camera_but_retained_by_real_light_visibility() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        VisibilityPlugin,
        MeshPlugin,
        ImagePlugin::default(),
    ));
    app.init_asset::<Shader>();
    app.add_plugins(Wc3BevyPlugin);
    app.add_systems(
        PostUpdate,
        check_dir_light_mesh_visibility
            .after(VisibilitySystems::CheckVisibility)
            .before(VisibilitySystems::MarkNewlyHiddenEntitiesInvisible),
    );
    let native = shadow_fixture();
    let model = semantic_wmo::import(&native, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let prepared = compiled.prepare(
        &mut meshes,
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
    );
    // The live evaluator reads these exact asset stores when updating opacity.
    // Dropping a separate temporary material store would hide both passes.
    let mut materials = app
        .world_mut()
        .remove_resource::<Assets<ModelMaterial>>()
        .unwrap();
    let mut commands = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut commands, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    app.insert_resource(materials);
    app.insert_resource(meshes);
    commands.apply(app.world_mut());
    app.world_mut()
        .entity_mut(root)
        .insert(RenderLayers::layer(3));
    let draws: Vec<_> = app
        .world_mut()
        .query::<(Entity, &ModelDraw)>()
        .iter(app.world())
        .map(|(entity, draw)| (entity, draw.purpose))
        .collect();
    assert_eq!(draws.len(), 2);
    for &(entity, _) in &draws {
        app.world_mut().entity_mut(entity).insert(NoFrustumCulling);
    }
    let camera = app
        .world_mut()
        .spawn((Camera3d::default(), RenderLayers::layer(3)))
        .id();
    let mut frusta = CascadesFrusta::default();
    frusta.frusta.insert(camera, vec![Frustum::default()]);
    let light = app
        .world_mut()
        .spawn((
            DirectionalLight {
                shadow_maps_enabled: true,
                ..Default::default()
            },
            RenderLayers::layer(3),
            frusta,
        ))
        .id();
    app.update();
    app.update();
    let visible = app
        .world()
        .get::<VisibleEntities>(camera)
        .unwrap()
        .get(TypeId::of::<Mesh3d>());
    let shadow_views = app.world().get::<CascadesVisibleEntities>(light).unwrap();
    let casters = &shadow_views.entities[&camera][0].entities;
    for &(entity, purpose) in &draws {
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Inherited
        );
        assert!(app
            .world()
            .get::<InheritedVisibility>(entity)
            .unwrap()
            .get());
        assert_eq!(visible.contains(&entity), purpose == DrawPurpose::Surface);
        assert!(
            casters.contains(&entity),
            "{purpose:?} lost its light-shadow pass"
        );
        assert!(app.world().get::<ViewVisibility>(entity).unwrap().get());
        assert!(app.world().entity(entity).contains::<Mesh3d>());
    }
    let shadow = draws
        .iter()
        .find(|(_, purpose)| *purpose == DrawPurpose::ShadowOnly)
        .unwrap()
        .0;
    app.world_mut()
        .entity_mut(shadow)
        .insert(RenderLayers::layer(4));
    app.update();
    assert!(!app
        .world()
        .get::<CascadesVisibleEntities>(light)
        .unwrap()
        .entities[&camera][0]
        .entities
        .contains(&shadow));
}

#[test]
#[ignore = "writes a complete native WMO/TGA fixture for the existing capture example"]
fn write_native_shadow_routing_capture_fixture() {
    use super::{create_dir_all, texture, write, write_fixture};
    let directory = "/tmp/wc3-wmo-shadow-routing-fixture";
    create_dir_all(directory).unwrap();
    for name in ["primary", "pattern", "tertiary"] {
        write(
            format!("{directory}/{name}.tga"),
            texture(&[[200, 150, 100, 255]; 16]),
        )
        .unwrap();
    }
    write_fixture(directory, "shadow-only", shadow_fixture());
}
