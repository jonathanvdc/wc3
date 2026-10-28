//! All geometry inspection paths resolve draw-selected tangent pools consistently.
use glam::{Vec2, Vec3, Vec4};
use wc3::model::error::{ErrorKind, GeometryError, Location};
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, DrawSubset, GeometryData,
    IntoPackedArray, Mesh, ProjectedGeometry, TangentBinding,
};

fn pool_values() -> [Vec4; 5] {
    [Vec4::ZERO, Vec4::ONE, Vec4::X, Vec4::Y, Vec4::Z]
}
fn bound_mesh() -> (Mesh<'static>, DrawSubset) {
    let mut mesh: Mesh<'_> = Mesh::new(
        GeometryData::triangles([Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::ONE], [[3, 1, 2]]).unwrap(),
    );
    mesh.geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Tangent,
            domain: AttributeDomain::Pool,
            values: AttributeValues::Vector4(pool_values().into_packed_array()),
        })
        .unwrap();
    let mut subset = mesh.whole_geometry().unwrap();
    subset.tangents = Some(TangentBinding {
        first_vertex: 1,
        vertex_count: 3,
        first_tangent: 2,
    });
    (mesh, subset)
}
fn gather(batch: &ProjectedGeometry, mesh: &Mesh<'_>, subset: &DrawSubset) -> Vec<Vec4> {
    let AttributeValues::Vector4(values) = batch
        .attribute(mesh, subset, &AttributeSemantic::Tangent)
        .unwrap()
        .unwrap()
    else {
        panic!("tangents lost their vector type")
    };
    values.to_vec()
}
fn batch(mesh: &Mesh<'_>, subset: &DrawSubset) -> ProjectedGeometry {
    mesh.project(subset, 0, [None; 3]).unwrap().remove(0)
}

#[test]
fn bound_pool_gather_matches_projection_with_reordered_vertices_and_nonzero_offsets() {
    let (mesh, subset) = bound_mesh();
    let batch = batch(&mesh, &subset);
    let expected = [Vec4::Z, Vec4::X, Vec4::Y];
    assert_eq!(batch.tangents, expected);
    assert_eq!(gather(&batch, &mesh, &subset), expected);
}

#[test]
fn explicit_pool_binding_wins_over_vertex_tangents_in_both_access_paths() {
    let (mut mesh, subset) = bound_mesh();
    mesh.geometry
        .set_tangents([Vec4::splat(-1.); 4].into_packed_array());
    let batch = batch(&mesh, &subset);
    assert_eq!(batch.tangents, [Vec4::Z, Vec4::X, Vec4::Y]);
    assert_eq!(gather(&batch, &mesh, &subset), batch.tangents);
}

#[test]
fn additional_uv_projection_does_not_hide_an_explicit_authored_tangent_request() {
    let (mut mesh, subset) = bound_mesh();
    mesh.geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::TexCoord(1),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector2([Vec2::ONE; 4].into_packed_array()),
        })
        .unwrap();
    let batch = mesh.project(&subset, 1, [None; 3]).unwrap().remove(0);
    assert!(batch.tangents.is_empty());
    assert_eq!(gather(&batch, &mesh, &subset), [Vec4::Z, Vec4::X, Vec4::Y]);
}

#[test]
fn absent_binding_uses_vertex_tangents_and_keeps_unbound_pool_errors() {
    let (mut mesh, mut subset) = bound_mesh();
    let batch = batch(&mesh, &subset);
    subset.tangents = None;
    for error in [
        mesh.project(&subset, 0, [None; 3]).unwrap_err(),
        batch
            .attribute(&mesh, &subset, &AttributeSemantic::Tangent)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error.kind(),
            ErrorKind::Geometry(GeometryError::MissingPoolBinding)
        ));
        assert_eq!(
            error.location(),
            &Location::new().attribute(AttributeSemantic::Tangent, AttributeDomain::Pool)
        );
    }
    mesh.geometry.set_tangents([Vec4::W; 4].into_packed_array());
    let projected = mesh.project(&subset, 0, [None; 3]).unwrap().remove(0);
    assert_eq!(projected.tangents, [Vec4::W; 3]);
    assert_eq!(gather(&projected, &mesh, &subset), projected.tangents);
}

#[test]
fn bound_pool_failures_retain_the_same_typed_evidence_when_gathered() {
    let (mut mesh, mut subset) = bound_mesh();
    let batch = batch(&mesh, &subset);
    // Source order begins with vertex 3, outside this shortened binding window.
    subset.tangents.as_mut().unwrap().vertex_count = 2;
    for error in [
        mesh.project(&subset, 0, [None; 3]).unwrap_err(),
        batch
            .attribute(&mesh, &subset, &AttributeSemantic::Tangent)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error.kind(),
            ErrorKind::Geometry(GeometryError::TangentVertexWindow {
                vertex: 3,
                first: 1,
                count: 2
            })
        ));
        assert_eq!(error.location(), &Location::new().field("tangents"));
    }
    subset.tangents.as_mut().unwrap().vertex_count = 3;
    // Index 4 is needed for the first source vertex, but this pool ends at 4.
    mesh.geometry
        .attributes
        .set(Attribute {
            semantic: AttributeSemantic::Tangent,
            domain: AttributeDomain::Pool,
            values: AttributeValues::Vector4((&pool_values()[..4]).into_packed_array()),
        })
        .unwrap();
    for error in [
        mesh.project(&subset, 0, [None; 3]).unwrap_err(),
        batch
            .attribute(&mesh, &subset, &AttributeSemantic::Tangent)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error.kind(),
            ErrorKind::Geometry(GeometryError::IndexOutOfBounds { index: 4, len: 4 })
        ));
        assert_eq!(error.location(), &Location::new().field("tangents"));
    }
    mesh.geometry
        .attributes
        .take(&AttributeSemantic::Tangent, AttributeDomain::Pool)
        .unwrap();
    for error in [
        mesh.project(&subset, 0, [None; 3]).unwrap_err(),
        batch
            .attribute(&mesh, &subset, &AttributeSemantic::Tangent)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error.kind(),
            ErrorKind::Geometry(GeometryError::MissingTangentPool)
        ));
        assert_eq!(error.location(), &Location::new().field("tangents"));
    }
}
