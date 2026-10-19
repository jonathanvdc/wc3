use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, VertexAttributeValues};
use bevy::prelude::*;
use bevy_wc3::{
    compile_model, BakeSpace, BakedSource, Playback, PoseBaker, PoseOptions, PreparedModel,
    WorldSelection,
};
use wc3::model::formats::mdlx::animation::{Track, ValueKeyframe};
use wc3::model::formats::mdlx::scene::{Bone, Node, NodeFlags};
use wc3::model::{
    conversion::ImportOptions,
    formats::mdlx::{mdl::Read as _, Document, V800},
    io,
    profiles::Universal,
};

fn prepare(
    source: &Document<'_, V800>,
    image: Option<Handle<Image>>,
) -> (
    PreparedModel,
    Assets<Mesh>,
    Assets<SkinnedMeshInverseBindposes>,
) {
    let model = io::mdlx::import_owned(source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let compiled = compile_model(&model, |_| image.clone()).unwrap();
    (compiled.prepare(&mut meshes, &mut binds), meshes, binds)
}
fn source() -> Document<'static, V800> {
    Document::<V800>::decode_mdl(include_str!("fixtures/quad_model.mdl")).unwrap()
}
fn positions(mesh: &Mesh) -> &[[f32; 3]] {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
}
fn playback(prepared: &PreparedModel, local: f64, shared: f64) -> Playback {
    let mut value = Playback::default();
    value.play(prepared.clips()[0].0);
    value.clip_time = local;
    value.shared_time = shared;
    value
}
#[test]
fn independent_clocks_pose_eight_influences_without_mutating_sources() {
    let mut native = source();
    native.set_global_sequences(&[1000]);
    native.set_bones((0..8).map(|id| {
        let mut node = Node::new("joint", id).unwrap();
        node.translation = Some(
            Track::linear(
                [
                    ValueKeyframe {
                        frame: 0,
                        value: [0.; 3],
                    },
                    ValueKeyframe {
                        frame: 1000,
                        value: [10., 0., 0.],
                    },
                ],
                (id >= 4).then_some(0),
            )
            .unwrap(),
        );
        Bone::new(node, u32::MAX, u32::MAX)
    }));
    native.set_pivot_points(&[[0.; 3]; 8]);
    native
        .geosets_mut()
        .next()
        .unwrap()
        .set_matrix_groups([[0, 1, 2, 3, 4, 5, 6, 7]])
        .unwrap();
    let (prepared, meshes, binds) = prepare(&native, None);
    let part = prepared.parts().next().unwrap();
    let input = positions(meshes.get(part.mesh).unwrap()).to_vec();
    let state = playback(&prepared, 0.25, 0.75);
    let mut baker = PoseBaker::default();
    let options = PoseOptions {
        playback: Some(&state),
        space: BakeSpace::Authored,
        ..default()
    };
    let first = baker.bake(&prepared, &meshes, &binds, options).unwrap();
    let second = baker.bake(&prepared, &meshes, &binds, options).unwrap();
    assert_eq!(
        positions(&first.parts[0].mesh),
        positions(&second.parts[0].mesh)
    );
    for (input, actual) in input.iter().zip(positions(&first.parts[0].mesh)) {
        assert!((actual[0] - input[0] - 5.).abs() < 1e-5);
    }
    assert!(!first.parts[0]
        .mesh
        .contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX));
    assert_eq!(positions(meshes.get(part.mesh).unwrap()), input);
    let BakedSource::Geometry(identity) = first.parts[0].source else {
        panic!()
    };
    assert_eq!(identity.instance, part.source.instance);
    assert_eq!(identity.mesh, part.source.mesh);
}
#[test]
fn snapshots_retain_image_handles_without_changing_shared_assets() {
    let mut images = Assets::<Image>::default();
    let image = images.add(Image::default());
    let (prepared, meshes, binds) = prepare(&source(), Some(image.clone()));
    let baked = PoseBaker::default()
        .bake(&prepared, &meshes, &binds, default())
        .unwrap();
    assert_eq!(
        baked.parts[0]
            .material
            .as_ref()
            .unwrap()
            .base
            .base_color_texture,
        Some(image.clone())
    );
    assert_eq!(
        prepared
            .parts()
            .next()
            .unwrap()
            .material
            .base
            .base_color_texture,
        Some(image)
    );
    assert_eq!(images.len(), 1);
    assert!(baked.bounds.is_some());
}
#[test]
fn mirrored_and_collapsed_scales_keep_finite_geometry() {
    for scale in [[-2., 1., 1.], [0.; 3]] {
        let mut native = source();
        native.bones_mut().next().unwrap().node.scaling = Some(Track::constant(scale));
        let (prepared, meshes, binds) = prepare(&native, None);
        let state = playback(&prepared, 0., 0.);
        let baked = PoseBaker::default()
            .bake(
                &prepared,
                &meshes,
                &binds,
                PoseOptions {
                    playback: Some(&state),
                    space: BakeSpace::Authored,
                    ..default()
                },
            )
            .unwrap();
        for position in positions(&baked.parts[0].mesh) {
            assert!(Vec3::from_array(*position).is_finite());
        }
        if let Some(VertexAttributeValues::Float32x4(tangents)) =
            baked.parts[0].mesh.attribute(Mesh::ATTRIBUTE_TANGENT)
        {
            assert!(tangents.iter().flatten().all(|value| value.is_finite()));
            if scale[0] < 0. {
                assert!(tangents.iter().all(|value| value[3] < 0.));
            }
        }
    }
}
#[test]
fn missing_assets_invalid_clocks_and_camera_requirements_are_errors() {
    let mut native = source();
    native.bones_mut().next().unwrap().node.flags = NodeFlags(8);
    let (prepared, mut meshes, binds) = prepare(&native, None);
    let mut baker = PoseBaker::default();
    assert!(baker.bake(&prepared, &meshes, &binds, default()).is_err());
    let camera = Transform::from_xyz(0., -10., 5.).looking_at(Vec3::ZERO, Vec3::Z);
    let options = PoseOptions {
        camera: Some(camera),
        ..default()
    };
    assert!(baker.bake(&prepared, &meshes, &binds, options).is_ok());
    let state = playback(&prepared, f64::NAN, 0.);
    assert!(baker
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                playback: Some(&state),
                ..options
            }
        )
        .is_err());
    let handle = prepared.parts().next().unwrap().mesh.clone();
    meshes.remove(&handle);
    assert!(baker.bake(&prepared, &meshes, &binds, options).is_err());
}

#[test]
fn world_group_selection_intersects_authored_surface_visibility() {
    use wc3::model::{
        profiles::wmo::{Group, GroupId, WorldSystem},
        System,
    };
    let mut model = io::mdlx::import_owned(&source(), &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let instance = model.instances().iter().next().unwrap().0;
    let system = model
        .edit(|builder| {
            builder.add_system(System::Wmo(WorldSystem {
                groups: vec![Group {
                    instances: vec![instance],
                    ..default()
                }],
                ..default()
            }))
        })
        .unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compile_model(&model, |_| None)
        .unwrap()
        .prepare(&mut meshes, &mut binds);
    let mut baker = PoseBaker::default();
    assert!(
        baker
            .bake(&prepared, &meshes, &binds, default())
            .unwrap()
            .parts[0]
            .visible
    );
    let mut selection = WorldSelection::default();
    selection.hidden_groups.insert(GroupId { system, index: 0 });
    let pose = baker
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                world_selection: Some(&selection),
                ..default()
            },
        )
        .unwrap();
    assert!(!pose.parts[0].visible);
    assert!(pose.bounds.is_none());
}
#[test]
fn malformed_skin_and_normal_streams_are_rejected() {
    for malformed_normal in [false, true] {
        let (prepared, mut meshes, binds) = prepare(&source(), None);
        let handle = prepared.parts().next().unwrap().mesh.clone();
        {
            let mut mesh = meshes.get_mut(&handle).unwrap();
            if malformed_normal {
                let count = mesh.count_vertices();
                *mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL).unwrap() =
                    VertexAttributeValues::Float32x2(vec![[0.; 2]; count]);
            } else {
                mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX);
                mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT);
            }
        }
        assert!(PoseBaker::default()
            .bake(&prepared, &meshes, &binds, default())
            .is_err());
    }
}
