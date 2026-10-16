//! Model mounting shared by authored resources and application composition.
use crate::model::{Error, FeatureId, Link, NodeId, Transform, ValidateLinks};

/// Frame followed by an independently instantiated model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountTarget {
    /// Follow a hierarchy node directly.
    Node(NodeId),
    /// Follow an exposed socket, including its authored activation.
    Socket(FeatureId),
}

/// Interpretation of a child model's axes and numerical units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoordinatePolicy {
    /// Retain the carrying model's authored axes and units.
    #[default]
    Host,
    /// Retain the child's standalone axes and physical units relative to the host frame.
    Child,
}

/// Placement of a child model, independent of resource loading and playback policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelMount {
    /// Node or exposed socket in the carrying model.
    pub target: MountTarget,
    /// Local offset in the target frame, composed before the child coordinate conversion.
    pub transform: Transform,
    /// Coordinate and unit interpretation of the child.
    pub coordinates: CoordinatePolicy,
}
impl ModelMount {
    /// Check the local transform before interpreting its rotation or pivot.
    pub fn validate_transform(&self) -> Result<(), Error> {
        self.transform.validate()
    }

    /// Place a child at a target using its carrying model's authored axes and units.
    pub fn new(target: MountTarget) -> Self {
        Self {
            target,
            transform: Transform::default(),
            coordinates: CoordinatePolicy::Host,
        }
    }
}
impl ValidateLinks for MountTarget {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(match *self {
            Self::Node(node) => Link::Node(node),
            Self::Socket(point) => Link::Feature(point),
        });
    }
}
impl ValidateLinks for ModelMount {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.target.visit_links(visitor);
        if self.transform.validate().is_err() {
            visitor(Link::InvalidValue);
        }
    }
}
