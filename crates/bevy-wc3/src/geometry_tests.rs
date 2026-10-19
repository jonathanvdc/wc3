use super::*;
use bevy::math::{Vec2, Vec4};
use bevy::prelude::Vec3;
use wc3::model::common::PackedArray;
use wc3::model::profiles::Universal;
use wc3::model::ModelMetadata;
use wc3::model::{
    Attribute, ConnectivitySets, GeometryData, Mesh as ModelMesh, ModelBuilder, PrimitiveSelection,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

fn positions(mesh: &Mesh) -> &[[f32; 3]] {
    let Some(VertexAttributeValues::Float32x3(values)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    values
}
fn mesh(positions: &[[f32; 3]], topology: Topology<'static>) -> ModelMesh<'static> {
    ModelMesh::new(GeometryData {
        positions: positions.iter().copied().map(Vec3::from_array).collect(),
        connectivity: ConnectivitySets::new(topology, Vec::new()),
        ..Default::default()
    })
}
#[test]
fn concave_polygon_triangulation_preserves_area_and_winding() {
    let points = [
        [0., 0., 0.],
        [3., 0., 0.],
        [3., 3., 0.],
        [1.5, 1., 0.],
        [0., 3., 0.],
    ];
    let mut source = mesh(
        &points,
        Topology::Polygons {
            corners: [0, 1, 2, 3, 4].into_iter().collect(),
            offsets: [0, 5].into_iter().collect(),
        },
    );
    let id = source.add_primary_subset(0, 1);
    let parts = compile_geometry(&source, source.subset(id).unwrap(), 0).unwrap();
    let vertices = positions(&parts[0].mesh);
    assert_eq!(vertices.len(), 9);
    let mut area = 0.;
    for triangle in vertices.as_chunks::<3>().0 {
        let [a, b, c] = [
            Vec3::from(triangle[0]),
            Vec3::from(triangle[1]),
            Vec3::from(triangle[2]),
        ];
        let signed = (b - a).cross(c - a).z * 0.5;
        assert!(signed > 0.);
        area += signed;
    }
    assert!((area - 6.).abs() < 1e-5);
}
#[test]
fn shared_vertices_keep_independent_corner_uvs() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Topology::Triangles([[0, 1, 2], [2, 1, 3]].into_iter().collect()),
    );
    let coordinates = [
        [0., 0.],
        [1., 0.],
        [0., 1.],
        [0.2, 0.8],
        [0.8, 0.2],
        [1., 1.],
    ];
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector2(
                coordinates.into_iter().map(Vec2::from_array).collect(),
            ),
        })
        .unwrap();
    let id = source.add_primary_subset(0, 2);
    let parts = compile_geometry(&source, source.subset(id).unwrap(), 0).unwrap();
    let Some(VertexAttributeValues::Float32x2(uvs)) = parts[0].mesh.attribute(Mesh::ATTRIBUTE_UV_0)
    else {
        panic!("UVs absent")
    };
    assert_eq!(uvs, &coordinates);
    assert_eq!(positions(&parts[0].mesh)[2], positions(&parts[0].mesh)[3]);
    assert_ne!(uvs[2], uvs[3]);
}
#[test]
fn strip_triangles_have_consistent_winding() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Topology::TriangleStrip([0, 1, 2, 3].into_iter().collect()),
    );
    let id = source.add_primary_subset(0, 2);
    let parts = compile_geometry(&source, source.subset(id).unwrap(), 0).unwrap();
    assert_eq!(positions(&parts[0].mesh).len(), 6);
    for triangle in positions(&parts[0].mesh).as_chunks::<3>().0 {
        let a = Vec3::from(triangle[0]);
        let b = Vec3::from(triangle[1]);
        let c = Vec3::from(triangle[2]);
        assert!((b - a).cross(c - a).z > 0.);
    }
}

#[test]
fn strip_index_window_preserves_original_attribute_rows() {
    use wc3::model::PrimitiveSelection;
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Topology::TriangleStrip([0, 1, 2, 3].into_iter().collect()),
    );
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector2(
                (0..6)
                    .map(|i| [i as f32, 0.])
                    .map(Vec2::from_array)
                    .collect(),
            ),
        })
        .unwrap();
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Color(0),
            domain: AttributeDomain::Face,
            values: AttributeValues::Vector4(
                [[1., 0., 0., 1.], [0., 1., 0., 1.]]
                    .into_iter()
                    .map(Vec4::from_array)
                    .collect(),
            ),
        })
        .unwrap();
    let subset = DrawSubset::new(PrimitiveSelection {
        connectivity: source.subsets.primary(),
        range: PrimitiveRange::StripIndices { first: 1, count: 3 },
    });
    let parts = compile_geometry(&source, &subset, 0).unwrap();
    let Some(VertexAttributeValues::Float32x2(uvs)) = parts[0].mesh.attribute(Mesh::ATTRIBUTE_UV_0)
    else {
        panic!("UVs absent")
    };
    assert_eq!(uvs, &[[4., 0.], [3., 0.], [5., 0.]]);
    let Some(VertexAttributeValues::Float32x4(colors)) =
        parts[0].mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("colors absent")
    };
    assert_eq!(colors, &[[0., 1., 0., 1.]; 3]);
}

#[test]
fn empty_uv_zero_slot_in_a_valid_model_uses_default_coordinates() {
    let mut slot_values = SlotValues::default();
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        Topology::Triangles([[0, 1, 2]].into_iter().collect()),
    );
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector2(SlotConstruct::construct(&mut slot_values)),
        })
        .unwrap();
    let subset = source.add_primary_subset(0, 1);
    let mut builder = ModelBuilder::<Universal>::new(ModelMetadata::Mdlx(Default::default()));
    let id = builder.add_mesh(source).unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let source = model.mesh(id).unwrap();
    let parts = compile_geometry(source, source.subset(subset).unwrap(), 0).unwrap();
    assert_eq!(
        parts[0].mesh.attribute(Mesh::ATTRIBUTE_UV_0),
        Some(&VertexAttributeValues::Float32x2(vec![[0., 0.]; 3]))
    );
}

#[test]
fn mesh_constant_attributes_expand_to_every_selected_corner() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        Topology::Triangles([[0, 1, 2]].into_iter().collect()),
    );
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Mesh,
            values: AttributeValues::Vector2(
                [[0.25, 0.75]].into_iter().map(Vec2::from_array).collect(),
            ),
        })
        .unwrap();
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Color(0),
            domain: AttributeDomain::Mesh,
            values: AttributeValues::Vector4(
                [[0.1, 0.2, 0.3, 1.]]
                    .into_iter()
                    .map(Vec4::from_array)
                    .collect(),
            ),
        })
        .unwrap();
    let id = source.add_primary_subset(0, 1);
    let parts = compile_geometry(&source, source.subset(id).unwrap(), 0).unwrap();
    assert_eq!(
        parts[0].mesh.attribute(Mesh::ATTRIBUTE_UV_0),
        Some(&VertexAttributeValues::Float32x2(vec![[0.25, 0.75]; 3]))
    );
    assert_eq!(
        parts[0].mesh.attribute(Mesh::ATTRIBUTE_COLOR),
        Some(&VertexAttributeValues::Float32x4(vec![
            [0.1, 0.2, 0.3, 1.];
            3
        ]))
    );
}

#[test]
fn selected_connectivity_corner_attributes_override_vertex_streams() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Topology::Triangles([[0, 1, 2]].into_iter().collect()),
    );
    let connectivity = source
        .add_connectivity(Topology::Triangles(
            [[2, 1, 3], [0, 1, 2]].into_iter().collect(),
        ))
        .unwrap();
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector2(
                [[9., 9.]; 4].into_iter().map(Vec2::from_array).collect(),
            ),
        })
        .unwrap();
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::ConnectivityCorner(connectivity),
            values: AttributeValues::Vector2(
                (0..6)
                    .map(|value| [value as f32, 1.])
                    .map(Vec2::from_array)
                    .collect(),
            ),
        })
        .unwrap();
    let subset = DrawSubset::new(PrimitiveSelection {
        connectivity,
        range: PrimitiveRange::Primitives { first: 1, count: 1 },
    });
    let parts = compile_geometry(&source, &subset, 0).unwrap();
    assert_eq!(
        parts[0].mesh.attribute(Mesh::ATTRIBUTE_UV_0),
        Some(&VertexAttributeValues::Float32x2(vec![
            [3., 1.],
            [4., 1.],
            [5., 1.]
        ]))
    );
}

#[test]
fn empty_and_short_strip_windows_produce_no_draws() {
    let source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Topology::TriangleStrip([0, 1, 2, 3].into_iter().collect()),
    );
    for (first, count) in [(4, 0), (3, 1), (2, 2)] {
        let subset = DrawSubset::new(PrimitiveSelection {
            connectivity: source.subsets.primary(),
            range: PrimitiveRange::StripIndices { first, count },
        });
        assert!(compile_geometry(&source, &subset, 0).unwrap().is_empty());
    }
}

#[test]
fn many_small_draw_ranges_preserve_global_face_correspondence() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        Topology::Triangles((0..256).map(|_| [0, 1, 2]).collect()),
    );
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Color(0),
            domain: AttributeDomain::Face,
            values: AttributeValues::Vector4(
                (0..256)
                    .map(|face| [face as f32, 0., 0., 1.])
                    .map(Vec4::from_array)
                    .collect(),
            ),
        })
        .unwrap();
    for first in [0, 1, 37, 128, 255] {
        let subset = DrawSubset::new(PrimitiveSelection {
            connectivity: source.subsets.primary(),
            range: PrimitiveRange::Primitives { first, count: 1 },
        });
        let parts = compile_geometry(&source, &subset, 0).unwrap();
        assert_eq!(positions(&parts[0].mesh).len(), 3);
        assert_eq!(
            parts[0].mesh.attribute(Mesh::ATTRIBUTE_COLOR),
            Some(&VertexAttributeValues::Float32x4(vec![
                [
                    first as f32,
                    0.,
                    0.,
                    1.
                ];
                3
            ]))
        );
    }
}

#[test]
fn third_texture_route_preserves_corner_domain_and_omits_unused_streams() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Topology::Triangles([[0, 1, 2], [2, 1, 3]].into_iter().collect()),
    );
    let third = [
        [0., 0.],
        [1., 0.],
        [0., 1.],
        [0.2, 0.8],
        [0.8, 0.2],
        [1., 1.],
    ];
    for (set, domain, values) in [
        (0, AttributeDomain::Vertex, vec![[0., 0.]; 4]),
        (1, AttributeDomain::Vertex, vec![[0.5, 0.5]; 4]),
        (2, AttributeDomain::Corner, third.to_vec()),
    ] {
        source
            .geometry
            .attributes
            .insert(Attribute {
                semantic: AttributeSemantic::TexCoord(set),
                domain,
                values: AttributeValues::Vector2(
                    values.into_iter().map(Vec2::from_array).collect(),
                ),
            })
            .unwrap();
    }
    let id = source.add_primary_subset(0, 2);
    let subset = source.subset(id).unwrap();
    let parts = compile_geometry_with_uvs(&source, subset, 0, Some(1), Some(2)).unwrap();
    assert_eq!(
        parts[0].mesh.attribute(EXTRA_TEXCOORD),
        Some(&VertexAttributeValues::Float32x2(third.to_vec()))
    );
    assert_ne!(third[2], third[3]);
    let plain = compile_geometry(&source, subset, 0).unwrap();
    assert!(plain[0].mesh.attribute(EXTRA_TEXCOORD).is_none());
    assert!(plain[0].mesh.attribute(Mesh::ATTRIBUTE_UV_1).is_none());
    let error = compile_geometry_with_uvs(&source, subset, 0, Some(1), Some(3))
        .err()
        .unwrap();
    assert!(error.0.contains("texture coordinate set 3 is absent"));
}

#[test]
fn second_color_projects_corner_values_independently_of_primary_illumination() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        Topology::Triangles([[0, 1, 2]].into_iter().collect()),
    );
    for (set, domain, values) in [
        (0, AttributeDomain::Mesh, vec![[0.1, 0.2, 0.3, 0.4]]),
        (
            1,
            AttributeDomain::Corner,
            vec![[0., 0., 0., 0.], [1., 0., 0., 0.5], [0., 1., 0., 1.]],
        ),
    ] {
        source
            .geometry
            .attributes
            .insert(Attribute {
                semantic: AttributeSemantic::Color(set),
                domain,
                values: AttributeValues::Vector4(
                    values.into_iter().map(Vec4::from_array).collect(),
                ),
            })
            .unwrap();
    }
    let parts = compile_geometry(&source, &source.whole_geometry().unwrap(), 0).unwrap();
    assert_eq!(
        parts[0].mesh.attribute(Mesh::ATTRIBUTE_COLOR),
        Some(&VertexAttributeValues::Float32x4(vec![
            [0.1, 0.2, 0.3, 0.4];
            3
        ]))
    );
    assert_eq!(
        parts[0].mesh.attribute(SECOND_COLOR),
        Some(&VertexAttributeValues::Float32x4(vec![
            [0., 0., 0., 0.],
            [1., 0., 0., 0.5],
            [0., 1., 0., 1.]
        ]))
    );
}

#[test]
fn fourth_uv_preserves_corner_rows_without_allocating_unused_streams() {
    let mut source = mesh(
        &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Topology::Triangles([[0, 1, 2], [2, 1, 3]].into_iter().collect()),
    );
    let coordinates = [
        [0., 0.],
        [1., 0.],
        [0., 1.],
        [0.2, 0.8],
        [0.8, 0.2],
        [1., 1.],
    ];
    source
        .geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(3),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector2(
                coordinates.into_iter().map(Vec2::from_array).collect(),
            ),
        })
        .unwrap();
    let id = source.add_primary_subset(0, 2);
    let subset = source.subset(id).unwrap();
    let parts =
        compile_geometry_with_color(&source, subset, 0, [None, None, Some(3)], |_, color| color)
            .unwrap();
    let Some(VertexAttributeValues::Float32x2(uvs)) = parts[0].mesh.attribute(FOURTH_TEXCOORD)
    else {
        panic!("fourth UVs absent")
    };
    assert_eq!(uvs, &coordinates);
    assert_ne!(uvs[2], uvs[3]);
    assert!(!parts[0].mesh.contains_attribute(Mesh::ATTRIBUTE_UV_1));
    assert!(!parts[0].mesh.contains_attribute(EXTRA_TEXCOORD));
    let parts = compile_geometry(&source, subset, 0).unwrap();
    assert!(!parts[0].mesh.contains_attribute(FOURTH_TEXCOORD));
}

#[test]
fn compact_skin_remaps_both_streams_and_ignores_vertices_outside_part() {
    use wc3::model::{GeometryInfluences, Influence, Model, ValueGroups};
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let joints: Vec<_> = (0..16)
        .map(|i| builder.add_named_node(format!("joint {i}")).unwrap())
        .collect();
    let mut geometry = GeometryData::triangles(
        ([[0.; 3]; 3])
            .into_iter()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        [[0, 1, 2]],
    )
    .unwrap();
    let selected = [14, 2, 10, 4, 12, 6, 8, 0];
    geometry.influences = GeometryInfluences::Variable(ValueGroups::from_rows([
        selected
            .map(|joint| Influence {
                joint,
                weight: 0.125,
            })
            .to_vec(),
        vec![Influence {
            joint: 15,
            weight: 1.,
        }],
        vec![Influence {
            joint: 15,
            weight: 1.,
        }],
    ]));
    let source = ModelMesh::new(geometry);
    let mut part = GeometryPart {
        mesh: Mesh::new(PrimitiveTopology::PointList, RenderAssetUsages::default()),
        vertices: vec![0],
    };
    let palette = apply_skin(&mut part, &source, &joints).unwrap();
    assert_eq!(palette, [0, 2, 4, 6, 8, 10, 12, 14]);
    assert_eq!(
        part.mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX),
        Some(&VertexAttributeValues::Uint16x4(vec![[7, 1, 5, 2]]))
    );
    assert_eq!(
        part.mesh.attribute(EXTRA_JOINT_INDEX),
        Some(&VertexAttributeValues::Uint16x4(vec![[6, 3, 4, 0]]))
    );
    assert_eq!(
        part.mesh.attribute(EXTRA_JOINT_WEIGHT),
        Some(&VertexAttributeValues::Float32x4(vec![[0.125; 4]]))
    );
}
