//! Structured semantic diagnostics retain actual identities and extension causes.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;
use wc3::model::error::{EditError, ErrorKind, Location, Owner, ReferenceError, Segment};
use wc3::model::profiles::{Mdlx, Profile};
use wc3::model::{AttributeDomain, AttributeSemantic, Error, Model};

#[derive(Debug)]
struct ApplicationConstraint {
    code: u32,
    source: io::Error,
}
impl Display for ApplicationConstraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "application constraint {}", self.code)
    }
}
impl StdError for ApplicationConstraint {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}
#[derive(Clone, Debug)]
struct ApplicationProfile;

macro_rules! mdlx_payloads {
    ($($payload:ident),* $(,)?) => {$(
        type $payload<'a> = <Mdlx as Profile>::$payload<'a>;
    )*};
}
impl Profile for ApplicationProfile {
    mdlx_payloads!(
        ImageResource,
        ModelResource,
        EffectResource,
        PhysicsResource,
        BoneOverrideResource,
        Appearance,
        Binding,
        Geometry,
        ModelMetadata,
        EventSource,
        System,
        Surface,
        Feature,
        NodeProperties,
        MeshProperties,
        InstanceProperties,
        ClipProperties,
    );
    type DrawProperties = <Mdlx as Profile>::DrawProperties;

    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        if let Some((id, _)) = model
            .nodes()
            .iter()
            .find(|(_, node)| node.name == "Rejected")
        {
            return Err(Error::extension(
                Location::for_owner(id).field("name"),
                ApplicationConstraint {
                    code: 81,
                    source: io::Error::new(io::ErrorKind::InvalidData, "reserved application name"),
                },
            ));
        }
        Ok(())
    }
}

#[test]
fn custom_profile_publication_failure_retains_its_typed_source_location_and_rollback() {
    let mut builder = Model::<ApplicationProfile>::builder("Example");
    let node = builder.add_named_node("Original").unwrap();
    let mut model = builder.finish().unwrap();
    let result = model.try_edit(|draft| {
        let mut definition = draft.model().node(node).unwrap().clone();
        definition.name = "Rejected".into();
        draft.set_node(node, definition)
    });

    assert_eq!(model.node(node).unwrap().name, "Original");
    let error = result.unwrap_err();
    let EditError::Validation(semantic) = &error else {
        panic!("the application operation completed before its profile rejected publication")
    };
    assert!(matches!(semantic.kind(), ErrorKind::Extension(_)));
    assert!(matches!(semantic.location().segments(),
        [Segment::Owner(Owner::Node(id)), Segment::Field(field)]
        if *id == node && field == "name"));
    assert!(std::ptr::eq(
        error.source().unwrap().downcast_ref::<Error>().unwrap(),
        semantic,
    ));
    let cause = semantic
        .source()
        .unwrap()
        .downcast_ref::<ApplicationConstraint>()
        .unwrap();
    assert_eq!(cause.code, 81);
    assert_eq!(
        cause
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn structured_locations_retain_owner_fields_indices_and_attribute_identity() {
    let mut builder = Model::<Mdlx>::builder("Example");
    let node = builder.add_named_node("Root").unwrap();
    let location = Location::new()
        .field("geometry")
        .field("attributes")
        .index(3)
        .attribute(AttributeSemantic::TexCoord(2), AttributeDomain::Vertex)
        .with_owner(node);
    assert!(
        matches!(location.segments(), [Segment::Owner(Owner::Node(id)), Segment::Field(geometry),
        Segment::Field(attributes), Segment::Index(3),
        Segment::Attribute { semantic: AttributeSemantic::TexCoord(2), domain: AttributeDomain::Vertex }]
        if *id == node && geometry == "geometry" && attributes == "attributes")
    );
    let error = Error::at(
        location,
        ReferenceError::Unknown {
            target: node.into(),
        },
    );
    assert!(
        matches!(error.kind(), ErrorKind::Reference(ReferenceError::Unknown { target: Owner::Node(id) }) if *id == node)
    );
}
#[test]
fn cloned_extension_errors_share_original_typed_cause_and_preserve_its_chain() {
    let error = Error::extension(
        Location::new().field("custom"),
        ApplicationConstraint {
            code: 42,
            source: io::Error::new(io::ErrorKind::InvalidData, "invalid application record"),
        },
    );
    let cloned = error.clone().with_context(Location::new().field("profile"));
    let original = error.source().unwrap();
    let shared = cloned.source().unwrap();
    assert!(std::ptr::eq(original, shared));
    assert_eq!(
        shared.downcast_ref::<ApplicationConstraint>().unwrap().code,
        42
    );
    assert_eq!(
        shared
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert!(
        matches!(cloned.location().segments(), [Segment::Field(profile), Segment::Field(custom)]
        if profile == "profile" && custom == "custom")
    );
    assert!(matches!(cloned.kind(), ErrorKind::Extension(_)));
}

#[test]
fn application_edit_failure_preserves_its_type_and_rolls_back_the_draft() {
    let mut builder = Model::<Mdlx>::builder("Example");
    let node = builder.add_named_node("Original").unwrap();
    let mut model = builder.finish().unwrap();
    let result = model.try_edit(|draft| {
        let mut definition = draft.model().node(node).unwrap().clone();
        definition.name = "Changed".into();
        draft.set_node(node, definition).unwrap();
        Err::<(), _>(ApplicationConstraint {
            code: 7,
            source: io::Error::new(io::ErrorKind::Interrupted, "cancelled"),
        })
    });
    assert_eq!(model.node(node).unwrap().name, "Original");
    let EditError::Operation(cause) = result.unwrap_err() else {
        panic!("application cancellation must retain its own phase")
    };
    assert_eq!(cause.code, 7);
}

#[test]
fn publication_failure_retains_semantic_cause_and_rolls_back_the_draft() {
    let mut builder = Model::<Mdlx>::builder("Example");
    let node = builder.add_named_node("Original").unwrap();
    let mut model = builder.finish().unwrap();
    let result = model.try_edit(|draft| {
        draft.set_node_parent(node, Some(node))?;
        Ok::<_, Error>(42)
    });
    assert_eq!(model.node(node).unwrap().parent, None);
    let EditError::Validation(error) = result.unwrap_err() else {
        panic!("completed invalid edits must fail publication")
    };
    assert!(
        matches!(error.kind(),ErrorKind::Reference(ReferenceError::Cycle {target:Owner::Node(id)}) if *id==node)
    );
    assert!(
        matches!(error.location().segments(),[Segment::Owner(Owner::Node(id)),Segment::Field(field)] if *id==node && field=="parent")
    );
}

#[test]
fn successful_try_edit_returns_the_operation_result_after_publication() {
    let mut model = Model::<Mdlx>::builder("Example").finish().unwrap();
    let result = model
        .try_edit(|draft| draft.add_named_node("Published"))
        .unwrap();
    assert_eq!(model.node(result).unwrap().name, "Published");
}

#[test]
fn rest_constant_conflicts_retain_slot_and_both_typed_values() {
    use wc3::model::error::ConstantValue;
    use wc3::model::PropertyValues;
    let mut values = PropertyValues::default();
    let slot = values.add(1f32);
    let mut different = values.clone();
    different.set(slot, 2.).unwrap();
    let mut builder = Model::<Mdlx>::builder("Example");
    builder.import_values(&values).unwrap();
    let error = builder.import_values(&different).unwrap_err();
    assert!(
        matches!(error.location().segments(),[Segment::Owner(Owner::Value(id))] if *id==slot.slot())
    );
    assert!(matches!(
        error.kind(),
        ErrorKind::Reference(ReferenceError::ConflictingConstants {
            existing: ConstantValue::Scalar(1.),
            incoming: ConstantValue::Scalar(2.)
        })
    ));
}

#[test]
fn resource_failures_distinguish_address_form_from_authored_filename_loss() {
    use wc3::model::error::{
        PayloadKind, ProfileKind, ProfileViolation, ReferenceError, ResourceAddressKind,
        ResourceVocabulary,
    };
    use wc3::model::{ImageResource, MdlxImageResource, PathResource, WorldImageResource};
    let error = PathResource::try_from(ImageResource::Root).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Reference(ReferenceError::ResourceAddress {
            vocabulary: ResourceVocabulary::Path,
            actual: ResourceAddressKind::Root
        })
    ));
    let error = MdlxImageResource::try_from(ImageResource::FileDataId {
        id: 7,
        filename: [].as_slice().into(),
    })
    .unwrap_err();
    assert!(
        matches!(error.kind(),ErrorKind::Profile(cause) if cause.target==ProfileKind::Mdlx && matches!(cause.violation,ProfileViolation::UnsupportedResourceAddress {owner:PayloadKind::ImageResource,actual:ResourceAddressKind::FileDataId}))
    );
    let error = WorldImageResource::try_from(ImageResource::FileDataId {
        id: 7,
        filename: [0xff, 1].as_slice().into(),
    })
    .unwrap_err();
    assert!(
        matches!(error.kind(),ErrorKind::Reference(ReferenceError::FilenameNotRepresentable {filename}) if filename==&[0xff,1])
    );
}

#[test]
fn bind_matrix_failures_retain_cardinality_or_the_actual_nonfinite_matrix() {
    use wc3::model::error::TransformError;
    use wc3::model::{BindingSchema, ExplicitBinding, Mat4};
    let binding = ExplicitBinding::new([Mat4::IDENTITY]).unwrap();
    let error = binding.validate(2).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Transform(TransformError::BindMatrixCount {
            expected: 2,
            actual: 1
        })
    ));
    let error =
        ExplicitBinding::new([Mat4::IDENTITY, Mat4::from_cols_array(&[f32::NAN; 16])]).unwrap_err();
    assert!(
        matches!(error.location().segments(),[Segment::Field(binding),Segment::Field(matrices),Segment::Index(1)] if binding=="binding" && matrices=="matrices")
    );
    assert!(
        matches!(error.kind(),ErrorKind::Transform(TransformError::NonFiniteMatrix {matrix}) if !matrix.is_finite())
    );
}

#[test]
fn portable_collision_errors_retain_triangle_corner_and_preserve_failed_add_rollback() {
    use wc3::model::common::IntoZeroVec;
    use wc3::model::error::PhysicsError;
    use wc3::model::scene::{ContactMaterial, Geometry, PhysicsSystem, Shape};
    use wc3::model::{Mat4, PackedArray};
    let mut system = PhysicsSystem::default();
    let error = system
        .add_shape(Shape {
            geometry: Geometry::Triangles {
                positions: PackedArray::from_storage(vec![[0f32; 3]].into_zero_vec()),
                triangles: PackedArray::from_storage(vec![[0u32, 2, 0]].into_zero_vec()),
            },
            transform: Mat4::IDENTITY,
            material: ContactMaterial::default(),
            callback_categories: 0,
        })
        .unwrap_err();
    assert_eq!(system.shapes().len(), 0);
    assert!(matches!(
        error.kind(),
        ErrorKind::Physics(PhysicsError::TriangleVertexOutOfBounds {
            vertex: 2,
            positions: 1
        })
    ));
    assert!(
        matches!(error.location().segments(),[Segment::Owner(Owner::Shape(_)),Segment::Field(geometry),Segment::Field(triangles),Segment::Index(0),Segment::Index(1)] if geometry=="geometry" && triangles=="triangles")
    );
}

#[test]
fn graph_timing_failures_retain_clip_or_clock_identity_and_actual_duration() {
    use wc3::model::error::AnimationError;
    use wc3::model::{Clip, SharedClock, Time};
    let mut builder = Model::<Mdlx>::builder("Example");
    let clip = builder
        .add_clip(Clip::named("invalid").with_duration(Time::seconds(-1.).unwrap()))
        .unwrap();
    let error = builder.finish().unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Animation(AnimationError::NegativeDuration { value: -1. })
    ));
    assert!(
        matches!(error.location().segments(),[Segment::Owner(Owner::Clip(id)),Segment::Field(field)] if *id==clip && field=="duration")
    );
    let mut builder = Model::<Mdlx>::builder("Example");
    let clock = builder
        .add_clock(SharedClock::Repeating {
            period: Time::default(),
        })
        .unwrap();
    let error = builder.finish().unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Animation(AnimationError::NonPositivePeriod { value: 0. })
    ));
    assert!(
        matches!(error.location().segments(),[Segment::Owner(Owner::Clock(id)),Segment::Field(field)] if *id==clock && field=="period")
    );
}
