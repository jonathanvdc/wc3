//! Authored model and bone controls, independent of native table storage.
use super::{
    values::{BoneFlags, ModelFlags},
    AnimationControl, Property,
};
use crate::model::semantic::{Property as Destination, Quaternion};
use std::borrow::Cow;

/// Game-wide model behavior settings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelMetadata {
    /// Authored animation-partition control; its relationship to split definitions is unresolved.
    pub partition_control: u32,
    /// Model behavior flags; unknown bits remain available.
    pub flags: ModelFlags,
}
/// Reset and animation behavior for a value whose initial state belongs to its owner.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PropertyControl<T> {
    /// Value used when resetting the source animation property.
    pub reset: T,
    /// Authored interpolation, source identity, and sampling controls.
    pub control: AnimationControl,
    /// Typed destination used by logical animation tracks, when bound.
    pub binding: Option<Destination<T>>,
}
/// Source game controls attached directly to an actual skeleton node.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoneControls {
    /// Authored bone identifier; older unsigned control words fit in the same domain.
    pub source_id: i64,
    /// Bone flags not represented by common inheritance and billboard behavior.
    pub extra_flags: BoneFlags,
    /// Unresolved bone control word adjacent to the parent identity.
    pub reserved: u16,
    /// Translation reset and sampling behavior; initial translation belongs to the node.
    pub position: PropertyControl<[f32; 3]>,
    /// Rotation reset and sampling behavior; initial rotation belongs to the node.
    pub rotation: PropertyControl<Quaternion>,
    /// Scale reset and sampling behavior; initial scale belongs to the node.
    pub scale: PropertyControl<[f32; 3]>,
    /// Animated batch selection control, independent of node visibility.
    pub batching: Property<u32>,
}
/// Named attachment carried by its owning node feature.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attachment<'a> {
    /// Attachment identity used by game content.
    pub name: Cow<'a, str>,
    /// Additional authored attachment control whose meaning remains unresolved.
    pub control: u32,
}

/// Authored role of a node in an M3 scene.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum NodeRole {
    /// The model's implicit, fixed coordinate frame; it has no skeleton bone record.
    #[default]
    ModelSpace,
    /// An actual skeleton bone with its complete game controls.
    Bone(Box<BoneControls>),
}
impl From<BoneControls> for NodeRole {
    fn from(value: BoneControls) -> Self {
        Self::Bone(value.into())
    }
}
