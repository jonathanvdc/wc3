use wc3::model::semantic::profiles::{
    diablo::{d3, d4},
    Extends, Narrows, Universal, Wmo, D3, D4,
};
use wc3::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometryData, GeometryEdit,
    GeometryInfluences, GeometrySchema, Influence, InfluenceRows, JointInfluences,
    M3CollisionGeometry, MdlxGeometry, Topology, UniversalGeometry, Values, VertexRemap,
    WmoGeometry, WmoTriangleControls,
};

fn duplicate() -> GeometryEdit<'static> {
    GeometryEdit {
        remap: Some(VertexRemap {
            new_to_old: vec![2, 0, 1, 0],
            topology: Topology::Triangles(vec![[3, 2, 0]].into()),
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
    let controls = [WmoTriangleControls {
        collision: true,
        lightmap_selector: Some(7),
        ..Default::default()
    }];
    let geometry = WmoGeometry::triangles(&positions[..], &triangles[..])
        .unwrap()
        .with_color_sets(vec![
            Values::from_slice(&primary),
            Values::from_slice(&lighting),
        ])
        .unwrap()
        .with_lightmap_coordinates(Values::from_slice(&lightmap))
        .unwrap()
        .with_tangent_vectors(Values::from_slice(&pool))
        .unwrap()
        .with_triangle_controls(Values::from_slice(&controls))
        .unwrap();
    let restored = <Wmo as Narrows<Universal>>::geometry(geometry.clone().into()).unwrap();
    assert_eq!(
        restored.color_sets()[1].as_slice().unwrap().as_ptr(),
        lighting.as_ptr()
    );
    assert_eq!(
        restored.lightmap_coordinates().as_slice().unwrap().as_ptr(),
        lightmap.as_ptr()
    );
    let mapped = geometry.edited(duplicate()).unwrap();
    assert_eq!(
        mapped.color_sets()[0].to_vec(),
        vec![primary[2], primary[0], primary[1], primary[0]]
    );
    assert_eq!(
        mapped.color_sets()[1].to_vec(),
        vec![lighting[2], lighting[0], lighting[1], lighting[0]]
    );
    assert_eq!(
        mapped.lightmap_coordinates().to_vec(),
        vec![lightmap[2], lightmap[0], lightmap[1], lightmap[0]]
    );
    assert_eq!(
        mapped.triangle_controls().as_slice().unwrap().as_ptr(),
        controls.as_ptr()
    );
    assert_eq!(
        mapped.tangent_vectors().as_slice().unwrap().as_ptr(),
        pool.as_ptr()
    );
    assert!(mapped.tangents().is_none());
    let strip = geometry.with_strip_indices(vec![0, 1, 2].into()).unwrap();
    assert!(strip.edited(duplicate()).is_err());
}

#[test]
fn three_slot_vertices_preserve_borrowing_independent_vectors_and_vertex_extras() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let tangents = [[1., 0., 0.], [2., 0., 1.], [0., 0., 0.]];
    let binormals = [[0., -1., 0.], [0., 0., -1.], [0., 1., 0.]];
    let auxiliary = [[0.1; 4], [0.2; 4], [0.3; 4]];
    let cloth = [13, 17, 19];
    let row = [
        Influence {
            joint: 2,
            weight: 0.6,
        },
        Influence {
            joint: 7,
            weight: 0.4,
        },
        Influence {
            joint: 99,
            weight: 0.,
        },
    ];
    let rows = [row; 3];
    let streams = d3::VertexStreams {
        positions: Values::from_slice(&positions),
        normals: vec![[0., 0., 1.]; 3].into(),
        tangents: Values::from_slice(&tangents),
        colors: vec![[1.; 4]; 3].into(),
        texture_coordinates: [vec![[0.; 2]; 3].into(), vec![[1.; 2]; 3].into()],
        extras: d3::VertexExtras {
            auxiliary_colors: Values::from_slice(&auxiliary),
            binormals: Values::from_slice(&binormals),
            cloth_vertices: Values::from_slice(&cloth),
        },
    };
    let geometry = d3::Geometry::new(streams, vec![[0, 1, 2]], Values::from_slice(&rows)).unwrap();
    let universal = <Universal as Extends<D3>>::geometry(geometry);
    assert!(matches!(universal, UniversalGeometry::Data(_)));
    let UniversalGeometry::Data(data) = universal.clone() else {
        unreachable!()
    };
    for index in 0..data.attributes.len() {
        let mut missing = data.clone();
        missing.attributes.remove(index);
        assert!(d3::Geometry::try_from(missing).is_err());
    }
    let mut wrong_type = data.clone();
    wrong_type.attributes[0].values = AttributeValues::Vector4(vec![[0.; 4]; 3].into());
    assert!(d3::Geometry::try_from(wrong_type).is_err());
    let mut wrong_domain = data.clone();
    wrong_domain.attributes[0].domain = AttributeDomain::Corner;
    assert!(d3::Geometry::try_from(wrong_domain).is_err());
    let mut repeated = data.clone();
    repeated.attributes.push(repeated.attributes[0].clone());
    assert!(d3::Geometry::try_from(repeated).is_err());
    let mut extra = data.clone();
    extra.attributes.push(Attribute {
        semantic: AttributeSemantic::Named("extra".into()),
        domain: AttributeDomain::Vertex,
        values: AttributeValues::Unsigned(vec![0; 3].into()),
    });
    assert!(d3::Geometry::try_from(extra).is_err());
    let mut frames = data.clone();
    frames.tangents = vec![[1., 0., 0., 1.]; 3].into();
    assert!(d3::Geometry::try_from(frames).is_err());
    let mut variable = data.clone();
    variable.influences = GeometryInfluences::Variable(
        rows.iter()
            .map(|row| row.to_vec())
            .collect::<Vec<_>>()
            .into(),
    );
    assert!(d3::Geometry::try_from(variable).is_err());
    let mut unskinned = data.clone();
    unskinned.influences = GeometryInfluences::None;
    let unskinned = d3::Geometry::try_from(unskinned).unwrap();
    assert!(matches!(unskinned.influences(), InfluenceRows::None));
    let mut wrong_uv = data.clone();
    wrong_uv.texture_coordinates.pop();
    assert!(d3::Geometry::try_from(wrong_uv).is_err());
    let mut wrong_topology = data;
    wrong_topology.topology = Topology::Points(vec![0, 1, 2].into());
    assert!(d3::Geometry::try_from(wrong_topology).is_err());
    let mapped_universal = universal.edited(duplicate()).unwrap();
    let restored = <D3 as Narrows<Universal>>::geometry(universal).unwrap();
    assert_eq!(
        restored.streams().tangents.as_slice().unwrap().as_ptr(),
        tangents.as_ptr()
    );
    assert_eq!(
        restored
            .streams()
            .extras
            .auxiliary_colors
            .as_slice()
            .unwrap()
            .as_ptr(),
        auxiliary.as_ptr()
    );
    assert_eq!(
        restored
            .streams()
            .extras
            .cloth_vertices
            .as_slice()
            .unwrap()
            .as_ptr(),
        cloth.as_ptr()
    );
    let InfluenceRows::Three(influences) = restored.influences() else {
        panic!("three-slot domain lost")
    };
    assert_eq!(influences.as_slice().unwrap().as_ptr(), rows.as_ptr());
    assert_eq!(influences.get(0), Some(row));
    assert_eq!(
        restored
            .streams()
            .extras
            .binormals
            .as_slice()
            .unwrap()
            .as_ptr(),
        binormals.as_ptr()
    );
    assert!(restored.tangents().is_none());
    let mapped = <D3 as Narrows<Universal>>::geometry(mapped_universal).unwrap();
    assert_eq!(
        mapped.streams().tangents.to_vec(),
        vec![tangents[2], tangents[0], tangents[1], tangents[0]]
    );
    assert_eq!(
        mapped.streams().extras.cloth_vertices.to_vec(),
        vec![19, 13, 17, 13]
    );
    assert_eq!(
        mapped.streams().extras.binormals.to_vec(),
        vec![binormals[2], binormals[0], binormals[1], binormals[0]]
    );
    assert_eq!(
        mapped.streams().extras.auxiliary_colors.to_vec(),
        vec![auxiliary[2], auxiliary[0], auxiliary[1], auxiliary[0]]
    );
    assert_eq!(mapped.influence_rows().get(3), Some(row));
}

#[test]
fn declared_geometry_widens_without_a_format_tag_and_narrows_by_its_actual_domains() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let declared = [11u32, 22, 33];
    let mut data = GeometryData::triangles(&positions[..], vec![[0, 1, 2]]).unwrap();
    data.attributes.push(Attribute {
        semantic: AttributeSemantic::Named("declared".into()),
        domain: AttributeDomain::Vertex,
        values: AttributeValues::Unsigned(Values::from_slice(&declared)),
    });
    let universal = <Universal as Extends<D4>>::geometry(d4::Geometry::new(data.clone()).unwrap());
    assert!(matches!(universal, UniversalGeometry::Data(_)));
    let restored = <D4 as Narrows<Universal>>::geometry(universal).unwrap();
    assert_eq!(
        restored.positions().as_slice().unwrap().as_ptr(),
        positions.as_ptr()
    );
    let AttributeValues::Unsigned(values) = &restored.data().attributes[0].values else {
        panic!("declared stream lost")
    };
    assert_eq!(values.as_slice().unwrap().as_ptr(), declared.as_ptr());
    assert!(<D4 as Narrows<Universal>>::geometry(data.clone().into()).is_ok());
    data.attributes[0].domain = AttributeDomain::Corner;
    assert!(<D4 as Narrows<Universal>>::geometry(data.into()).is_err());
    let points = GeometryData {
        positions: Values::from_slice(&positions),
        topology: Topology::Points(vec![0, 1, 2].into()),
        ..Default::default()
    };
    assert!(<D4 as Narrows<Universal>>::geometry(points.into()).is_err());
    let vertices = MdlxGeometry::triangles(&positions[..], vec![[0, 1, 2]]).unwrap();
    assert!(<D4 as Narrows<Universal>>::geometry(vertices.clone().into()).is_ok());
    let grouped = vertices
        .clone()
        .with_influences(JointInfluences::MatrixGroups(vec![vec![0, 1]; 3].into()))
        .unwrap();
    assert!(<D4 as Narrows<Universal>>::geometry(grouped.into()).is_err());
    let weighted = vertices
        .with_influences(JointInfluences::Four(
            vec![
                [Influence {
                    joint: 0,
                    weight: 0.
                }; 4];
                3
            ]
            .into(),
        ))
        .unwrap();
    let restored = <D4 as Narrows<Universal>>::geometry(weighted.into()).unwrap();
    assert!(matches!(restored.influences(), InfluenceRows::Four(_)));
}

#[test]
fn collision_remapping_leaves_face_streams_borrowed_and_rejects_shading() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 1.]];
    let controls = [4.25];
    let geometry = M3CollisionGeometry::new(&positions[..], vec![[0, 1, 2]])
        .unwrap()
        .with_face_normals(&normals[..])
        .unwrap()
        .with_triangle_controls(&controls[..])
        .unwrap();
    let mapped = geometry.edited(duplicate()).unwrap();
    assert_eq!(
        mapped.face_normals().unwrap().as_slice().unwrap().as_ptr(),
        normals.as_ptr()
    );
    assert_eq!(
        mapped
            .triangle_controls()
            .unwrap()
            .as_slice()
            .unwrap()
            .as_ptr(),
        controls.as_ptr()
    );
    assert!(mapped.normals().is_none());
    assert!(mapped
        .edited(GeometryEdit {
            normals: Some(vec![[0., 0., 1.]; 4].into()),
            ..Default::default()
        })
        .is_err());
}
