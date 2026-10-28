//! Projection preserves authored correspondence while staging large primitive selections.
use glam::{Vec2, Vec3, Vec4};
use std::cell::Cell;
use wc3::model::error::{ErrorKind, GeometryError, Location, TriangulationFailure};
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivitySets, DrawSubset,
    GeometryData, IntoPackedArray, Mesh, PrimitiveKind, PrimitiveRange, PrimitiveSelection,
    SourceCorner, Topology,
};

fn mesh(positions: impl IntoIterator<Item = Vec3>, topology: Topology<'static>) -> Mesh<'static> {
    Mesh::new(GeometryData {
        positions: positions.into_iter().collect(),
        connectivity: ConnectivitySets::new(topology, Vec::new()),
        ..Default::default()
    })
}
fn selection(mesh: &Mesh<'_>, range: PrimitiveRange) -> DrawSubset {
    DrawSubset::new(PrimitiveSelection {
        connectivity: mesh.subsets.primary(),
        range,
    })
}
fn corner(vertex: usize, face: usize, corner: usize) -> SourceCorner {
    SourceCorner {
        vertex,
        face,
        corner,
    }
}

#[test]
fn selected_nested_groups_coalesce_only_adjacent_primitive_families() {
    let mesh = mesh(
        [Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::ONE],
        Topology::Mixed(vec![
            Topology::Points([0].into_packed_array()),
            Topology::Mixed(vec![
                Topology::Triangles([[0, 1, 2], [2, 1, 3]].into_packed_array()),
                Topology::Triangles([[0, 2, 3]].into_packed_array()),
                Topology::Lines([[0, 3]].into_packed_array()),
                Topology::Triangles([[1, 2, 3]].into_packed_array()),
            ]),
            Topology::Points([3].into_packed_array()),
        ]),
    );
    let subset = selection(&mesh, PrimitiveRange::Primitives { first: 2, count: 4 });
    let batches = mesh.project(&subset, 0, [None; 3]).unwrap();
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].kind, PrimitiveKind::Triangles);
    assert_eq!(
        batches[0].sources,
        [
            corner(2, 2, 4),
            corner(1, 2, 5),
            corner(3, 2, 6),
            corner(0, 3, 7),
            corner(2, 3, 8),
            corner(3, 3, 9)
        ]
    );
    assert_eq!(batches[1].kind, PrimitiveKind::Lines);
    assert_eq!(batches[1].sources, [corner(0, 4, 10), corner(3, 4, 11)]);
    assert_eq!(batches[2].kind, PrimitiveKind::Triangles);
    assert_eq!(
        batches[2].sources,
        [corner(1, 5, 12), corner(2, 5, 13), corner(3, 5, 14)]
    );
}

#[test]
fn odd_strip_restart_swaps_complete_source_corners_and_their_attributes() {
    let mut mesh = mesh(
        [Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::ONE, Vec3::Y * 2.],
        Topology::TriangleStrip([0, 1, 2, 3, 4].into_packed_array()),
    );
    mesh.geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector2(
                (0..9).map(|i| Vec2::new(i as f32, -(i as f32))).collect(),
            ),
        })
        .unwrap();
    let ordinary = mesh
        .project(
            &selection(&mesh, PrimitiveRange::Primitives { first: 1, count: 2 }),
            0,
            [None; 3],
        )
        .unwrap();
    assert_eq!(
        ordinary[0].sources,
        [
            corner(2, 1, 3),
            corner(1, 1, 4),
            corner(3, 1, 5),
            corner(2, 2, 6),
            corner(3, 2, 7),
            corner(4, 2, 8)
        ]
    );
    let restarted = mesh
        .project(
            &selection(&mesh, PrimitiveRange::StripIndices { first: 1, count: 4 }),
            0,
            [None; 3],
        )
        .unwrap();
    let sources = [
        corner(1, 1, 4),
        corner(2, 1, 3),
        corner(3, 1, 5),
        corner(3, 2, 7),
        corner(2, 2, 6),
        corner(4, 2, 8),
    ];
    assert_eq!(restarted[0].sources, sources);
    assert_eq!(
        restarted[0].texture_coordinates[0],
        sources.map(|source| Vec2::new(source.corner as f32, -(source.corner as f32)))
    );
    assert_eq!(
        restarted[0].positions,
        sources.map(|source| mesh.positions().get(source.vertex).unwrap())
    );
    let empty = mesh
        .project(
            &selection(&mesh, PrimitiveRange::StripIndices { first: 3, count: 2 }),
            0,
            [None; 3],
        )
        .unwrap();
    assert!(empty.is_empty());
}

#[test]
fn concave_polygon_triangulation_retains_corner_attributes_and_interior_area() {
    let positions = [
        Vec3::ZERO,
        Vec3::new(2., 0., 0.),
        Vec3::new(2., 2., 0.),
        Vec3::new(1., 1., 0.),
        Vec3::new(0., 2., 0.),
    ];
    let mut mesh = mesh(
        positions,
        Topology::Polygons {
            corners: [0, 1, 2, 3, 4].into_packed_array(),
            offsets: [0, 5].into_packed_array(),
        },
    );
    let custom = AttributeSemantic::Named("authored corner".into());
    for attribute in [
        Attribute {
            semantic: AttributeSemantic::TexCoord(0),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector2((0..5).map(|i| Vec2::splat(i as f32)).collect()),
        },
        Attribute {
            semantic: custom.clone(),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Unsigned([10, 20, 30, 40, 50].into_packed_array()),
        },
    ] {
        mesh.geometry.attributes.insert(attribute).unwrap();
    }
    let subset = mesh.whole_geometry().unwrap();
    let batches = mesh.project(&subset, 0, [None; 3]).unwrap();
    assert_eq!(batches.len(), 1);
    let batch = &batches[0];
    assert_eq!(batch.sources.len(), 9);
    let area: f32 = batch
        .positions
        .as_chunks::<3>()
        .0
        .iter()
        .map(|triangle| {
            let signed_twice_area = (triangle[1] - triangle[0])
                .cross(triangle[2] - triangle[0])
                .z;
            assert!(signed_twice_area > 0.);
            signed_twice_area * 0.5
        })
        .sum();
    assert_eq!(area, 3.);
    let AttributeValues::Unsigned(values) =
        batch.attribute(&mesh, &subset, &custom).unwrap().unwrap()
    else {
        panic!("corner attribute lost its type")
    };
    for (i, source) in batch.sources.iter().enumerate() {
        assert_eq!(source.face, 0);
        assert_eq!(source.vertex, source.corner);
        assert_eq!(batch.positions[i], positions[source.vertex]);
        assert_eq!(
            batch.texture_coordinates[0][i],
            Vec2::splat(source.corner as f32)
        );
        assert_eq!(values.get(i), Some((source.corner as u32 + 1) * 10));
        assert_eq!(batch.normals[i], Vec3::Z);
    }
}

#[test]
fn selected_indices_are_checked_before_triangulation_or_color_callbacks() {
    let mut mesh = mesh(
        [Vec3::ZERO, Vec3::X, Vec3::X * 2., Vec3::X * 3.],
        Topology::Mixed(vec![
            Topology::Polygons {
                corners: [0, 1, 2, 3].into_packed_array(),
                offsets: [0, 4].into_packed_array(),
            },
            Topology::Points([99].into_packed_array()),
        ]),
    );
    mesh.geometry.set_colors([Vec4::ONE; 4].into_packed_array());
    let subset = mesh.whole_geometry().unwrap();
    let calls = Cell::new(0);
    let error = mesh
        .project_with_color(&subset, 0, [None; 3], |_, value| {
            calls.set(calls.get() + 1);
            value
        })
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::IndexOutOfBounds { index: 99, len: 4 })
    ));
    assert_eq!(
        error.location(),
        &Location::for_owner(mesh.subsets.primary())
            .field("corners")
            .index(4)
    );
    assert_eq!(calls.get(), 0);
    let polygon = selection(&mesh, PrimitiveRange::Primitives { first: 0, count: 1 });
    let error = mesh.project(&polygon, 0, [None; 3]).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::Triangulation {
            reason: TriangulationFailure::ZeroArea,
            corners: 4
        })
    ));
}

#[test]
fn later_triangulation_failure_precedes_earlier_batch_color_callbacks() {
    let mut mesh = mesh(
        [Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::X * 2., Vec3::X * 3.],
        Topology::Mixed(vec![
            Topology::Points([2].into_packed_array()),
            Topology::Polygons {
                corners: [0, 1, 3, 4].into_packed_array(),
                offsets: [0, 4].into_packed_array(),
            },
        ]),
    );
    mesh.geometry.set_colors([Vec4::ONE; 5].into_packed_array());
    let calls = Cell::new(0);
    let error = mesh
        .project_with_color(&mesh.whole_geometry().unwrap(), 0, [None; 3], |_, color| {
            calls.set(calls.get() + 1);
            color
        })
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::Triangulation {
            reason: TriangulationFailure::ZeroArea,
            corners: 4
        })
    ));
    assert_eq!(
        error.location(),
        &Location::new().field("topology").index(1)
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn empty_polygon_rows_retain_their_place_around_nonempty_batches() {
    let mesh = mesh(
        [Vec3::ZERO, Vec3::X, Vec3::Y],
        Topology::Polygons {
            corners: [0, 1, 2].into_packed_array(),
            offsets: [0, 0, 3, 3].into_packed_array(),
        },
    );
    let batches = mesh
        .project(&mesh.whole_geometry().unwrap(), 0, [None; 3])
        .unwrap();
    assert_eq!(batches.len(), 3);
    assert!(batches
        .iter()
        .all(|batch| batch.kind == PrimitiveKind::Triangles));
    assert!(batches[0].sources.is_empty());
    assert_eq!(
        batches[1].sources,
        [corner(0, 1, 0), corner(1, 1, 1), corner(2, 1, 2)]
    );
    assert!(batches[2].sources.is_empty());
}
