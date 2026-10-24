use wc3::model::common::IntoPackedArray;
use wc3::model::error::{
    ErrorKind, PayloadKind, PayloadVocabulary, ProfileError, ProfileKind, ProfileViolation,
};
use wc3::model::profiles::{Mdlx, Narrows, Universal, Wmo};
use wc3::model::{GeometryData, GeometryKind, IdentityBinding, M2Geometry, UniversalBinding, Vec3};

#[test]
fn narrowing_retains_target_and_required_owner_payload() {
    let error = <Mdlx as Narrows<Universal>>::node(None).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::Mdlx,
            violation: ProfileViolation::RequiredPayload {
                owner: PayloadKind::NodeProperties
            }
        })
    ));
    let error = <Wmo as Narrows<Universal>>::binding(UniversalBinding::Identity(IdentityBinding))
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::Wmo,
            violation: ProfileViolation::UnsupportedPayload {
                owner: PayloadKind::Binding,
                actual: PayloadVocabulary::IdentityBinding
            }
        })
    ));
}

#[test]
fn geometry_projection_retains_target_and_rejected_purpose() {
    let geometry = GeometryData {
        kind: GeometryKind::Collision,
        positions: vec![Vec3::ZERO].into_packed_array(),
        ..GeometryData::default()
    };
    assert!(matches!(
        M2Geometry::try_from(geometry).unwrap_err().kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::M2,
            violation: ProfileViolation::GeometryKind {
                actual: GeometryKind::Collision
            }
        })
    ));
}
