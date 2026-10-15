use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::profiles::{
    diablo::{d3, d4},
    Extends, Narrows, Universal, Wmo, D3, D4,
};
use wc3::model::GeometrySchema;
use wc3::model::ValueGroups;
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivitySets, GeometryData,
    GeometryEdit, GeometryInfluences, Influence, InfluenceRows, JointInfluences,
    M3CollisionGeometry, MdlxGeometry, SurfaceTriangleControls, Topology, VertexRemap, WmoGeometry,
};

fn duplicate() -> GeometryEdit<'static> {
    GeometryEdit {
        remap: Some(VertexRemap {
            new_to_old: vec![2, 0, 1, 0],
            topology: Topology::Triangles(vec![[3, 2, 0]].into_zero_vec()),
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
    let geometry = WmoGeometry::triangles(&positions[..], &triangles[..])
        .unwrap()
        .with_color_sets(vec![
            ZeroVec::alloc_from_slice(&primary),
            ZeroVec::alloc_from_slice(&lighting),
        ])
        .unwrap()
        .with_lightmap_coordinates(ZeroVec::alloc_from_slice(&lightmap))
        .unwrap()
        .with_tangent_vectors(ZeroVec::alloc_from_slice(&pool))
        .unwrap()
        .with_triangle_controls(ZeroVec::alloc_from_slice(&controls))
        .unwrap();
    let restored = <Wmo as Narrows<Universal>>::geometry(geometry.clone().into()).unwrap();
    assert_eq!(restored.color_sets()[1].to_vec(), lighting);
    assert_eq!(restored.lightmap_coordinates().to_vec(), lightmap);
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
    assert_eq!(mapped.triangle_controls().to_vec(), controls);
    assert_eq!(mapped.tangent_vectors().to_vec(), pool);
    assert!(mapped.tangents().is_none());
    let strip = geometry
        .with_strip_indices(vec![0, 1, 2].into_zero_vec())
        .unwrap();
    assert!(strip.edited(duplicate()).is_err());
}

#[test]
fn three_slot_vertices_preserve_independent_vectors_and_vertex_extras() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let tangents = [[1., 0., 0.], [2., 0., 1.], [0., 0., 0.]];
    let binormals = [[0., -1., 0.], [0., 0., -1.], [0., 1., 0.]];
    let auxiliary = [[0.1; 4], [0.2; 4], [0.3; 4]];
    let cloth = [13, 17, 19];
    let direction_auxiliary = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
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
        positions: ZeroVec::alloc_from_slice(&positions),
        normals: vec![[0., 0., 1.]; 3].into_zero_vec(),
        tangents: ZeroVec::alloc_from_slice(&tangents),
        colors: vec![[1.; 4]; 3].into_zero_vec(),
        texture_coordinates: [
            vec![[0.; 2]; 3].into_zero_vec(),
            vec![[1.; 2]; 3].into_zero_vec(),
        ],
        extras: d3::VertexExtras {
            auxiliary_colors: ZeroVec::alloc_from_slice(&auxiliary),
            binormals: ZeroVec::alloc_from_slice(&binormals),
            direction_auxiliary: ZeroVec::alloc_from_slice(&direction_auxiliary),
            cloth_vertices: ZeroVec::alloc_from_slice(&cloth),
        },
    };
    let geometry =
        d3::Geometry::new(streams, vec![[0, 1, 2]], ZeroVec::alloc_from_slice(&rows)).unwrap();
    let universal = <Universal as Extends<D3>>::geometry(geometry);

    let data = universal.clone();
    for index in 0..data.attributes.len() {
        let mut missing = data.clone();
        missing.attributes.remove(index);
        if data.attributes[index].semantic
            == AttributeSemantic::Named(d3::DIRECTION_AUXILIARY_ATTRIBUTE.into())
        {
            assert!(d3::Geometry::try_from(missing)
                .unwrap()
                .streams()
                .extras
                .direction_auxiliary
                .is_empty());
        } else {
            assert!(d3::Geometry::try_from(missing).is_err());
        }
    }
    let auxiliary_index = data
        .attributes
        .iter()
        .position(|attribute| {
            attribute.semantic == AttributeSemantic::Named(d3::DIRECTION_AUXILIARY_ATTRIBUTE.into())
        })
        .unwrap();
    let mut wrong_type = data.clone();
    wrong_type
        .attributes
        .update(auxiliary_index, |attribute| {
            attribute.values = AttributeValues::Byte4(vec![[0; 4]; 3].into_zero_vec())
        })
        .unwrap();
    assert!(d3::Geometry::try_from(wrong_type).is_err());
    let mut wrong_domain = data.clone();
    wrong_domain
        .attributes
        .update(0, |attribute| attribute.domain = AttributeDomain::Corner)
        .unwrap();
    assert!(d3::Geometry::try_from(wrong_domain).is_err());
    let mut repeated = data.clone();
    assert!(repeated
        .attributes
        .insert(repeated.attributes[0].clone())
        .is_err());
    let mut extra = data.clone();
    extra
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Named("extra".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(vec![0; 3].into_zero_vec()),
        })
        .unwrap();
    assert!(d3::Geometry::try_from(extra).is_err());
    let mut frames = data.clone();
    frames.set_tangents(vec![[1., 0., 0., 1.]; 3].into_zero_vec());
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
    wrong_uv
        .attributes
        .take(&AttributeSemantic::TexCoord(1), AttributeDomain::Vertex);
    assert!(d3::Geometry::try_from(wrong_uv).is_err());
    let mut wrong_topology = data;
    wrong_topology.connectivity.primary = Topology::Points(vec![0, 1, 2].into_zero_vec());
    assert!(d3::Geometry::try_from(wrong_topology).is_err());
    let mapped_universal = universal.edited(duplicate()).unwrap();
    let restored = <D3 as Narrows<Universal>>::geometry(universal).unwrap();
    assert_eq!(
        restored.streams().extras.direction_auxiliary.to_vec(),
        direction_auxiliary
    );
    assert_eq!(restored.streams().tangents.to_vec(), tangents);
    assert_eq!(
        restored.streams().extras.auxiliary_colors.to_vec(),
        auxiliary
    );
    assert_eq!(restored.streams().extras.cloth_vertices.to_vec(), cloth);
    let InfluenceRows::Three(influences) = restored.influences() else {
        panic!("three-slot domain lost")
    };
    assert_eq!(influences.to_vec(), rows);
    assert_eq!(influences.get(0), Some(row));
    assert_eq!(restored.streams().extras.binormals.to_vec(), binormals);
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
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Named("declared".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(ZeroVec::alloc_from_slice(&declared)),
        })
        .unwrap();
    let universal = <Universal as Extends<D4>>::geometry(d4::Geometry::new(data.clone()).unwrap());

    let restored = <D4 as Narrows<Universal>>::geometry(universal).unwrap();
    assert_eq!(restored.positions().to_vec(), positions);
    let AttributeValues::Unsigned(values) = &restored.data().attributes[0].values else {
        panic!("declared stream lost")
    };
    assert_eq!(values.to_vec(), declared);
    assert!(<D4 as Narrows<Universal>>::geometry(data.clone()).is_ok());
    data.attributes
        .update(0, |attribute| attribute.domain = AttributeDomain::Corner)
        .unwrap();
    assert!(<D4 as Narrows<Universal>>::geometry(data).is_err());
    let points = GeometryData {
        positions: ZeroVec::alloc_from_slice(&positions),
        connectivity: ConnectivitySets::new(
            Topology::Points(vec![0, 1, 2].into_zero_vec()),
            Vec::new(),
        ),
        ..Default::default()
    };
    assert!(<D4 as Narrows<Universal>>::geometry(points).is_err());
    let vertices = MdlxGeometry::triangles(&positions[..], vec![[0, 1, 2]]).unwrap();
    assert!(<D4 as Narrows<Universal>>::geometry(vertices.clone().into()).is_ok());
    let grouped = vertices
        .clone()
        .with_influences(JointInfluences::MatrixGroups(ValueGroups::from_rows(vec![
            vec![
                0, 1
            ];
            3
        ])))
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
            .into_zero_vec(),
        ))
        .unwrap();
    let restored = <D4 as Narrows<Universal>>::geometry(weighted.into()).unwrap();
    assert!(matches!(restored.influences(), InfluenceRows::Four(_)));
}

#[test]
fn collision_remapping_preserves_face_stream_values_and_rejects_shading() {
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
    assert_eq!(mapped.face_normals().unwrap().to_vec(), normals);
    assert_eq!(mapped.triangle_controls().unwrap().to_vec(), controls);
    assert!(mapped.normals().is_none());
    assert!(mapped
        .edited(GeometryEdit {
            normals: Some(vec![[0., 0., 1.]; 4].into_zero_vec()),
            ..Default::default()
        })
        .is_err());
}

#[test]
fn world_shared_domains_preserve_pools_slots_controls_and_checked_edits() {
    use wc3::model::profiles::diablo::{d3, d4};
    use wc3::model::{GeometryKind, M2Geometry, M3DisplayGeometry};
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let triangles = [[0, 1, 2]];
    let colors = [[0.5; 4]; 3];
    let lightmap = [[0.25; 2]; 3];
    let strips = [0, 1, 2, 0, 1];
    let tangents = [[0., 0., 1., -1.]];
    let controls = [SurfaceTriangleControls {
        collision: true,
        render: true,
        transition: true,
        spatial_queries: true,
        ground_type_selector: Some(3),
        ground_type: Some(99),
        lightmap_selector: Some(4),
        alignment: 7,
        unclassified: 0x8000.into(),
        ..Default::default()
    }];
    let original = WmoGeometry::triangles(&positions[..], &triangles[..])
        .unwrap()
        .with_color_sets(vec![
            ZeroVec::alloc_from_slice(&colors),
            ZeroVec::alloc_from_slice(&colors),
        ])
        .unwrap()
        .with_lightmap_coordinates(ZeroVec::alloc_from_slice(&lightmap))
        .unwrap()
        .with_strip_indices(ZeroVec::alloc_from_slice(&strips))
        .unwrap()
        .with_tangent_vectors(ZeroVec::alloc_from_slice(&tangents))
        .unwrap()
        .with_triangle_controls(ZeroVec::alloc_from_slice(&controls))
        .unwrap();
    let universal: GeometryData<'_> = original.into();
    let data = &universal;
    assert_eq!(data.strip_indices().unwrap().to_vec(), strips);
    assert_eq!(data.tangent_pool().unwrap().to_vec(), tangents);
    assert_eq!(data.surface_triangles().unwrap().to_vec(), controls);
    assert_eq!(data.colors().unwrap().to_vec(), colors);
    assert!(!data.edit_capabilities().vertex_remap);
    assert!(universal.edited(duplicate()).is_err());
    assert!(MdlxGeometry::try_from(data.clone()).is_err());
    assert!(M2Geometry::try_from(data.clone()).is_err());
    assert!(M3DisplayGeometry::try_from(data.clone()).is_err());
    assert!(d3::Geometry::try_from(data.clone()).is_err());
    assert!(d4::Geometry::new(data.clone()).is_err());
    let restored = WmoGeometry::try_from(universal.clone()).unwrap();
    assert_eq!(restored.triangle_controls().get(0), Some(controls[0]));
    let edited = universal
        .edited(GeometryEdit {
            colors: Some(vec![[0.75; 4]; 3].into_zero_vec()),
            ..Default::default()
        })
        .unwrap();
    let edited = WmoGeometry::try_from(edited).unwrap();
    assert_eq!(edited.color_sets()[0].to_vec(), vec![[0.75; 4]; 3]);
    assert_eq!(edited.color_sets()[1].to_vec(), colors);
    let mut without_pools = data.clone();
    without_pools.connectivity =
        wc3::model::ConnectivitySets::new(without_pools.connectivity.primary.clone(), Vec::new());
    without_pools
        .attributes
        .take(&AttributeSemantic::Tangent, AttributeDomain::Pool);
    let remapped = WmoGeometry::try_from(without_pools.edited(duplicate()).unwrap()).unwrap();
    assert_eq!(
        remapped.lightmap_coordinates().to_vec(),
        vec![lightmap[2], lightmap[0], lightmap[1], lightmap[0]]
    );
    assert_eq!(remapped.triangle_controls().to_vec(), controls);
    let mut invalid_indices = data.clone();
    invalid_indices.connectivity = wc3::model::ConnectivitySets::new(
        invalid_indices.connectivity.primary.clone(),
        vec![Topology::TriangleStrip(vec![3].into_zero_vec())],
    );
    assert!(invalid_indices.validate().is_err());
    let mut invalid_pool = data.clone();
    invalid_pool
        .attributes
        .set(Attribute {
            semantic: AttributeSemantic::Tangent,
            domain: AttributeDomain::Pool,
            values: AttributeValues::Vector4(vec![[f32::NAN; 4]].into_zero_vec()),
        })
        .unwrap();
    assert!(invalid_pool.validate().is_err());
    let mut wrong_purpose = data.clone();
    wrong_purpose.kind = GeometryKind::Collision;
    assert!(WmoGeometry::try_from(wrong_purpose).is_err());
    let mut extra = data.clone();
    extra
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Named("extra".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(vec![0; 3].into_zero_vec()),
        })
        .unwrap();
    assert!(WmoGeometry::try_from(extra).is_err());
    let mut gap = data.clone();
    gap.attributes
        .retain(|attribute| attribute.semantic != AttributeSemantic::Color(0));
    assert!(WmoGeometry::try_from(gap).is_err());
    let empty = WmoGeometry::default()
        .with_color_sets(vec![ZeroVec::default(), ZeroVec::default()])
        .unwrap();
    let restored = WmoGeometry::try_from(GeometryData::from(empty)).unwrap();
    assert_eq!(restored.color_sets().len(), 2);
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
    let mut data = GeometryData::triangles(&positions[..], vec![[0, 1, 2]]).unwrap();
    data.set_normals(ZeroVec::alloc_from_slice(&normals));
    data.set_colors(ZeroVec::alloc_from_slice(&colors));
    data.set_texture_coordinates(vec![ZeroVec::default(), ZeroVec::alloc_from_slice(&uv)]);
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Normal,
            domain: AttributeDomain::Face,
            values: AttributeValues::Vector3(ZeroVec::alloc_from_slice(&face_normals)),
        })
        .unwrap();
    assert!(data
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Normal,
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector3(vec![[1.; 3]; 3].into_zero_vec()),
        })
        .is_err());
    assert!(data
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Tangent,
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector3(vec![[1.; 3]; 3].into_zero_vec()),
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
    assert_eq!(data.normals().unwrap().to_vec(), normals);
    assert_eq!(data.colors().unwrap().to_vec(), colors);
    assert!(data.texture_coordinates()[0].is_empty());
    assert_eq!(data.texture_coordinates()[1].to_vec(), uv);
    data.validate().unwrap();
    let edited = data
        .edited(GeometryEdit {
            normals: Some(ZeroVec::default()),
            ..Default::default()
        })
        .unwrap();
    assert!(edited.normals().is_none());
    assert_eq!(
        edited
            .attributes
            .vector3(&AttributeSemantic::Normal, AttributeDomain::Face)
            .unwrap()
            .to_vec(),
        face_normals
    );
    data.attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Tangent,
            domain: AttributeDomain::Pool,
            values: AttributeValues::Vector4(ZeroVec::alloc_from_slice(&pool)),
        })
        .unwrap();
    data.validate().unwrap();
    assert_eq!(data.tangent_pool().unwrap().to_vec(), pool);
    assert!(data.tangents().is_none());
    assert!(!data.edit_capabilities().vertex_remap);
    assert_eq!(data.kind(), GeometryKind::Display);
    let mut collection = GeometryAttributes::default();
    collection
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(2),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector2(ZeroVec::alloc_from_slice(&uv)),
        })
        .unwrap();
    assert_eq!(collection.texture_coordinates().len(), 3);
    assert!(collection.texture_coordinates()[0].is_empty());
    assert!(collection.texture_coordinates()[1].is_empty());
    assert_eq!(collection.texture_coordinates()[2].to_vec(), uv);
}
