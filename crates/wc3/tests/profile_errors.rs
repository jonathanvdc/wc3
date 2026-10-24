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

#[test]
fn feature_narrowing_retains_family_without_retaining_rejected_payload() {
    use wc3::model::error::FeatureFamily;
    use wc3::model::{Camera, Construct, NodeCamera, NodeFeature, PropertyValues};
    let mut values = PropertyValues::default();
    let feature = NodeFeature::Camera(Camera::Node(Box::new(NodeCamera::construct(&mut values))));
    let error = <Wmo as Narrows<Universal>>::feature(feature, &values).unwrap_err();
    drop(values);
    assert!(matches!(
        error.kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::Wmo,
            violation: ProfileViolation::UnsupportedFeature {
                family: FeatureFamily::Camera
            }
        })
    ));
}

#[test]
fn material_projection_reports_step_count_and_target() {
    use wc3::model::profiles::wmo;
    use wc3::model::MaterialSurface;
    let error = wmo::MaterialStep::try_from(MaterialSurface::default()).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::Wmo,
            violation: ProfileViolation::MaterialStepCount {
                expected: 1,
                actual: 0
            }
        })
    ));
}

#[test]
fn world_group_bounds_retain_group_component_and_numeric_evidence() {
    use wc3::model::error::{GeometryError, Segment};
    use wc3::model::profiles::wmo::{Group, WorldSystem};
    let world = WorldSystem {
        groups: vec![Group {
            bounds: [Vec3::X, Vec3::ZERO],
            ..Group::default()
        }],
        ..WorldSystem::default()
    };
    let error = world.validate().unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Geometry(GeometryError::InvertedBounds {
            minimum: 1.,
            maximum: 0.
        })
    ));
    assert!(
        matches!(error.location().segments(), [Segment::Field(groups), Segment::Index(0), Segment::Field(bounds), Segment::Index(0), Segment::Index(0)] if groups == "groups" && bounds == "bounds")
    );
}
