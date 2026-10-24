use std::borrow::Cow;
use wc3::model::common::IntoPackedArray;
use wc3::model::error::{ErrorKind, GeometryError, PolygonBoundaryRule, Segment};
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Bounds, ConnectivitySets,
    DeformableGeometry, GeometryAttributes, GeometryData, GeometrySchema, Mat4, Topology, Vec3,
};

#[test]
fn stream_count_retains_semantic_and_exact_lengths() {
    let mut geometry =
        GeometryData::triangles(vec![Vec3::ZERO, Vec3::X, Vec3::Y], vec![[0, 1, 2]]).unwrap();
    geometry.set_normals(vec![Vec3::Z].into_packed_array());
    let error = geometry.validate().unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::CountMismatch {
            expected: 3,
            actual: 1
        })
    ));
    assert!(matches!(
        error.location().segments(),
        [Segment::Attribute {
            semantic: AttributeSemantic::Normal,
            domain: AttributeDomain::Vertex
        }]
    ));
}

#[test]
fn topology_retains_rejected_index_and_polygon_boundary() {
    let error = GeometryData::triangles(vec![Vec3::ZERO], vec![[0, 1, 0]]).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::IndexOutOfBounds { index: 1, len: 1 })
    ));
    assert!(matches!(
        error.location().segments().last(),
        Some(Segment::Index(1))
    ));
    let geometry = GeometryData {
        positions: vec![Vec3::ZERO; 3].into_packed_array(),
        connectivity: ConnectivitySets::new(
            Topology::Polygons {
                corners: vec![0, 1, 2].into_packed_array(),
                offsets: vec![1, 3].into_packed_array(),
            },
            vec![],
        ),
        ..GeometryData::default()
    };
    assert!(matches!(
        geometry.validate().unwrap_err().kind(),
        ErrorKind::Geometry(GeometryError::PolygonBoundary {
            rule: PolygonBoundaryRule::StartsAtZero,
            actual: Some(1),
            corners: 3,
            ..
        })
    ));
}

#[test]
fn attribute_error_owns_custom_identity_after_source_is_dropped() {
    let error = {
        let name = String::from("authored light mask");
        let mut attributes = GeometryAttributes::default();
        attributes
            .insert(Attribute {
                semantic: AttributeSemantic::Named(Cow::Borrowed(&name)),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Scalar(vec![1.].into_packed_array()),
            })
            .unwrap();
        attributes
            .insert(Attribute {
                semantic: AttributeSemantic::Named(Cow::Borrowed(&name)),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Scalar(vec![1.].into_packed_array()),
            })
            .unwrap_err()
    };
    assert_owned_attribute(error);
}

fn assert_owned_attribute(error: wc3::model::Error) {
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::DuplicateAttribute)
    ));
    assert!(
        matches!(error.location().segments(), [Segment::Attribute { semantic: AttributeSemantic::Named(Cow::Owned(name)), domain: AttributeDomain::Vertex }] if name == "authored light mask")
    );
}

#[test]
fn deformation_reports_nonfinite_vertex_without_mutating_streams() {
    let mut positions = [Vec3::X, Vec3::Y];
    let original = positions;
    let mut transforms = [Mat4::IDENTITY; 2];
    transforms[1].w_axis.x = f32::INFINITY;
    let error = DeformableGeometry {
        positions: &mut positions,
        normals: &mut [],
        tangents: &mut [],
    }
    .deform(&transforms)
    .unwrap_err();
    assert!(
        matches!(error.kind(), ErrorKind::Geometry(GeometryError::NonFinite { value }) if value.is_infinite())
    );
    assert!(
        matches!(error.location().segments(), [Segment::Field(field), Segment::Index(1), Segment::Index(0)] if field == "positions")
    );
    assert_eq!(positions, original);
    assert!(
        matches!(Bounds::from_positions(&[Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX)]).unwrap_err().kind(), ErrorKind::Geometry(GeometryError::RadiusOverflow { radius }) if radius.is_finite())
    );
}
