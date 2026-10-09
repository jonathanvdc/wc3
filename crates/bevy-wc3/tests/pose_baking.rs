use bevy::asset::RenderAssetUsages;
use bevy::ecs::world::CommandQueue;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_wc3::{
    prepare_model, spawn_prepared_model, PreparedModel, Wc3Model, Wc3PartId, Wc3PoseBaker,
    Wc3PoseOptions, Wc3TextureBindings, Wc3TextureColorSpace, Wc3TextureRole, Wc3TextureSlot,
};
use wc3::model::animation::{Track, ValueKeyframe};
use wc3::model::geometry::Geoset;
use wc3::model::materials::LayerFilterMode;
use wc3::model::scene::{Bone, Node, NodeFlags};

fn prepare(source: &Wc3Model) -> (PreparedModel, Assets<Mesh>) {
    let mut meshes = Assets::default();
    let prepared = prepare_model(
        &mut meshes,
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
        source,
        |_| None,
    )
    .unwrap();
    (prepared, meshes)
}

fn options(elapsed_ms: f64) -> Wc3PoseOptions {
    Wc3PoseOptions {
        sequence: Some(0),
        elapsed_ms,
        global_elapsed_ms: elapsed_ms,
        ..default()
    }
}

fn positions(mesh: &Mesh) -> &[[f32; 3]] {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
}

fn vector_track(global: Option<u32>) -> Track<[f32; 3]> {
    Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: [0.0; 3],
            },
            ValueKeyframe {
                frame: 1000,
                value: [10.0, 0.0, 0.0],
            },
        ],
        global,
    )
    .unwrap()
}

#[test]
fn source_identity_survives_empty_geosets_shared_meshes_and_spawning() {
    let mut source = Wc3Model::decode_mdl(include_str!("fixtures/geoset_capture.mdl")).unwrap();
    let mut geosets = source.model.geosets();
    let empty = Geoset::new(&[], &[], &[]).unwrap();
    geosets.insert(0, empty);
    source.model.set_geosets(&geosets);
    let mut material = source.model.materials().remove(0);
    material.layers.push(material.layers[0].clone());
    source.model.set_materials(&[material]);
    let (prepared, mut meshes) = prepare(&source);
    let parts: Vec<_> = prepared.parts().collect();
    assert_eq!(
        parts.iter().map(|part| part.id).collect::<Vec<_>>(),
        vec![
            Wc3PartId {
                geoset: 1,
                material: 0,
                layer: 0
            },
            Wc3PartId {
                geoset: 1,
                material: 0,
                layer: 1
            },
            Wc3PartId {
                geoset: 2,
                material: 0,
                layer: 0
            },
            Wc3PartId {
                geoset: 2,
                material: 0,
                layer: 1
            },
        ]
    );
    assert_eq!(parts[0].mesh, parts[1].mesh);
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_prepared_model(
        &mut commands,
        &mut meshes,
        &mut Assets::default(),
        &prepared,
    );
    queue.apply(&mut world);
    let mut query = world.query::<&Wc3PartId>();
    let mut spawned: Vec<_> = query.iter(&world).copied().collect();
    spawned.sort_by_key(|part| (part.geoset, part.layer));
    assert_eq!(
        spawned,
        parts.iter().map(|part| part.id).collect::<Vec<_>>()
    );
}

#[test]
fn independent_clocks_pose_eight_influences_and_repeat_without_mutating_meshes() {
    let mut source = Wc3Model::decode_mdl(include_str!("fixtures/quad_model.mdl")).unwrap();
    source.model.set_global_sequences(&[1000]);
    let mut bones = Vec::new();
    for id in 0..8 {
        let mut node = Node::new("joint", id).unwrap();
        node.translation = Some(vector_track((id >= 4).then_some(0)));
        bones.push(Bone::new(node, u32::MAX, u32::MAX));
    }
    source.model.set_bones(&bones);
    source.model.set_pivot_points(&[[0.0; 3]; 8]);
    let mut geoset = source.model.geosets().remove(0);
    geoset
        .set_matrix_groups(&[vec![0, 1, 2, 3, 4, 5, 6, 7]])
        .unwrap();
    source.model.set_geosets(&[geoset]);
    let (prepared, meshes) = prepare(&source);
    let input = positions(meshes.get(prepared.parts().next().unwrap().mesh).unwrap()).to_vec();
    let mut baker = Wc3PoseBaker::default();
    let mut images = Assets::default();
    let first = baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            Wc3PoseOptions {
                sequence: Some(0),
                elapsed_ms: 250.0,
                global_elapsed_ms: 750.0,
                ..default()
            },
        )
        .unwrap();
    assert!((positions(&first.parts[0].mesh)[0][0] - input[0][0] - 5.0).abs() < 1e-5);
    assert!(!first.parts[0]
        .mesh
        .contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX));
    assert!(!first.parts[0]
        .mesh
        .attributes()
        .any(|(attribute, _)| attribute.name == "Wc3_ExtraJointIndex"));
    assert_eq!(
        positions(meshes.get(prepared.parts().next().unwrap().mesh).unwrap()),
        input
    );
    let second = baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            options(1250.0),
        )
        .unwrap();
    assert!((positions(&second.parts[0].mesh)[0][0] - input[0][0] - 2.5).abs() < 1e-5);
    let third = baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            Wc3PoseOptions {
                sequence: Some(0),
                elapsed_ms: 250.0,
                global_elapsed_ms: 750.0,
                ..default()
            },
        )
        .unwrap();
    assert_eq!(
        positions(&third.parts[0].mesh),
        positions(&first.parts[0].mesh)
    );
}

#[test]
fn material_snapshots_preserve_override_provenance_and_reuse_linear_variants() {
    let mut source = Wc3Model::decode_mdl(include_str!("fixtures/hd_capture.mdl")).unwrap();
    let mut textures = source.model.textures();
    textures[1].replaceable_id = 31;
    source.model.set_textures(&textures);
    let mut materials = source.model.materials();
    materials[0].layers[0].filter_mode = LayerFilterMode::Transparent;
    source.model.set_materials(&materials);
    let (prepared, meshes) = prepare(&source);
    let mut images = Assets::default();
    let mut image = Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![128; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        ..default()
    });
    let by_id = images.add(image.clone());
    let by_slot = images.add(image);
    let mut bindings = Wc3TextureBindings::default();
    bindings.set_replaceable(31, by_id.clone());
    bindings.set_slot(Wc3TextureSlot::Bitmap(1), by_slot.clone());
    bindings.set_slot(Wc3TextureSlot::Bitmap(2), by_slot.clone());
    bindings.set_slot(Wc3TextureSlot::Bitmap(3), by_slot.clone());
    let mut baker = Wc3PoseBaker::default();
    let pose = baker
        .bake(&prepared, &meshes, &mut images, &bindings, options(500.0))
        .unwrap();
    let snapshot = &pose.parts[0].material;
    let normal = snapshot.texture(Wc3TextureRole::Normal).unwrap();
    assert_eq!(normal.bitmap, 1);
    assert_eq!(normal.authored_path.as_deref(), Some("hd_normal.png"));
    assert_eq!(normal.replaceable_id, Some(31));
    assert_eq!(normal.image, Some(by_slot.clone()));
    assert_eq!(normal.color_space, Wc3TextureColorSpace::Linear);
    let ImageSampler::Descriptor(sampler) = &normal.sampler else {
        panic!("lost sampler")
    };
    assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
    let linear = snapshot.material.base.normal_map_texture.clone().unwrap();
    assert_ne!(linear, by_slot);
    assert_eq!(
        images.get(&linear).unwrap().texture_descriptor.format,
        TextureFormat::Rgba8Unorm
    );
    assert_eq!(
        images.get(&by_slot).unwrap().texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(snapshot.material.base.emissive.red, 2.0);
    assert_eq!(snapshot.material.base.uv_transform.translation.x, 0.25);
    assert_eq!(snapshot.fresnel_color, [0.2, 0.4, 1.0]);
    assert_eq!(snapshot.material.base.alpha_mode, AlphaMode::Mask(0.75));
    let count = images.len();
    let repeated = baker
        .bake(&prepared, &meshes, &mut images, &bindings, options(500.0))
        .unwrap();
    assert_eq!(images.len(), count);
    assert_eq!(
        repeated.parts[0].material.material.base.normal_map_texture,
        Some(linear)
    );
    bindings.clear_slot(Wc3TextureSlot::Bitmap(1));
    let replaced = baker
        .bake(&prepared, &meshes, &mut images, &bindings, options(500.0))
        .unwrap();
    assert_eq!(
        replaced.parts[0]
            .material
            .texture(Wc3TextureRole::Normal)
            .unwrap()
            .image,
        Some(by_id)
    );
}

#[test]
fn nonlooping_visibility_tint_bounds_and_lods_are_evaluated_without_world_state() {
    let source = Wc3Model::decode_mdl(include_str!("fixtures/geoset_capture.mdl")).unwrap();
    let (prepared, meshes) = prepare(&source);
    let mut baker = Wc3PoseBaker::default();
    let pose = baker
        .bake(
            &prepared,
            &meshes,
            &mut Assets::default(),
            &Wc3TextureBindings::default(),
            options(2000.0),
        )
        .unwrap();
    assert!(pose.parts[0].visible);
    assert!(!pose.parts[1].visible);
    assert_eq!(
        pose.parts[1]
            .material
            .material
            .base
            .base_color
            .to_linear()
            .to_f32_array(),
        [0.0, 1.0, 0.0, 0.0]
    );
    assert_eq!(pose.bounds.unwrap().center, pose.parts[0].bounds.center);
    let source = Wc3Model::decode_mdl(include_str!("fixtures/lod_capture.mdl")).unwrap();
    let (prepared, meshes) = prepare(&source);
    let pose = baker
        .bake(
            &prepared,
            &meshes,
            &mut Assets::default(),
            &Wc3TextureBindings::default(),
            options(0.0),
        )
        .unwrap();
    assert_eq!(
        pose.parts.iter().map(|part| part.lod).collect::<Vec<_>>(),
        prepared.parts().map(|part| part.lod).collect::<Vec<_>>()
    );
    assert!(pose.parts.len() >= 3);
}

#[test]
fn malformed_hierarchies_nonfinite_times_and_missing_camera_are_errors() {
    let mut source = Wc3Model::decode_mdl(include_str!("fixtures/quad_model.mdl")).unwrap();
    let (prepared, meshes) = prepare(&source);
    let mut baker = Wc3PoseBaker::default();
    let mut images = Assets::default();
    assert!(baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            options(f64::NAN)
        )
        .is_err());
    assert!(baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            Wc3PoseOptions {
                sequence: Some(99),
                ..default()
            }
        )
        .is_err());
    let mut bones = source.model.bones();
    bones[0].node.flags = NodeFlags(8);
    source.model.set_bones(&bones);
    let (prepared, meshes) = prepare(&source);
    assert!(baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            options(0.0)
        )
        .is_err());
    assert!(baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            Wc3PoseOptions {
                camera: Some(Transform::from_xyz(0.0, -10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Z)),
                ..options(0.0)
            }
        )
        .is_ok());
    bones[0].node.flags = NodeFlags(0);
    bones[0].node.parent_id = 0;
    source.model.set_bones(&bones);
    let (prepared, meshes) = prepare(&source);
    assert!(baker
        .bake(
            &prepared,
            &meshes,
            &mut images,
            &Wc3TextureBindings::default(),
            options(0.0)
        )
        .is_err());
}

#[test]
fn mirrored_and_collapsed_scales_keep_normals_and_tangents_finite() {
    let mut source = Wc3Model::decode_mdl(include_str!("fixtures/quad_model.mdl")).unwrap();
    let mut bones = source.model.bones();
    bones[0].node.scaling = Some(Track::constant([-2.0, 1.0, 1.0]));
    source.model.set_bones(&bones);
    let (prepared, meshes) = prepare(&source);
    let pose = Wc3PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &mut Assets::default(),
            &Wc3TextureBindings::default(),
            options(0.0),
        )
        .unwrap();
    let Some(VertexAttributeValues::Float32x4(tangents)) =
        pose.parts[0].mesh.attribute(Mesh::ATTRIBUTE_TANGENT)
    else {
        panic!("tangents missing")
    };
    assert_eq!(tangents[0][3], -1.0);
    bones[0].node.scaling = Some(Track::constant([0.0; 3]));
    source.model.set_bones(&bones);
    let (prepared, meshes) = prepare(&source);
    let pose = Wc3PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &mut Assets::default(),
            &Wc3TextureBindings::default(),
            options(0.0),
        )
        .unwrap();
    assert!(positions(&pose.parts[0].mesh)
        .iter()
        .flatten()
        .all(|value| value.is_finite()));
    assert!(pose.parts[0]
        .mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .flatten()
        .all(|value| value.is_finite()));
}
