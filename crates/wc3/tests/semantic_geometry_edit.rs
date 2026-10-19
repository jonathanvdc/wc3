use glam::{Vec2, Vec3};
use wc3::model::common::PackedArray;
use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::ModelMetadata;
use wc3::model::ValueGroups;
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometryData, GeometryEdit,
    GeometrySchema, Influence, JointInfluences, M2Geometry, MdlxGeometry, StreamSupport, Topology,
    VertexRemap, WmoGeometry,
};

fn positions() -> ZeroVec<'static, [f32; 3]> {
    vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]].into_zero_vec()
}
fn triangles() -> ZeroVec<'static, [u32; 3]> {
    vec![[0, 1, 2]].into_zero_vec()
}
fn four() -> ZeroVec<'static, [Influence; 4]> {
    vec![
        [Influence {
            joint: 0,
            weight: 0.
        }; 4];
        3
    ]
    .into_zero_vec()
}
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
fn remapping_preserves_attribute_domains_and_profile_payloads() {
    let mut data = GeometryData::triangles(positions(), triangles()).unwrap();
    data.attributes = vec![
        Attribute {
            semantic: AttributeSemantic::Named("vertex".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(PackedArray::from_storage(
                vec![10, 20, 30].into_zero_vec(),
            )),
        },
        Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector2(PackedArray::from_storage(
                vec![[0.; 2], [1.; 2], [2.; 2]].into_zero_vec(),
            )),
        },
    ]
    .try_into()
    .unwrap();
    let mapped = data.edited(duplicate()).unwrap();
    match &mapped.attributes[0].values {
        AttributeValues::Unsigned(v) => assert_eq!(v.to_vec(), vec![30, 10, 20, 10]),
        _ => panic!(),
    }
    match &mapped.attributes[1].values {
        AttributeValues::Vector2(v) => assert_eq!(
            (v.to_vec())
                .into_iter()
                .map(|v| v.to_array())
                .collect::<Vec<_>>(),
            vec![[0.; 2], [1.; 2], [2.; 2]]
        ),
        _ => panic!(),
    }
    let wmo = WmoGeometry::triangles(positions(), triangles())
        .unwrap()
        .with_color_sets(vec![
            PackedArray::from_storage(vec![[1.; 4]; 3].into_zero_vec()),
            PackedArray::from_storage(vec![[2.; 4]; 3].into_zero_vec()),
        ])
        .unwrap()
        .with_lightmap_coordinates(PackedArray::from_storage(
            vec![[0.; 2], [1.; 2], [2.; 2]].into_zero_vec(),
        ))
        .unwrap();
    let mapped = wmo.edited(duplicate()).unwrap();
    assert_eq!(mapped.color_sets()[1].len(), 4);
    assert_eq!(
        mapped.lightmap_coordinates().get(0),
        Some(Vec2::from_array([2.; 2]))
    );
    let mdlx = MdlxGeometry::triangles(positions(), triangles())
        .unwrap()
        .with_influences(JointInfluences::MatrixGroups(ValueGroups::from_rows(vec![
            vec![0],
            vec![1],
            vec![2],
        ])))
        .unwrap();
    let mapped = mdlx.edited(duplicate()).unwrap();
    match mapped.vertex_influences() {
        JointInfluences::MatrixGroups(v) => assert_eq!(v.get(0).unwrap().to_vec(), vec![2]),
        _ => panic!(),
    }
}
#[test]
fn constraints_and_connectivity_changes_are_explicit() {
    let m2 = M2Geometry::new(
        positions(),
        positions(),
        [
            PackedArray::from_storage(vec![[0.; 2]; 3].into_zero_vec()),
            PackedArray::from_storage(vec![[0.; 2]; 3].into_zero_vec()),
        ],
        triangles(),
        four(),
    )
    .unwrap();
    assert_eq!(m2.edit_capabilities().normals, StreamSupport::Required);
    assert!(m2
        .edited(GeometryEdit {
            normals: Some(PackedArray::from_storage(ZeroVec::default())),
            ..Default::default()
        })
        .is_err());
    assert!(m2
        .edited(GeometryEdit {
            texture_coordinates: Some(vec![]),
            ..Default::default()
        })
        .is_err());
    assert!(m2
        .edited(GeometryEdit {
            tangents: Some(PackedArray::from_storage(ZeroVec::default())),
            ..Default::default()
        })
        .is_err());
    let data = GeometryData::triangles(positions(), triangles()).unwrap();
    let invalid = GeometryEdit {
        remap: Some(VertexRemap {
            new_to_old: vec![0, 1, 2],
            topology: Topology::Triangles(PackedArray::from_storage(
                vec![[2, 1, 0]].into_zero_vec(),
            )),
        }),
        ..Default::default()
    };
    assert!(data.edited(invalid).is_err());
    let wmo = WmoGeometry::triangles(positions(), triangles())
        .unwrap()
        .with_strip_indices(PackedArray::from_storage(vec![0, 1, 2].into_zero_vec()))
        .unwrap();
    assert!(!wmo.edit_capabilities().vertex_remap);
    assert!(wmo.edited(duplicate()).is_err());
}

#[test]
fn edits_publish_transactionally_and_keep_unmodified_borrowed_streams() {
    use wc3::model::profiles::Universal;
    use wc3::model::{Error, Mesh, Model};
    let source = ZeroVec::alloc_from_slice(&[[0., 0., 1.]; 3]);
    let geometry = MdlxGeometry::triangles(positions(), triangles())
        .unwrap()
        .with_normals(source.as_slice().as_zerovec())
        .unwrap();
    let edited = geometry
        .edited(GeometryEdit {
            positions: Some(PackedArray::from_storage(
                vec![[2., 0., 0.]; 3].into_zero_vec(),
            )),
            ..Default::default()
        })
        .unwrap();
    assert!(!edited.normals().unwrap().is_owned());
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let id = draft
        .add_mesh(Mesh::new(GeometryData::from(geometry)))
        .unwrap();
    let mut collection = draft.finish().unwrap();
    collection
        .edit(|draft| {
            let mut mesh = draft.model().mesh(id).unwrap().clone();
            mesh.geometry = mesh.geometry.edited(duplicate())?;
            draft.set_mesh(id, mesh)
        })
        .unwrap();
    assert_eq!(collection.mesh(id).unwrap().positions().len(), 4);
    let failed: Result<(), Error> = collection.edit(|draft| {
        let mut mesh = draft.model().mesh(id).unwrap().clone();
        mesh.geometry = mesh.geometry.edited(GeometryEdit {
            positions: Some(PackedArray::from_storage(
                vec![[4., 0., 0.]; 4].into_zero_vec(),
            )),
            ..Default::default()
        })?;
        draft.set_mesh(id, mesh)?;
        Err(Error::new("test", "abort transaction"))
    });
    assert!(failed.is_err());
    assert_eq!(
        collection.mesh(id).unwrap().positions().get(0),
        Some(Vec3::from_array([0., 1., 0.]))
    );
}

#[test]
fn polygon_corner_order_and_attributes_survive_vertex_splits() {
    let mut data = GeometryData::empty();
    data.positions = PackedArray::from_storage(positions());
    data.connectivity.primary = Topology::Polygons {
        corners: PackedArray::from_storage(vec![0, 1, 2].into_zero_vec()),
        offsets: PackedArray::from_storage(vec![0, 3].into_zero_vec()),
    };
    let mut geometry = data;
    geometry.attributes = vec![Attribute {
        semantic: AttributeSemantic::Named("selection".into()),
        domain: AttributeDomain::Vertex,
        values: AttributeValues::Unsigned(PackedArray::from_storage(
            vec![10, 20, 30].into_zero_vec(),
        )),
    }]
    .try_into()
    .unwrap();
    let edit = GeometryEdit {
        remap: Some(VertexRemap {
            new_to_old: vec![2, 0, 1, 0],
            topology: Topology::Polygons {
                corners: PackedArray::from_storage(vec![3, 2, 0].into_zero_vec()),
                offsets: PackedArray::from_storage(vec![0, 3].into_zero_vec()),
            },
        }),
        ..Default::default()
    };
    let result = geometry.edited(edit).unwrap();
    match &result.attributes[0].values {
        AttributeValues::Unsigned(v) => assert_eq!(v.to_vec(), vec![30, 10, 20, 10]),
        _ => panic!(),
    }
    let Topology::Polygons { corners, offsets } = result.topology() else {
        panic!("polygon topology changed");
    };
    assert_eq!(corners.to_vec(), vec![3, 2, 0]);
    assert_eq!(offsets.to_vec(), vec![0, 3]);
}
