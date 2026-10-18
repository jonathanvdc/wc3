use super::super::DrawSource;
use super::*;
use wc3::model::appearance::{diablo3, diablo4, UniversalAppearance};
use wc3::model::controls::{diablo3 as d3_controls, diablo4 as d4_controls};
use wc3::model::materials::mdlx::{Layer, LayeredSurface};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    InstanceId, Material as SemanticMaterial, MaterialId, MaterialSurface, Mesh as ModelMesh,
    MeshInstance, ModelBuilder,
};

fn material(builder: &mut ModelBuilder<'static>, name: &str) -> MaterialId {
    let slot_values = builder.values_mut();
    let surface = MaterialSurface::try_from(LayeredSurface {
        layers: vec![Layer::construct(slot_values)],
        ..SlotConstruct::construct(slot_values)
    })
    .unwrap();
    builder
        .add_material(SemanticMaterial {
            name: name.to_owned().into(),
            surface,
        })
        .unwrap()
}
fn instance(builder: &mut ModelBuilder<'static>, controls: Option<MeshControls>) -> InstanceId {
    let node = builder.add_named_node("whole geometry").unwrap();
    let mesh = builder
        .add_mesh(
            ModelMesh::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]])
                .with_controls(controls),
        )
        .unwrap();
    builder.add_instance(MeshInstance::new(node, mesh)).unwrap()
}
#[test]
fn diablo3_empty_draw_instance_renders_whole_geometry_with_selected_appearance() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new("d3 whole-instance");
    let a = material(&mut builder, "a");
    let b = material(&mut builder, "b");
    let mut appearance = diablo3::Appearance::default();
    let slot = appearance.add_slot(diablo3::Slot::named("body"));
    let va = appearance.add_variant(slot, a).unwrap();
    let vb = appearance.add_variant(slot, b).unwrap();
    appearance.add_look("a", vec![(slot, va)]).unwrap();
    let selected = appearance.add_look("b", vec![(slot, vb)]).unwrap();
    builder.set_appearance(UniversalAppearance::D3(appearance));
    let id = instance(
        &mut builder,
        Some(MeshControls::D3(d3_controls::MeshControls {
            material_slot: Some(slot),
            ..SlotConstruct::construct(&mut slot_values)
        })),
    );
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    assert!(model.instance(id).unwrap().draws.is_empty());
    assert!(compile_model(&model, |_| None).is_err());
    let compiled = compile_model_with_options(
        &model,
        CompileOptions {
            material: MaterialSelection::Diablo3(selected),
            ..SlotConstruct::construct(&mut slot_values)
        },
        |_| None,
    )
    .unwrap();
    assert_eq!(compiled.draws.len(), 1);
    assert_eq!(compiled.draws[0].instance, id);
    assert_eq!(compiled.draws[0].material_id, b);
    assert_eq!(compiled.draws[0].source, DrawSource::WholeInstance);
    assert_eq!(compiled.draws[0].mesh.count_vertices(), 3);
}
#[test]
fn diablo4_unambiguous_slot_renders_whole_geometry_without_fabricated_draw() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new("d4 whole-instance");
    let expected = material(&mut builder, "body");
    let mut appearance = diablo4::Appearance::default();
    let slot = appearance.add_slot(diablo4::Slot::new(42, -1, 0));
    appearance.add_variant(slot, expected).unwrap();
    builder.set_appearance(UniversalAppearance::D4(appearance));
    let id = instance(
        &mut builder,
        Some(MeshControls::D4(d4_controls::MeshControls {
            material_slot: Some(slot),
            ..SlotConstruct::construct(&mut slot_values)
        })),
    );
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    assert_eq!(compiled.draws.len(), 1);
    assert_eq!(compiled.draws[0].instance, id);
    assert_eq!(compiled.draws[0].source, DrawSource::WholeInstance);
    assert_eq!(compiled.draws[0].material_id, expected);
}
#[test]
fn generic_empty_draw_instances_remain_without_render_passes() {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new("generic empty instance");
    instance(&mut builder, None);
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    assert!(compile_model(&model, |_| None).unwrap().draws.is_empty());
}

#[test]
fn world_draw_materials_separate_lighting_classes_and_preserve_source_colors() {
    let mut slot_values = SlotValues::default();
    use crate::material::native_fixtures::wmo::Material as WmoMaterial;
    use bevy::mesh::VertexAttributeValues;
    use wc3::model::controls::world::ModelMetadata as WorldMetadata;
    use wc3::model::controls::{world::RegionControls, world_draw, DrawControls, InstanceControls};
    use wc3::model::scene::world::WorldOptions;
    use wc3::model::{Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Draw};

    let mut builder = ModelBuilder::new("world lighting classes");
    builder.set_metadata(ModelMetadata::Wmo(WorldMetadata {
        ambient: [32, 64, 96, 255],
        options: WorldOptions {
            no_vertex_attenuation: true,
            ..SlotConstruct::construct(&mut slot_values)
        },
        ..SlotConstruct::construct(&mut slot_values)
    }));
    let node = builder.add_named_node("room").unwrap();
    let mut mesh = ModelMesh::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]);
    mesh.geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Color(0),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector4([[0.5, 0.5, 0.5, 0.]; 3].into_iter().collect()),
        })
        .unwrap();
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let material = builder
        .add_material(SemanticMaterial {
            name: "stone".into(),
            surface: MaterialSurface::from(WmoMaterial {
                shader: Some(0),
                uv_scroll: [[0.25, 0.], [0.; 2]],
                ..SlotConstruct::construct(&mut slot_values)
            }),
        })
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.controls = Some(InstanceControls::Wmo(Box::new(RegionControls::default())));
    for lighting in [
        world_draw::Lighting::Interior,
        world_draw::Lighting::Exterior,
        world_draw::Lighting::Interior,
    ] {
        instance.draws.push(Draw::new(
            subset,
            Some(DrawControls::Wmo(world_draw::DrawControls {
                lighting,
                ..SlotConstruct::construct(&mut slot_values)
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
    assert_eq!(values.get(0), Some([0.5, 0.5, 0.5, 0.]));
}

#[test]
fn draw_skin_palettes_compact_subsets_share_passes_and_preserve_baking() {
    let mut slot_values = SlotValues::default();
    use crate::{BakeSpace, PoseBaker, PoseOptions};
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    use bevy::mesh::VertexAttributeValues;
    use wc3::model::{
        Draw, ExplicitBinding, GeometryData, GeometryInfluences, IdentityBinding, Influence, Skin,
        UniversalBinding,
    };

    for explicit in [false, true] {
        let mut builder = Model::<Universal>::builder("compact palettes");
        let joints: Vec<_> = (0..12)
            .map(|i| builder.add_named_node(format!("joint {i}")).unwrap())
            .collect();
        let matrices: Vec<_> = (0..12)
            .map(|i| Mat4::from_translation(Vec3::X * i as f32).to_cols_array_2d())
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
            ],
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
                surface: MaterialSurface::try_from(LayeredSurface {
                    layers: vec![
                        Layer::construct(&mut slot_values),
                        Layer::construct(&mut slot_values),
                    ],
                    ..SlotConstruct::construct(&mut slot_values)
                })
                .unwrap(),
            })
            .unwrap();
        let mut instance = MeshInstance::new(joints[0], mesh);
        instance.skin = Some(skin);
        instance.draws = vec![
            Draw::new(first, SlotConstruct::construct(&mut slot_values)),
            Draw::new(second, SlotConstruct::construct(&mut slot_values)),
        ];
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
                Mat4::from_cols_array_2d(&matrices[slot])
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
                    ..SlotConstruct::construct(&mut slot_values)
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
    use crate::material::native_fixtures::wmo::Material as WmoMaterial;
    use wc3::model::controls::{world_draw, DrawControls};
    use wc3::model::materials::program::{MaterialStep, MaterialSurface};
    use wc3::model::Draw;

    let mut builder = ModelBuilder::new("world pass lighting");
    let node = builder.add_named_node("room").unwrap();
    let mut mesh = ModelMesh::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let interior = WmoMaterial {
        shader: Some(0),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut exterior = interior.clone();
    exterior.state.rasterization.controls.exterior_lit = true;
    let material = builder
        .add_material(SemanticMaterial {
            name: "two lighting programmes".into(),
            surface: MaterialSurface {
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
        Some(DrawControls::Wmo(world_draw::DrawControls {
            lighting: world_draw::Lighting::Interior,
            ..SlotConstruct::construct(&mut slot_values)
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
    use wc3::model::{Curve, Draw, DynamicTrack, ImageResource, Time, TimeSource, Track};

    let mut builder = ModelBuilder::new("animated image selection");
    let first = Texture::new(
        builder.add_image_resource(ImageResource::Path(b"first.blp".as_slice().into())),
    );
    let second = Texture::new(
        builder.add_image_resource(ImageResource::Path(b"second.blp".as_slice().into())),
    );
    let layer = Layer::construct(builder.values_mut());
    let image = layer.binding.image;
    builder.values_mut().set(image, Some(first)).unwrap();
    let surface = MaterialSurface::try_from(LayeredSurface {
        layers: vec![layer],
        ..SlotConstruct::construct(builder.values_mut())
    })
    .unwrap();
    let material = builder
        .add_material(SemanticMaterial {
            name: "animated texture".into(),
            surface,
        })
        .unwrap();
    let node = builder.add_named_node("root").unwrap();
    let mut mesh = ModelMesh::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]);
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
