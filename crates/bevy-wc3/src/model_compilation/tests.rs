//! Cross-boundary model preparation invariants.
use super::{compile_model, compile_model_with_options, CompileOptions};
use crate::composition::DrawSelection;
use crate::material::native_fixtures::wmo::Material as WmoMaterial;
use crate::{BakeSpace, PoseBaker, PoseOptions};
use bevy::asset::Assets;
use bevy::math::{Mat4, Vec2, Vec3, Vec4};
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, Mesh, VertexAttributeValues};
use std::collections::HashSet;
use std::sync::Arc;
use wc3::model::materials::mdlx::MaterialStep as MdlxStep;
use wc3::model::materials::program::MaterialStep;
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::profiles::wmo::draw as world_draw;
use wc3::model::profiles::wmo::{ModelMetadata as WorldMetadata, Region};
use wc3::model::profiles::Universal;
use wc3::model::properties::{DrawProperties, InstanceProperties};
use wc3::model::scene::world::WorldOptions;
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Construct as SlotConstruct,
    Curve, Draw, DynamicTrack, ExplicitBinding, GeometryData, GeometryInfluences, IdentityBinding,
    ImageResource, Influence, Material as SemanticMaterial, MaterialSurface, Mesh as ModelMesh,
    MeshInstance, Model, ModelBuilder, ModelMetadata, PropertyValues as SlotValues, Skin, Texture,
    Time, TimeSource, Track, UniversalBinding,
};

fn triangle_mesh<const N: usize>(
    positions: [[f32; 3]; N],
    triangles: [[u32; 3]; 1],
) -> ModelMesh<'static> {
    ModelMesh::triangles(positions.map(Vec3::from_array), triangles)
}

#[test]
fn generic_empty_draw_instances_remain_without_render_passes() {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("whole geometry").unwrap();
    let mesh = builder
        .add_mesh(triangle_mesh(
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            [[0, 1, 2]],
        ))
        .unwrap();
    builder.add_instance(MeshInstance::new(node, mesh)).unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    assert!(compile_model(&model, |_| None).unwrap().draws.is_empty());
}

#[test]
fn world_draw_materials_separate_lighting_classes_and_preserve_source_colors() {
    let slot_values = SlotValues::default();

    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    builder.set_metadata(ModelMetadata::Wmo(WorldMetadata {
        ambient: [32, 64, 96, 255],
        options: WorldOptions {
            no_vertex_attenuation: true,
            ..Default::default()
        },
        ..Default::default()
    }));
    let node = builder.add_named_node("room").unwrap();
    let mut mesh = triangle_mesh([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]);
    mesh.geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Color(0),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector4(
                [Vec4::new(0.5, 0.5, 0.5, 0.); 3].into_iter().collect(),
            ),
        })
        .unwrap();
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let material = builder
        .add_material(SemanticMaterial {
            name: "stone".into(),
            surface: MaterialSurface::from(WmoMaterial {
                shader: Some(0),
                uv_scroll: [Vec2::new(0.25, 0.), Vec2::ZERO],
                ..Default::default()
            }),
        })
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.properties = Some(InstanceProperties::Wmo(Box::new(Region::default())));
    for lighting in [
        world_draw::Lighting::Interior,
        world_draw::Lighting::Exterior,
        world_draw::Lighting::Interior,
    ] {
        instance.draws.push(Draw::new(
            subset,
            Some(DrawProperties::Wmo(world_draw::DrawProperties {
                lighting,
                ..Default::default()
            })),
        ));
    }
    builder
        .add_instance_with_materials(instance, vec![material; 3])
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    assert!(compiled.has_animation);
    assert_eq!(compiled.draws[0].world_state, compiled.draws[2].world_state);
    assert_ne!(compiled.draws[0].world_state, compiled.draws[1].world_state);
    assert!(Arc::ptr_eq(
        &compiled.draws[0].mesh,
        &compiled.draws[1].mesh
    ));
    let values = compiled.draws[0]
        .mesh
        .attribute(Mesh::ATTRIBUTE_COLOR)
        .unwrap();
    let VertexAttributeValues::Float32x4(values) = values else {
        panic!("expected color values")
    };
    assert_eq!(values[0], [48., 32., 16., 0.].map(|value| value / 255.));
    let original = model.mesh(mesh).unwrap();
    let original = original
        .geometry
        .attributes
        .get(&AttributeSemantic::Color(0), AttributeDomain::Vertex)
        .unwrap();
    let AttributeValues::Vector4(values) = &original.values else {
        panic!("expected raw colors")
    };
    assert_eq!(values.get(0), Some(Vec4::new(0.5, 0.5, 0.5, 0.)));
}

#[test]
fn draw_skin_palettes_compact_subsets_share_passes_and_preserve_baking() {
    let mut slot_values = SlotValues::default();

    for explicit in [false, true] {
        let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        let joints: Vec<_> = (0..12)
            .map(|i| builder.add_named_node(format!("joint {i}")).unwrap())
            .collect();
        let matrices: Vec<_> = (0..12)
            .map(|i| Mat4::from_translation(Vec3::X * i as f32))
            .collect();
        let binding = if explicit {
            UniversalBinding::Explicit(ExplicitBinding::new(matrices.clone()).unwrap())
        } else {
            UniversalBinding::Identity(IdentityBinding)
        };
        let skin = builder
            .add_skin(Skin::new(joints.clone(), binding))
            .unwrap();
        // An authored skin not referenced by geometry must not allocate a runtime palette.
        builder
            .add_skin(Skin::new(
                joints.clone(),
                UniversalBinding::Identity(IdentityBinding),
            ))
            .unwrap();
        let mut geometry = GeometryData::triangles(
            [
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
            ]
            .map(Vec3::from_array),
            [[0, 1, 2], [3, 4, 5]],
        )
        .unwrap();
        geometry.influences = GeometryInfluences::Four(
            (0..6)
                .map(|i| {
                    [
                        Influence {
                            joint: if i < 3 { 3 } else { 8 },
                            weight: 1.,
                        },
                        Influence {
                            joint: 11,
                            weight: 0.,
                        },
                        Influence {
                            joint: 11,
                            weight: 0.,
                        },
                        Influence {
                            joint: 11,
                            weight: 0.,
                        },
                    ]
                })
                .collect(),
        );
        let mut mesh = ModelMesh::new(geometry);
        let first = mesh.add_primary_subset(0, 1);
        let second = mesh.add_primary_subset(1, 1);
        let mesh = builder.add_mesh(mesh).unwrap();
        let material = builder
            .add_material(SemanticMaterial {
                name: "two passes".into(),
                surface: MaterialSurface::from(MdlxMaterial {
                    declaration: Default::default(),
                    steps: vec![
                        MdlxStep::construct(&mut slot_values),
                        MdlxStep::construct(&mut slot_values),
                    ],
                }),
            })
            .unwrap();
        let mut instance = MeshInstance::new(joints[0], mesh);
        instance.skin = Some(skin);
        instance.draws = vec![Draw::new(first, None), Draw::new(second, None)];
        builder
            .add_instance_with_materials(instance, vec![material; 2])
            .unwrap();
        let model = {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap();
        let compiled = compile_model(&model, |_| None).unwrap();
        assert_eq!(compiled.nodes.len(), 12);
        assert_eq!(compiled.skins.len(), 2);
        assert_eq!(compiled.draws.len(), 4);
        for (draw_index, draw) in compiled.draws.iter().enumerate() {
            let slot = if draw_index < 2 { 3 } else { 8 };
            let palette = &compiled.skins[draw.skin.unwrap()];
            assert_eq!(palette.joints, vec![slot]);
            let expected = if explicit {
                matrices[slot]
            } else {
                Mat4::IDENTITY
            };
            assert_eq!(palette.binds.as_ref(), &[expected]);
            let Some(VertexAttributeValues::Uint16x4(indices)) =
                draw.mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
            else {
                panic!("joint indices absent")
            };
            assert!(indices.iter().all(|indices| *indices == [0; 4]));
            assert_eq!(
                draw.mesh
                    .skinned_mesh_bounds()
                    .unwrap()
                    .aabb_index_to_joint_index[0]
                    .0,
                0
            );
        }
        assert_eq!(compiled.draws[0].skin, compiled.draws[1].skin);
        let mut meshes = Assets::default();
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        let prepared = compiled.prepare(&mut meshes, &mut binds);
        let baked = PoseBaker::default()
            .bake(
                &prepared,
                &meshes,
                &binds,
                PoseOptions {
                    space: BakeSpace::Authored,
                    ..Default::default()
                },
            )
            .unwrap();
        for (index, part) in baked.parts.iter().enumerate() {
            let Some(VertexAttributeValues::Float32x3(positions)) =
                part.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("positions absent")
            };
            let translation = if explicit {
                if index < 2 {
                    3.
                } else {
                    8.
                }
            } else {
                0.
            };
            assert_eq!(positions[0], [translation, 0., 0.]);
        }
    }
}

#[test]
fn a_pass_stack_keeps_world_lighting_local_to_each_pass() {
    let mut slot_values = SlotValues::default();

    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("room").unwrap();
    let mut mesh = triangle_mesh([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let interior = WmoMaterial {
        shader: Some(0),
        ..Default::default()
    };
    let mut exterior = interior.clone();
    exterior.state.rasterization.controls.exterior_lit = true;
    let material = builder
        .add_material(SemanticMaterial {
            name: "two lighting programmes".into(),
            surface: MaterialSurface {
                declaration: Default::default(),
                steps: vec![
                    MaterialStep::Wmo(Box::new(interior.into())),
                    MaterialStep::Wmo(Box::new(exterior.into())),
                    MaterialStep::Mdlx(Box::new(SlotConstruct::construct(&mut slot_values))),
                ],
            },
        })
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.draws.push(Draw::new(
        subset,
        Some(DrawProperties::Wmo(world_draw::DrawProperties {
            lighting: world_draw::Lighting::Interior,
            ..Default::default()
        })),
    ));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    assert_eq!(compiled.draws.len(), 3);
    assert_eq!(compiled.draws[0].world_state[3], 1);
    assert_eq!(compiled.draws[1].world_state[3], 2);
    assert_eq!(compiled.draws[2].world_state, [0; 4]);
}

#[test]
fn draw_selection_resolves_images_referenced_by_texture_slots_and_curves() {
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let first = Texture::new(
        builder.add_image_resource(ImageResource::Path(b"first.blp".as_slice().into())),
    );
    let second = Texture::new(
        builder.add_image_resource(ImageResource::Path(b"second.blp".as_slice().into())),
    );
    let layer = MdlxStep::construct(builder.values_mut());
    let image = layer.shader.inputs().binding.image;
    builder.values_mut().set(image, Some(first)).unwrap();
    let surface = MaterialSurface::from(MdlxMaterial {
        declaration: Default::default(),
        steps: vec![layer],
    });
    let material = builder
        .add_material(SemanticMaterial {
            name: "animated texture".into(),
            surface,
        })
        .unwrap();
    let node = builder.add_named_node("root").unwrap();
    let mut mesh = triangle_mesh([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]);
    mesh.geometry
        .set_texture_coordinates(vec![[Vec2::ZERO, Vec2::X, Vec2::Y].into_iter().collect()]);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    let draw = Draw::new(subset, None);
    let draw_id = draw.id();
    instance.draws.push(draw);
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    builder
        .add_global_track(DynamicTrack::OptionalTexture(Track {
            target: image,
            clock: TimeSource::Animation,
            curve: Curve::step([Time::seconds(0.).unwrap()], [Some(second)]).unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let mut resolved = HashSet::new();
    compile_model_with_options(
        &model,
        CompileOptions {
            draws: DrawSelection {
                draws: Some([draw_id].into()),
                ..Default::default()
            },
            ..Default::default()
        },
        |request| {
            resolved.insert(request.texture.resource);
            None
        },
    )
    .unwrap();
    assert!(resolved.contains(&first.resource));
    assert!(resolved.contains(&second.resource));
}
