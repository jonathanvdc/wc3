use glam::Vec3;
use wc3::model::common::PackedArray;
use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::profiles::{Narrows, Universal, Wmo};
use wc3::model::GeometrySchema;
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometryData, GeometryEdit,
    SurfaceTriangleControls, Topology, VertexRemap, WmoGeometry,
};

fn duplicate() -> GeometryEdit<'static> {
    GeometryEdit {
        remap: Some(VertexRemap {
            new_to_old: vec![2, 0, 1, 0],
            topology: Topology::Triangles(PackedArray::from_storage(
                vec![[3, 2, 0]].into_zero_vec(),
            )),
        }),
        ..Default::default()
    }
}

#[test]
fn world_vertex_streams_remap_while_face_controls_and_draw_pools_keep_their_domains() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let triangles = [[0, 1, 2]];
    let primary = [[0.1; 4], [0.2; 4], [0.3; 4]];
    let lighting = [[0.4; 4], [0.5; 4], [0.6; 4]];
    let lightmap = [[0.1; 2], [0.2; 2], [0.3; 2]];
    let pool = [[1., 0., 0., 1.]];
    let controls = [SurfaceTriangleControls {
        collision: true,
        lightmap_selector: Some(7),
        ..Default::default()
    }];
    let geometry = WmoGeometry::triangles(
        positions[..]
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        &triangles[..],
    )
    .unwrap()
    .with_color_sets(vec![
        PackedArray::from_storage(ZeroVec::alloc_from_slice(&primary)),
        PackedArray::from_storage(ZeroVec::alloc_from_slice(&lighting)),
    ])
    .unwrap()
    .with_lightmap_coordinates(PackedArray::from_storage(ZeroVec::alloc_from_slice(
        &lightmap,
    )))
    .unwrap()
    .with_tangent_vectors(PackedArray::from_storage(ZeroVec::alloc_from_slice(&pool)))
    .unwrap()
    .with_triangle_controls(PackedArray::from_storage(ZeroVec::alloc_from_slice(
        &controls,
    )))
    .unwrap();
    let restored = <Wmo as Narrows<Universal>>::geometry(geometry.clone().into()).unwrap();
    assert_eq!(
        (restored.color_sets()[1].to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        lighting
    );
    assert_eq!(
        (restored.lightmap_coordinates().to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        lightmap
    );
    let mapped = geometry.edited(duplicate()).unwrap();
    assert_eq!(
        (mapped.color_sets()[0].to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        vec![primary[2], primary[0], primary[1], primary[0]]
    );
    assert_eq!(
        (mapped.color_sets()[1].to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        vec![lighting[2], lighting[0], lighting[1], lighting[0]]
    );
    assert_eq!(
        (mapped.lightmap_coordinates().to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        vec![lightmap[2], lightmap[0], lightmap[1], lightmap[0]]
    );
    assert_eq!(mapped.triangle_controls().to_vec(), controls);
    assert_eq!(
        (mapped.tangent_vectors().to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        pool
    );
    assert!(mapped.tangents().is_none());
    let strip = geometry
        .with_strip_indices(PackedArray::from_storage(vec![0, 1, 2].into_zero_vec()))
        .unwrap();
    assert!(strip.edited(duplicate()).is_err());
}

#[test]
fn canonical_attributes_share_storage_and_check_keys_types_and_domains() {
    use wc3::model::{GeometryAttributes, GeometryKind};
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 1.]; 3];
    let face_normals = [[0., 1., 0.]];
    let uv = [[0.5; 2]; 3];
    let colors = [[0.25; 4]; 3];
    let pool = [[0., 1., 0., -1.]];
    let mut data = GeometryData::triangles(
        positions[..]
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        vec![[0, 1, 2]],
    )
    .unwrap();
    data.set_normals(PackedArray::from_storage(ZeroVec::alloc_from_slice(
        &normals,
    )));
    data.set_colors(PackedArray::from_storage(ZeroVec::alloc_from_slice(
        &colors,
    )));
    data.set_texture_coordinates(vec![
        PackedArray::from_storage(ZeroVec::default()),
        PackedArray::from_storage(ZeroVec::alloc_from_slice(&uv)),
    ]);
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Normal,
            domain: AttributeDomain::Face,
            values: AttributeValues::Vector3(PackedArray::from_storage(ZeroVec::alloc_from_slice(
                &face_normals,
            ))),
        })
        .unwrap();
    assert!(data
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Normal,
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector3(PackedArray::from_storage(
                vec![[1.; 3]; 3].into_zero_vec()
            )),
        })
        .is_err());
    assert!(data
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Tangent,
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector3(PackedArray::from_storage(
                vec![[1.; 3]; 3].into_zero_vec()
            )),
        })
        .is_err());
    let face = data
        .attributes
        .iter()
        .position(|attribute| attribute.domain == AttributeDomain::Face)
        .unwrap();
    assert!(data
        .attributes
        .update(face, |attribute| attribute.domain = AttributeDomain::Vertex)
        .is_err());
    assert_eq!(data.attributes[face].domain, AttributeDomain::Face);
    assert_eq!(
        (data.normals().unwrap().to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        normals
    );
    assert_eq!(
        (data.colors().unwrap().to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        colors
    );
    assert!(data.texture_coordinates()[0].is_empty());
    assert_eq!(
        (data.texture_coordinates()[1].to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        uv
    );
    data.validate().unwrap();
    let edited = data
        .edited(GeometryEdit {
            normals: Some(PackedArray::from_storage(ZeroVec::default())),
            ..Default::default()
        })
        .unwrap();
    assert!(edited.normals().is_none());
    assert_eq!(
        (edited
            .attributes
            .vector3(&AttributeSemantic::Normal, AttributeDomain::Face)
            .unwrap()
            .to_vec())
        .into_iter()
        .map(|v| v.to_array())
        .collect::<Vec<_>>(),
        face_normals
    );
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Tangent,
            domain: AttributeDomain::Pool,
            values: AttributeValues::Vector4(PackedArray::from_storage(ZeroVec::alloc_from_slice(
                &pool,
            ))),
        })
        .unwrap();
    data.validate().unwrap();
    assert_eq!(
        (data.tangent_pool().unwrap().to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        pool
    );
    assert!(data.tangents().is_none());
    assert!(!data.edit_capabilities().vertex_remap);
    assert_eq!(data.kind(), GeometryKind::Display);
    let mut collection = GeometryAttributes::default();
    collection
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(2),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector2(PackedArray::from_storage(ZeroVec::alloc_from_slice(
                &uv,
            ))),
        })
        .unwrap();
    assert_eq!(collection.texture_coordinates().len(), 3);
    assert!(collection.texture_coordinates()[0].is_empty());
    assert!(collection.texture_coordinates()[1].is_empty());
    assert_eq!(
        (collection.texture_coordinates()[2].to_vec())
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
        uv
    );
}
