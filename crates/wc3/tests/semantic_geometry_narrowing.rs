use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::semantic::profiles::{Extends, Mdlx, Narrows, Universal, M2, M3};
use wc3::model::semantic::ValueGroups;
use wc3::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivitySets, GeometryData,
    GeometryEdit, GeometryInfluences, GeometryKind, GeometrySchema, Influence, InfluenceRows,
    JointInfluences, M2Geometry, M3CollisionGeometry, M3DisplayGeometry, M3Geometry, MdlxGeometry,
    Topology,
};

fn four() -> [Influence; 4] {
    [
        Influence {
            joint: 2,
            weight: 0.75,
        },
        Influence {
            joint: 7,
            weight: 0.25,
        },
        Influence {
            joint: 11,
            weight: 0.0,
        },
        Influence {
            joint: 3,
            weight: 0.0,
        },
    ]
}

#[test]
fn shared_vertices_narrow_across_profiles_without_changing_borrowed_buffers() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 1.]; 3];
    let uv0 = [[0., 0.], [1., 0.], [0., 1.]];
    let uv1 = [[0.2, 0.3]; 3];
    let triangles = [[0, 1, 2]];
    let influences = [four(); 3];
    let encoded_positions = ZeroVec::alloc_from_slice(&positions);
    let encoded_normals = ZeroVec::alloc_from_slice(&normals);
    let encoded_uv0 = ZeroVec::alloc_from_slice(&uv0);
    let encoded_uv1 = ZeroVec::alloc_from_slice(&uv1);
    let encoded_triangles = ZeroVec::alloc_from_slice(&triangles);
    let encoded_influences = ZeroVec::alloc_from_slice(&influences);
    let source = M2Geometry::new(
        encoded_positions.as_slice().as_zerovec(),
        encoded_normals.as_slice().as_zerovec(),
        [
            encoded_uv0.as_slice().as_zerovec(),
            encoded_uv1.as_slice().as_zerovec(),
        ],
        encoded_triangles.as_slice().as_zerovec(),
        encoded_influences.as_slice().as_zerovec(),
    )
    .unwrap();
    let widened = <Universal as Extends<M2>>::geometry(source);

    let mdlx = <Mdlx as Narrows<Universal>>::geometry(widened.clone()).unwrap();
    let m3 = <M3 as Narrows<Universal>>::geometry(widened).unwrap();
    for geometry in [GeometryData::from(mdlx), GeometryData::from(m3)] {
        assert!(!geometry.positions().is_owned());
        assert_eq!(
            geometry.positions().as_bytes().as_ptr(),
            encoded_positions.as_bytes().as_ptr()
        );
        assert_eq!(geometry.positions().to_vec(), positions);
        assert_eq!(geometry.normals().unwrap().to_vec(), normals);
        assert_eq!(geometry.texture_coordinates()[0].to_vec(), uv0);
        assert_eq!(geometry.texture_coordinates()[1].to_vec(), uv1);
        let InfluenceRows::Four(rows) = geometry.influences() else {
            panic!("slot representation changed")
        };
        assert_eq!(rows.to_vec(), influences);
        assert_eq!(rows.get(0), Some(four()));
        let Topology::Triangles(rows) = geometry.topology() else {
            panic!("topology changed")
        };
        assert_eq!(rows.to_vec(), triangles);
        let restored = <M2 as Narrows<Universal>>::geometry(geometry).unwrap();
        assert_eq!(restored.influences().joint_weights(0).unwrap(), four());
    }
}

#[test]
fn owned_encoded_storage_survives_geometry_projection() {
    let positions: ZeroVec<'static, [f32; 3]> = vec![[1., 2., 3.]].into_zero_vec();
    let pointer = positions.as_bytes().as_ptr();
    let normals = (vec![[0., 0., 1.]]).into_zero_vec();
    let source = M3Geometry::new(positions, normals, Vec::<[u32; 3]>::new(), vec![four()]).unwrap();
    let widened = <Universal as Extends<M3>>::geometry(source);
    let restored = <Mdlx as Narrows<Universal>>::geometry(widened).unwrap();
    assert_eq!(restored.positions().as_bytes().as_ptr(), pointer);
    assert!(restored.normals().unwrap().is_owned());
    assert_eq!(restored.normals().unwrap().get(0), Some([0., 0., 1.]));
}

#[test]
fn matrix_groups_keep_equal_weight_semantics_and_reject_weighted_only_profiles() {
    let groups = [vec![8, 2, 19, 6, 5]];
    let source = MdlxGeometry::triangles(vec![[0.; 3]], Vec::<[u32; 3]>::new())
        .unwrap()
        .with_influences(JointInfluences::MatrixGroups(ValueGroups::from_rows(
            groups.iter().cloned(),
        )))
        .unwrap();
    let widened = <Universal as Extends<Mdlx>>::geometry(source);
    assert!(<M2 as Narrows<Universal>>::geometry(widened.clone()).is_err());
    assert!(<M3 as Narrows<Universal>>::geometry(widened.clone()).is_err());
    let restored = <Mdlx as Narrows<Universal>>::geometry(widened).unwrap();
    let JointInfluences::MatrixGroups(rows) = restored.vertex_influences() else {
        panic!("matrix groups changed")
    };
    assert_eq!(rows.to_vec(), groups);
    assert_eq!(
        restored.influences().joint_weights(0).unwrap()[0].weight,
        0.2
    );
}

#[test]
fn target_restrictions_reject_streams_instead_of_dropping_or_fabricating_them() {
    let make = || {
        M3DisplayGeometry::new(
            vec![[0.; 3]],
            vec![[0., 0., 1.]],
            Vec::<[u32; 3]>::new(),
            vec![four()],
        )
        .unwrap()
    };
    let two_uv = make()
        .with_texture_coordinates(vec![vec![[0.; 2]].into_zero_vec(); 2])
        .unwrap();
    assert!(M2Geometry::try_from(GeometryData::from(two_uv.clone())).is_ok());
    let colors = two_uv.clone().with_colors(vec![[1.; 4]]).unwrap();
    assert!(M2Geometry::try_from(GeometryData::from(colors.clone())).is_err());
    assert!(MdlxGeometry::try_from(GeometryData::from(colors)).is_err());
    let tangents = two_uv.with_tangents(vec![[1., 0., 0., 1.]]).unwrap();
    assert!(M2Geometry::try_from(GeometryData::from(tangents.clone())).is_err());
    assert!(MdlxGeometry::try_from(GeometryData::from(tangents)).is_ok());
    assert!(M2Geometry::try_from(GeometryData::from(make())).is_err());
    let sparse = make()
        .with_texture_coordinates(vec![ZeroVec::default(), vec![[1.; 2]].into_zero_vec()])
        .unwrap();
    assert!(M3DisplayGeometry::try_from(GeometryData::from(sparse.clone())).is_ok());
    assert!(M2Geometry::try_from(GeometryData::from(sparse.clone())).is_err());
    assert!(MdlxGeometry::try_from(GeometryData::from(sparse)).is_err());
    let no_normals = MdlxGeometry::triangles(vec![[0.; 3]], Vec::<[u32; 3]>::new())
        .unwrap()
        .with_influences(JointInfluences::Four(vec![four()].into_zero_vec()))
        .unwrap();
    assert!(M3DisplayGeometry::try_from(GeometryData::from(no_normals)).is_err());
    let no_skin = GeometryData::triangles(vec![[0.; 3]], Vec::<[u32; 3]>::new()).unwrap();
    assert!(M3DisplayGeometry::try_from(no_skin).is_err());
}

#[test]
fn grouping_and_collision_remain_distinct_from_single_display_topology() {
    let groups = MdlxGeometry::triangle_groups(
        vec![[0.; 3]],
        vec![
            vec![[0, 0, 0]].into_zero_vec(),
            Vec::<[u32; 3]>::new().into_zero_vec(),
        ],
    )
    .unwrap();
    let universal: GeometryData<'_> = groups.into();
    assert!(<M2 as Narrows<Universal>>::geometry(universal.clone()).is_err());
    assert!(<M3 as Narrows<Universal>>::geometry(universal.clone()).is_err());
    let restored = <Mdlx as Narrows<Universal>>::geometry(universal).unwrap();
    let Topology::Mixed(groups) = restored.topology() else {
        panic!("groups lost")
    };
    assert_eq!(groups.len(), 2);
    let collision = M3CollisionGeometry::new(vec![[0.; 3]], vec![[0, 0, 0]])
        .unwrap()
        .with_face_normals(vec![[0., 0., 1.]])
        .unwrap()
        .with_triangle_controls(vec![3.5])
        .unwrap();
    let universal = GeometryData::from(M3Geometry::Collision(collision));

    assert_eq!(universal.kind(), GeometryKind::Collision);
    assert!(<Mdlx as Narrows<Universal>>::geometry(universal.clone()).is_err());
    let M3Geometry::Collision(restored) = <M3 as Narrows<Universal>>::geometry(universal).unwrap()
    else {
        panic!("collision became display")
    };
    assert_eq!(restored.triangle_controls().unwrap().get(0), Some(3.5));
    assert_eq!(restored.face_normals().unwrap().get(0), Some([0., 0., 1.]));
}

#[test]
fn general_geometry_projects_only_when_its_full_domain_is_supported() {
    let mut general = GeometryData::triangles(vec![[0.; 3]], vec![[0, 0, 0]]).unwrap();
    assert!(<Mdlx as Narrows<Universal>>::geometry(general.clone()).is_ok());
    general
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Named("custom".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(vec![23].into_zero_vec()),
        })
        .unwrap();
    assert!(<Mdlx as Narrows<Universal>>::geometry(general).is_err());
    let mut general = GeometryData::triangles(vec![[0.; 3]], vec![[0, 0, 0]]).unwrap();
    general.influences = GeometryInfluences::Variable(
        vec![vec![Influence {
            joint: 2,
            weight: 1.,
        }]]
        .into(),
    );
    assert!(<Mdlx as Narrows<Universal>>::geometry(general).is_err());
}

#[test]
fn universal_edits_may_expand_capabilities_but_narrowing_checks_the_result() {
    let source = M2Geometry::new(
        vec![[0.; 3]],
        vec![[0., 0., 1.]],
        [vec![[0.; 2]].into_zero_vec(), vec![[0.; 2]].into_zero_vec()],
        Vec::<[u32; 3]>::new(),
        vec![four()],
    )
    .unwrap();
    let edit = GeometryEdit {
        tangents: Some(vec![[1., 0., 0., 1.]].into_zero_vec()),
        ..Default::default()
    };
    assert!(source.edited(edit.clone()).is_err());
    let universal = GeometryData::from(source).edited(edit).unwrap();
    assert!(<M2 as Narrows<Universal>>::geometry(universal.clone()).is_err());
    assert!(<M3 as Narrows<Universal>>::geometry(universal).is_ok());
}

#[test]
fn shared_data_can_keep_custom_attributes_and_fixed_slots_together() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 1.]; 3];
    let uv = [[0.; 2]; 3];
    let rows = [four(); 3];
    let custom = [10u32, 20, 30];
    let mut data = GeometryData::from(
        M2Geometry::new(
            &positions[..],
            &normals[..],
            [
                ZeroVec::alloc_from_slice(&uv),
                ZeroVec::alloc_from_slice(&uv),
            ],
            vec![[0, 1, 2]],
            &rows[..],
        )
        .unwrap(),
    );
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Named("custom".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(ZeroVec::alloc_from_slice(&custom)),
        })
        .unwrap();
    let universal = data;

    assert!(<Mdlx as Narrows<Universal>>::geometry(universal.clone()).is_err());
    assert!(<M2 as Narrows<Universal>>::geometry(universal.clone()).is_err());
    assert!(<M3 as Narrows<Universal>>::geometry(universal.clone()).is_err());
    let mut data = universal;
    let GeometryInfluences::Four(values) = &data.influences else {
        panic!("slot domain lost")
    };
    assert_eq!(values.to_vec(), rows);
    let AttributeValues::Unsigned(values) = &data
        .attributes
        .get(
            &AttributeSemantic::Named("custom".into()),
            AttributeDomain::Vertex,
        )
        .unwrap()
        .values
    else {
        unreachable!()
    };
    assert_eq!(values.to_vec(), custom);
    data.attributes.take(
        &AttributeSemantic::Named("custom".into()),
        AttributeDomain::Vertex,
    );
    let restored = M2Geometry::try_from(data).unwrap();
    let InfluenceRows::Four(values) = restored.influences() else {
        unreachable!()
    };
    assert_eq!(values.to_vec(), rows);
}

#[test]
fn remapping_keeps_each_influence_domain_and_custom_attribute_correspondence() {
    use std::mem::discriminant;
    use wc3::model::semantic::VertexRemap;
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let weighted = Influence {
        joint: 2,
        weight: 1.,
    };
    let zero = Influence {
        joint: 9,
        weight: 0.,
    };
    let variable = [vec![weighted], vec![weighted], vec![weighted]];
    let three = [[weighted, zero, zero]; 3];
    let four = [[weighted, zero, zero, zero]; 3];
    let groups = [vec![2, 7], vec![7, 2], vec![9, 7, 2]];
    let custom = [10u32, 20, 30];
    for influences in [
        GeometryInfluences::None,
        GeometryInfluences::Variable(ValueGroups::from_rows(variable)),
        GeometryInfluences::Three(ZeroVec::alloc_from_slice(&three)),
        GeometryInfluences::Four(ZeroVec::alloc_from_slice(&four)),
        GeometryInfluences::MatrixGroups(ValueGroups::from_rows(groups)),
    ] {
        let data = GeometryData {
            positions: ZeroVec::alloc_from_slice(&positions),
            connectivity: ConnectivitySets::new(
                Topology::Triangles(vec![[0, 1, 2]].into_zero_vec()),
                Vec::new(),
            ),
            attributes: vec![Attribute {
                semantic: AttributeSemantic::Named("custom".into()),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Unsigned(ZeroVec::alloc_from_slice(&custom)),
            }]
            .try_into()
            .unwrap(),
            influences,
            ..Default::default()
        };
        let mapped = data
            .edited(GeometryEdit {
                remap: Some(VertexRemap {
                    new_to_old: vec![2, 0, 1, 0],
                    topology: Topology::Triangles(vec![[3, 2, 0]].into_zero_vec()),
                }),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            discriminant(&mapped.influences),
            discriminant(&data.influences)
        );
        for (new, old) in [2usize, 0, 1, 0].into_iter().enumerate() {
            assert_eq!(
                mapped.influences.rows().joint_weights(new),
                data.influences.rows().joint_weights(old)
            );
        }
        let AttributeValues::Unsigned(values) = &mapped.attributes[0].values else {
            unreachable!()
        };
        assert_eq!(values.to_vec(), vec![30, 10, 20, 10]);
    }
}

#[test]
fn narrowing_does_not_relabel_variable_width_rows_as_fixed_slots() {
    let rows = [four(); 3];
    let mut data = GeometryData::from(
        M2Geometry::new(
            vec![[0.; 3]; 3],
            vec![[0., 0., 1.]; 3],
            [
                vec![[0.; 2]; 3].into_zero_vec(),
                vec![[0.; 2]; 3].into_zero_vec(),
            ],
            vec![[0, 1, 2]],
            &rows[..],
        )
        .unwrap(),
    );
    data.influences =
        GeometryInfluences::Variable(ValueGroups::from_rows(vec![four().to_vec(); 3]));
    assert!(M2Geometry::try_from(data.clone()).is_err());
    assert!(MdlxGeometry::try_from(data.clone()).is_err());
    assert!(M3DisplayGeometry::try_from(data).is_err());
}

#[test]
fn collision_purpose_and_face_streams_survive_shared_storage() {
    use wc3::model::semantic::profiles::{
        diablo::{d3, d4},
        D3, D4,
    };
    use wc3::model::semantic::VertexRemap;
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 2.]];
    let controls = [-3.5];
    let original = M3CollisionGeometry::new(&positions[..], vec![[0, 1, 2]])
        .unwrap()
        .with_face_normals(&normals[..])
        .unwrap()
        .with_triangle_controls(&controls[..])
        .unwrap();
    let data = GeometryData::from(original);
    assert_eq!(data.kind(), GeometryKind::Collision);
    let restored = M3CollisionGeometry::try_from(data.clone()).unwrap();
    assert_eq!(restored.positions().to_vec(), positions);
    assert_eq!(restored.face_normals().unwrap().to_vec(), normals);
    assert_eq!(restored.triangle_controls().unwrap().to_vec(), controls);
    let mapped = data
        .edited(GeometryEdit {
            remap: Some(VertexRemap {
                new_to_old: vec![2, 0, 1, 0],
                topology: Topology::Triangles(vec![[3, 2, 0]].into_zero_vec()),
            }),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(mapped.kind(), GeometryKind::Collision);
    let mapped = M3CollisionGeometry::try_from(mapped).unwrap();
    assert_eq!(mapped.face_normals().unwrap().to_vec(), normals);
    assert_eq!(mapped.triangle_controls().unwrap().to_vec(), controls);
    let mut duplicate = data.clone();
    assert!(duplicate
        .attributes
        .insert(duplicate.attributes[0].clone())
        .is_err());
    let mut wrong_type = data.clone();
    assert!(wrong_type
        .attributes
        .update(0, |attribute| attribute.values =
            AttributeValues::Scalar(vec![1.].into_zero_vec()))
        .is_err());
    let mut wrong_domain = data.clone();
    wrong_domain
        .attributes
        .update(0, |attribute| attribute.domain = AttributeDomain::Mesh)
        .unwrap();
    assert!(M3CollisionGeometry::try_from(wrong_domain).is_err());
    let mut shading = data.clone();
    shading.set_normals(vec![[0., 0., 1.]; 3].into_zero_vec());
    assert!(M3CollisionGeometry::try_from(shading).is_err());
    let mut skinning = data.clone();
    skinning.influences = GeometryInfluences::Four(vec![four(); 3].into_zero_vec());
    assert!(M3CollisionGeometry::try_from(skinning).is_err());
    let mut display = data;
    display.kind = GeometryKind::Display;
    assert!(M3CollisionGeometry::try_from(display).is_err());

    for topology in [
        Topology::Points(vec![0, 1, 2].into_zero_vec()),
        Topology::Polygons {
            corners: vec![0, 1, 2].into_zero_vec(),
            offsets: vec![0, 3].into_zero_vec(),
        },
        Topology::Triangles(vec![[0, 1, 2]].into_zero_vec()),
    ] {
        let data = GeometryData {
            kind: GeometryKind::Collision,
            positions: ZeroVec::alloc_from_slice(&positions),
            connectivity: ConnectivitySets::new(topology, Vec::new()),
            ..Default::default()
        };
        assert!(MdlxGeometry::try_from(data.clone()).is_err());
        assert!(M2Geometry::try_from(data.clone()).is_err());
        assert!(M3DisplayGeometry::try_from(data.clone()).is_err());
        assert!(d3::Geometry::try_from(data.clone()).is_err());
        assert!(d4::Geometry::new(data.clone()).is_err());
        let universal = data;
        assert!(<D3 as Narrows<Universal>>::geometry(universal.clone()).is_err());
        assert!(<D4 as Narrows<Universal>>::geometry(universal.clone()).is_err());
        let restored = <M3 as Narrows<Universal>>::geometry(universal).unwrap();
        assert!(matches!(restored, M3Geometry::Collision(_)));
    }
}
