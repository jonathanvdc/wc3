use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::semantic::profiles::{
    diablo::{d3, d4},
    Either,
};
use wc3::model::semantic::ValueGroups;
use wc3::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometryData, GeometryEdit,
    GeometrySchema, Influence, JointInfluences, M2Geometry, M3CollisionGeometry, M3Geometry,
    MdlxGeometry, StreamSupport, Topology, VertexRemap, WmoGeometry,
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
            topology: Topology::Triangles(vec![[3, 2, 0]].into_zero_vec()),
        }),
        ..Default::default()
    }
}
fn exercise<G: GeometrySchema<'static>>(geometry: G) {
    let old = geometry.positions().to_vec();
    let remapped = geometry.edited(duplicate()).unwrap();
    assert_eq!(
        remapped.positions().to_vec(),
        vec![old[2], old[0], old[1], old[0]]
    );
    assert_eq!(geometry.positions().to_vec(), old);
    remapped.validate().unwrap();
    let failure = GeometryEdit {
        positions: Some(vec![[f32::NAN, 0., 0.]; 3].into_zero_vec()),
        ..Default::default()
    };
    assert!(geometry.edited(failure).is_err());
    assert_eq!(geometry.positions().to_vec(), old);
}
#[test]
fn common_tools_work_across_profiles_and_dispatchers() {
    let data = GeometryData::triangles(positions(), triangles()).unwrap();
    exercise(data.clone());
    exercise(
        MdlxGeometry::triangles(positions(), triangles())
            .unwrap()
            .with_influences(JointInfluences::MatrixGroups(ValueGroups::from_rows(vec![
                vec![
                    0, 1
                ];
                3
            ])))
            .unwrap(),
    );
    let m2 = M2Geometry::new(
        positions(),
        positions(),
        [
            vec![[0., 0.]; 3].into_zero_vec(),
            vec![[1., 1.]; 3].into_zero_vec(),
        ],
        triangles(),
        four(),
    )
    .unwrap();
    exercise(m2.clone());
    exercise(GeometryData::from(m2.clone()));
    exercise(Either::<M2Geometry<'static>, GeometryData<'static>>::Left(
        m2,
    ));
    exercise(Either::<M2Geometry<'static>, GeometryData<'static>>::Right(
        data.clone(),
    ));
    exercise(M3Geometry::new(positions(), positions(), triangles(), four()).unwrap());
    exercise(M3Geometry::Collision(
        M3CollisionGeometry::new(positions(), triangles())
            .unwrap()
            .with_triangle_controls(vec![7.])
            .unwrap(),
    ));
    exercise(WmoGeometry::triangles(positions(), triangles()).unwrap());
    exercise(d4::Geometry::new(data).unwrap());
    let streams = d3::VertexStreams {
        positions: positions(),
        normals: positions(),
        colors: vec![[1.; 4]; 3].into_zero_vec(),
        texture_coordinates: [
            vec![[0.; 2]; 3].into_zero_vec(),
            vec![[1.; 2]; 3].into_zero_vec(),
        ],
        tangents: positions(),
        extras: d3::VertexExtras {
            auxiliary_colors: vec![[0.5; 4]; 3].into_zero_vec(),
            binormals: positions(),
            direction_auxiliary: vec![[1, 2, 3], [4, 5, 6], [7, 8, 9]].into_zero_vec(),
            cloth_vertices: vec![8, 9, 10].into_zero_vec(),
        },
    };
    let d3 = d3::Geometry::new(streams, triangles(), ZeroVec::default()).unwrap();
    let mapped = d3.edited(duplicate()).unwrap();
    assert_eq!(
        mapped.streams().extras.cloth_vertices.to_vec(),
        vec![10, 8, 9, 8]
    );
    assert_eq!(
        mapped.streams().extras.direction_auxiliary.to_vec(),
        vec![[7, 8, 9], [1, 2, 3], [4, 5, 6], [1, 2, 3]]
    );
    let universal: GeometryData = d3.clone().into();
    let restored = d3::Geometry::try_from(universal.edited(duplicate()).unwrap()).unwrap();
    assert_eq!(
        restored.streams().extras.direction_auxiliary.to_vec(),
        mapped.streams().extras.direction_auxiliary.to_vec()
    );
    let mut malformed = d3.streams().clone();
    malformed.extras.direction_auxiliary = vec![[0; 3]].into_zero_vec();
    assert!(d3.clone().with_streams(malformed).is_err());
    exercise(d3);
}
#[test]
fn remapping_preserves_attribute_domains_and_profile_payloads() {
    let mut data = GeometryData::triangles(positions(), triangles()).unwrap();
    data.attributes = vec![
        Attribute {
            semantic: AttributeSemantic::Named("vertex".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(vec![10, 20, 30].into_zero_vec()),
        },
        Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector2(vec![[0.; 2], [1.; 2], [2.; 2]].into_zero_vec()),
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
        AttributeValues::Vector2(v) => assert_eq!(v.to_vec(), vec![[0.; 2], [1.; 2], [2.; 2]]),
        _ => panic!(),
    }
    let wmo = WmoGeometry::triangles(positions(), triangles())
        .unwrap()
        .with_color_sets(vec![
            vec![[1.; 4]; 3].into_zero_vec(),
            vec![[2.; 4]; 3].into_zero_vec(),
        ])
        .unwrap()
        .with_lightmap_coordinates(vec![[0.; 2], [1.; 2], [2.; 2]].into_zero_vec())
        .unwrap();
    let mapped = wmo.edited(duplicate()).unwrap();
    assert_eq!(mapped.color_sets()[1].len(), 4);
    assert_eq!(mapped.lightmap_coordinates().get(0), Some([2.; 2]));
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
            vec![[0.; 2]; 3].into_zero_vec(),
            vec![[0.; 2]; 3].into_zero_vec(),
        ],
        triangles(),
        four(),
    )
    .unwrap();
    assert_eq!(m2.edit_capabilities().normals, StreamSupport::Required);
    assert!(m2
        .edited(GeometryEdit {
            normals: Some(ZeroVec::default()),
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
            tangents: Some(ZeroVec::default()),
            ..Default::default()
        })
        .is_err());
    let data = GeometryData::triangles(positions(), triangles()).unwrap();
    let invalid = GeometryEdit {
        remap: Some(VertexRemap {
            new_to_old: vec![0, 1, 2],
            topology: Topology::Triangles(vec![[2, 1, 0]].into_zero_vec()),
        }),
        ..Default::default()
    };
    assert!(data.edited(invalid).is_err());
    let wmo = WmoGeometry::triangles(positions(), triangles())
        .unwrap()
        .with_strip_indices(vec![0, 1, 2].into_zero_vec())
        .unwrap();
    assert!(!wmo.edit_capabilities().vertex_remap);
    assert!(wmo.edited(duplicate()).is_err());
}

#[test]
fn edits_publish_transactionally_and_keep_unmodified_borrowed_streams() {
    use wc3::model::semantic::profiles::Universal;
    use wc3::model::semantic::{Error, Mesh, Model};
    let source = ZeroVec::alloc_from_slice(&[[0., 0., 1.]; 3]);
    let geometry = MdlxGeometry::triangles(positions(), triangles())
        .unwrap()
        .with_normals(source.as_slice().as_zerovec())
        .unwrap();
    let edited = geometry
        .edited(GeometryEdit {
            positions: Some(vec![[2., 0., 0.]; 3].into_zero_vec()),
            ..Default::default()
        })
        .unwrap();
    assert!(!edited.normals().unwrap().is_owned());
    let mut draft = Model::<Universal>::builder("Editable");
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
            positions: Some(vec![[4., 0., 0.]; 4].into_zero_vec()),
            ..Default::default()
        })?;
        draft.set_mesh(id, mesh)?;
        Err(Error::new("test", "abort transaction"))
    });
    assert!(failed.is_err());
    assert_eq!(
        collection.mesh(id).unwrap().positions().get(0),
        Some([0., 1., 0.])
    );
}

#[test]
fn polygon_corner_order_and_attributes_survive_vertex_splits() {
    let mut data = GeometryData::empty();
    data.positions = positions();
    data.connectivity.primary = Topology::Polygons {
        corners: vec![0, 1, 2].into_zero_vec(),
        offsets: vec![0, 3].into_zero_vec(),
    };
    let mut geometry = data;
    geometry.attributes = vec![Attribute {
        semantic: AttributeSemantic::Named("selection".into()),
        domain: AttributeDomain::Vertex,
        values: AttributeValues::Unsigned(vec![10, 20, 30].into_zero_vec()),
    }]
    .try_into()
    .unwrap();
    let edit = GeometryEdit {
        remap: Some(VertexRemap {
            new_to_old: vec![2, 0, 1, 0],
            topology: Topology::Polygons {
                corners: vec![3, 2, 0].into_zero_vec(),
                offsets: vec![0, 3].into_zero_vec(),
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
