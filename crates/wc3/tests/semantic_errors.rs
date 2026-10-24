//! Structured semantic diagnostics retain actual identities and extension causes.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;
use wc3::model::error::{ErrorKind, Location, Owner, ReferenceError, Segment};
use wc3::model::profiles::Mdlx;
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
