use wc3::model::common::IntoPackedArray;
use wc3::model::error::{
    ErrorKind, PayloadForm, PayloadKind, ProfileError, ProfileKind, ProfileViolation,
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
                actual: PayloadForm::IdentityBinding
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

#[test]
fn nested_topology_retains_the_rejected_child_kind_and_index() {
    use wc3::model::error::{Segment, TopologyKind};
    use wc3::model::{ConnectivitySets, MdlxGeometry, Topology};
    let geometry = GeometryData {
        positions: vec![Vec3::ZERO, Vec3::X].into_packed_array(),
        connectivity: ConnectivitySets::new(
            Topology::Mixed(vec![Topology::Lines(vec![[0, 1]].into_packed_array())]),
            vec![],
        ),
        ..GeometryData::default()
    };
    let error = MdlxGeometry::try_from(geometry).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::Mdlx,
            violation: ProfileViolation::GeometryTopology {
                actual: TopologyKind::Lines
            }
        })
    ));
    assert!(
        matches!(error.location().segments(), [Segment::Field(topology), Segment::Field(groups), Segment::Index(0)] if topology == "topology" && groups == "groups")
    );
}

#[test]
fn empty_explicit_lightmap_retains_its_projection_rule() {
    use wc3::model::error::AttributeProjectionRule;
    use wc3::model::{
        Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometrySchema, WmoGeometry,
    };
    let mut geometry = GeometryData::default();
    geometry
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::LightmapCoordinates,
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Vector2(Vec::new().into_packed_array()),
        })
        .unwrap();
    geometry.validate().unwrap();
    assert!(matches!(
        WmoGeometry::try_from(geometry).unwrap_err().kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::Wmo,
            violation: ProfileViolation::GeometryAttribute {
                rule: AttributeProjectionRule::Nonempty,
                ..
            }
        })
    ));
}

#[test]
fn world_single_vectors_have_only_their_actual_component_index() {
    use wc3::model::error::{GeometryError, Location};
    use wc3::model::profiles::wmo::{Fog, Portal, Volume, WorldSystem};
    use wc3::model::{Vec2, Vec4};
    let world = WorldSystem {
        portals: vec![Portal {
            plane: Vec4::new(0., 0., 1., f32::NAN),
            ..Portal::default()
        }],
        ..WorldSystem::default()
    };
    let error = world.validate().unwrap_err();
    assert!(
        matches!(error.kind(), ErrorKind::Geometry(GeometryError::NonFinite { value }) if value.is_nan())
    );
    assert_eq!(
        error.location(),
        &Location::new()
            .field("portals")
            .index(0)
            .field("plane")
            .index(3)
    );
    for (field, volume) in [
        (
            "center",
            Volume::Sphere {
                center: Vec3::new(0., f32::NAN, 0.),
                radii: Vec2::ONE,
            },
        ),
        (
            "radii",
            Volume::Sphere {
                center: Vec3::ZERO,
                radii: Vec2::new(0., f32::INFINITY),
            },
        ),
    ] {
        let world = WorldSystem {
            fogs: vec![Fog {
                volume,
                above_water: Default::default(),
                underwater: Default::default(),
                doodad_set: None,
                infinite: false,
                controls: Default::default(),
            }],
            ..WorldSystem::default()
        };
        let error = world.validate().unwrap_err();
        assert!(
            matches!(error.kind(), ErrorKind::Geometry(GeometryError::NonFinite { value }) if !value.is_finite())
        );
        assert_eq!(
            error.location(),
            &Location::new()
                .field("fogs")
                .index(0)
                .field("volume")
                .field(field)
                .index(1)
        );
    }
}

#[test]
fn guild_palette_failures_identify_the_actual_rgb_component() {
    use wc3::model::error::Location;
    use wc3::model::profiles::m2::{GuildMaterialStep, MaterialStep, UnresolvedMaterialStep};
    use wc3::model::{Construct, PropertyValues};
    let mut values = PropertyValues::default();
    let mut guild = GuildMaterialStep::construct(&mut values);
    guild.palette.background.y = f32::NAN;
    let mut unresolved = UnresolvedMaterialStep::construct(&mut values);
    unresolved.guild_palette.background.y = f32::INFINITY;
    for (field, step) in [
        ("palette", MaterialStep::Guild(Box::new(guild))),
        ("guild_palette", MaterialStep::Unresolved(unresolved)),
    ] {
        let error = step.validate().unwrap_err();
        assert_eq!(
            error.location(),
            &Location::new().field(field).field("background").index(1)
        );
    }
}
