use wc3::model::semantic::profiles::{Extends, Mdlx, Narrows, Universal, M2, M3};
use wc3::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometryData, GeometryEdit,
    GeometryInfluences, GeometryKind, GeometrySchema, Influence, InfluenceRows, JointInfluences,
    M2Geometry, M3CollisionGeometry, M3DisplayGeometry, M3Geometry, MdlxGeometry, Topology,
    UniversalGeometry, Values,
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
fn shared_vertices_narrow_across_profiles_without_changing_borrowed_streams() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 1.]; 3];
    let uv0 = [[0., 0.], [1., 0.], [0., 1.]];
    let uv1 = [[0.2, 0.3]; 3];
    let triangles = [[0, 1, 2]];
    let influences = [four(); 3];
    let source = M2Geometry::new(
        &positions[..],
        &normals[..],
        [&uv0[..], &uv1[..]].map(Values::from_slice),
        &triangles[..],
        &influences[..],
    )
    .unwrap();
    let widened = <Universal as Extends<M2>>::geometry(source);
    assert!(matches!(widened, UniversalGeometry::Data(_)));
    let mdlx = <Mdlx as Narrows<Universal>>::geometry(widened.clone()).unwrap();
    let m3 = <M3 as Narrows<Universal>>::geometry(widened).unwrap();
    for geometry in [UniversalGeometry::from(mdlx), UniversalGeometry::from(m3)] {
        assert_eq!(
            geometry.positions().as_slice().unwrap().as_ptr(),
            positions.as_ptr()
        );
        assert_eq!(
            geometry.normals().unwrap().as_slice().unwrap().as_ptr(),
            normals.as_ptr()
        );
        assert_eq!(
            geometry.texture_coordinates()[0]
                .as_slice()
                .unwrap()
                .as_ptr(),
            uv0.as_ptr()
        );
        assert_eq!(
            geometry.texture_coordinates()[1]
                .as_slice()
                .unwrap()
                .as_ptr(),
            uv1.as_ptr()
        );
        let InfluenceRows::Four(rows) = geometry.influences() else {
            panic!("slot representation changed")
        };
        assert_eq!(rows.as_slice().unwrap().as_ptr(), influences.as_ptr());
        assert_eq!(rows.get(0), Some(four()));
        let Topology::Triangles(rows) = geometry.topology() else {
            panic!("topology changed")
        };
        assert_eq!(rows.as_slice().unwrap().as_ptr(), triangles.as_ptr());
        let restored = <M2 as Narrows<Universal>>::geometry(geometry).unwrap();
        assert_eq!(restored.influences().joint_weights(0).unwrap(), four());
    }
}

#[test]
fn owned_and_projected_storage_survives_geometry_projection() {
    let positions: Values<'static, [f32; 3]> = vec![[1., 2., 3.]].into();
    let pointer = positions.as_slice().unwrap().as_ptr();
    let normals = Values::owned(vec![[0., 0., 1.]]).map(|value| value);
    let source = M3Geometry::new(positions, normals, Vec::<[u32; 3]>::new(), vec![four()]).unwrap();
    let widened = <Universal as Extends<M3>>::geometry(source);
    let restored = <Mdlx as Narrows<Universal>>::geometry(widened).unwrap();
    assert_eq!(restored.positions().as_slice().unwrap().as_ptr(), pointer);
    assert!(restored.normals().unwrap().as_slice().is_none());
    assert_eq!(restored.normals().unwrap().get(0), Some([0., 0., 1.]));
}

#[test]
fn matrix_groups_keep_equal_weight_semantics_and_reject_weighted_only_profiles() {
    let groups = [vec![8, 2, 19, 6, 5]];
    let source = MdlxGeometry::triangles(vec![[0.; 3]], Vec::<[u32; 3]>::new())
        .unwrap()
        .with_influences(JointInfluences::MatrixGroups(Values::from_slice(&groups)))
        .unwrap();
    let widened = <Universal as Extends<Mdlx>>::geometry(source);
    assert!(<M2 as Narrows<Universal>>::geometry(widened.clone()).is_err());
    assert!(<M3 as Narrows<Universal>>::geometry(widened.clone()).is_err());
    let restored = <Mdlx as Narrows<Universal>>::geometry(widened).unwrap();
    let JointInfluences::MatrixGroups(rows) = restored.vertex_influences() else {
        panic!("matrix groups changed")
    };
    assert_eq!(rows.as_slice().unwrap().as_ptr(), groups.as_ptr());
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
        .with_texture_coordinates(vec![vec![[0.; 2]].into(); 2])
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
        .with_texture_coordinates(vec![Values::default(), vec![[1.; 2]].into()])
        .unwrap();
    assert!(M3DisplayGeometry::try_from(GeometryData::from(sparse.clone())).is_ok());
    assert!(M2Geometry::try_from(GeometryData::from(sparse.clone())).is_err());
    assert!(MdlxGeometry::try_from(GeometryData::from(sparse)).is_err());
    let no_normals = MdlxGeometry::triangles(vec![[0.; 3]], Vec::<[u32; 3]>::new())
        .unwrap()
        .with_influences(JointInfluences::Four(vec![four()].into()))
        .unwrap();
    assert!(M3DisplayGeometry::try_from(GeometryData::from(no_normals)).is_err());
    let no_skin = GeometryData::triangles(vec![[0.; 3]], Vec::<[u32; 3]>::new()).unwrap();
    assert!(M3DisplayGeometry::try_from(no_skin).is_err());
}

#[test]
fn grouping_and_collision_remain_distinct_from_single_display_topology() {
    let groups = MdlxGeometry::triangle_groups(
        vec![[0.; 3]],
        vec![vec![[0, 0, 0]].into(), Vec::<[u32; 3]>::new().into()],
    )
    .unwrap();
    let universal: UniversalGeometry<'_> = groups.into();
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
    let universal = UniversalGeometry::from(M3Geometry::Collision(collision));
    assert!(matches!(universal, UniversalGeometry::Collision(_)));
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
    assert!(<Mdlx as Narrows<Universal>>::geometry(general.clone().into()).is_ok());
    general.attributes.push(Attribute {
        semantic: AttributeSemantic::Named("custom".into()),
        domain: AttributeDomain::Vertex,
        values: AttributeValues::Unsigned(vec![23].into()),
    });
    assert!(<Mdlx as Narrows<Universal>>::geometry(general.into()).is_err());
    let mut general = GeometryData::triangles(vec![[0.; 3]], vec![[0, 0, 0]]).unwrap();
    general.influences = GeometryInfluences::Variable(
        vec![vec![Influence {
            joint: 2,
            weight: 1.,
        }]]
        .into(),
    );
    assert!(<Mdlx as Narrows<Universal>>::geometry(general.into()).is_err());
}

#[test]
fn universal_edits_may_expand_capabilities_but_narrowing_checks_the_result() {
    let source = M2Geometry::new(
        vec![[0.; 3]],
        vec![[0., 0., 1.]],
        [vec![[0.; 2]].into(), vec![[0.; 2]].into()],
        Vec::<[u32; 3]>::new(),
        vec![four()],
    )
    .unwrap();
    let edit = GeometryEdit {
        tangents: Some(vec![[1., 0., 0., 1.]].into()),
        ..Default::default()
    };
    assert!(source.edited(edit.clone()).is_err());
    let universal = UniversalGeometry::from(source).edited(edit).unwrap();
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
            [Values::from_slice(&uv), Values::from_slice(&uv)],
            vec![[0, 1, 2]],
            &rows[..],
        )
        .unwrap(),
    );
    data.attributes.push(Attribute {
        semantic: AttributeSemantic::Named("custom".into()),
        domain: AttributeDomain::Vertex,
        values: AttributeValues::Unsigned(Values::from_slice(&custom)),
    });
    let universal: UniversalGeometry<'_> = data.into();
    assert!(matches!(universal, UniversalGeometry::Data(_)));
    assert!(<Mdlx as Narrows<Universal>>::geometry(universal.clone()).is_err());
    assert!(<M2 as Narrows<Universal>>::geometry(universal.clone()).is_err());
    assert!(<M3 as Narrows<Universal>>::geometry(universal.clone()).is_err());
    let UniversalGeometry::Data(mut data) = universal else {
        unreachable!()
    };
    let GeometryInfluences::Four(values) = &data.influences else {
        panic!("slot domain lost")
    };
    assert_eq!(values.as_slice().unwrap().as_ptr(), rows.as_ptr());
    let AttributeValues::Unsigned(values) = &data.attributes[0].values else {
        unreachable!()
    };
    assert_eq!(values.as_slice().unwrap().as_ptr(), custom.as_ptr());
    data.attributes.clear();
    let restored = M2Geometry::try_from(data).unwrap();
    let InfluenceRows::Four(values) = restored.influences() else {
        unreachable!()
    };
    assert_eq!(values.as_slice().unwrap().as_ptr(), rows.as_ptr());
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
        GeometryInfluences::Variable(Values::from_slice(&variable)),
        GeometryInfluences::Three(Values::from_slice(&three)),
        GeometryInfluences::Four(Values::from_slice(&four)),
        GeometryInfluences::MatrixGroups(Values::from_slice(&groups)),
    ] {
        let data = GeometryData {
            positions: Values::from_slice(&positions),
            topology: Topology::Triangles(vec![[0, 1, 2]].into()),
            attributes: vec![Attribute {
                semantic: AttributeSemantic::Named("custom".into()),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Unsigned(Values::from_slice(&custom)),
            }],
            influences,
            ..Default::default()
        };
        let mapped = data
            .edited(GeometryEdit {
                remap: Some(VertexRemap {
                    new_to_old: vec![2, 0, 1, 0],
                    topology: Topology::Triangles(vec![[3, 2, 0]].into()),
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
            [vec![[0.; 2]; 3].into(), vec![[0.; 2]; 3].into()],
            vec![[0, 1, 2]],
            &rows[..],
        )
        .unwrap(),
    );
    data.influences = GeometryInfluences::Variable(vec![four().to_vec(); 3].into());
    assert!(M2Geometry::try_from(data.clone()).is_err());
    assert!(MdlxGeometry::try_from(data.clone()).is_err());
    assert!(M3DisplayGeometry::try_from(data).is_err());
}
