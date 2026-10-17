//! Layered-model ownership, sequence bounds, and hierarchy controls.
use crate::model::{Bounds, ClipId, FeatureId, ImageResourceId, Link, MeshId, Time, ValidateLinks};
/// Texture resource authoring controls beyond the common sampling description.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureControls {
    /// Authored filename retained even when a replaceable category drives sampling.
    pub fallback_path: Option<ImageResourceId>,
    /// Texture behavior bits with unresolved meanings.
    pub unclassified: TextureFlags,
}
use crate::model::values::layered::{NodeFlags, SequenceFlags, TextureFlags};
use std::borrow::Cow;

/// Authored sequence controls beyond common animation timing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sequence {
    /// Start of this playback interval on the authored model timeline.
    pub start: Time,
    /// Movement speed used by the game animation controller.
    pub move_speed: f32,
    /// Selection rarity within a sequence family.
    pub rarity: f32,
    /// Animation synchronization point.
    pub sync_point: u32,
    /// Unresolved sequence behavior bits.
    pub unclassified: SequenceFlags,
    /// Authored bounding sphere radius.
    pub radius: f32,
    /// Authored bounding box minimum and maximum.
    pub bounds: [[f32; 3]; 2],
}
/// Mesh-level Warcraft III controls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    /// Include this mesh in glider behavior.
    pub glider: bool,
    /// Disable picking for this mesh.
    pub unselectable: bool,
    /// Sequence-specific authored mesh bounds.
    pub sequence_bounds: Vec<(ClipId, [[f32; 3]; 2], f32)>,
    /// Additional authored bounding volumes with no declared sequence association.
    pub unassigned_bounds: Vec<Bounds>,
}
/// Additional Warcraft III bone controls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bone {
    /// Restrict a bone to one mesh; none applies to all meshes.
    pub mesh: Option<MeshId>,
    /// Individual geometry-animation feature controlling this bone.
    pub animation: Option<FeatureId>,
}
/// Warcraft III authoring controls attached directly to one hierarchy node.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeControls {
    /// Native flag bits not mapped to explicit semantic fields.
    pub unmapped_flags: NodeFlags,
    /// Axis-lock declarations masked by a full spherical billboard.
    /// These flags remain inactive unless the semantic billboard is spherical;
    /// changing it to another mode does not reactivate the retained declarations.
    pub spherical_axis_locks: BillboardAxisLocks,
    /// Optional deformation-joint gate controls.
    pub bone: Option<Bone>,
    /// Optional authored bind-pose matrix, without an inferred matrix convention.
    pub bind_pose: Option<[f32; 12]>,
}
/// Independently authored native axis-lock declarations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BillboardAxisLocks {
    /// Native X-axis lock declaration.
    pub x: bool,
    /// Native Y-axis lock declaration.
    pub y: bool,
    /// Native Z-axis lock declaration.
    pub z: bool,
}
/// Genuinely model-wide Warcraft III authoring controls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelMetadata<'a> {
    /// External animation-file resource.
    pub animation_file: Cow<'a, str>,
    /// Sequence transition blend duration in seconds.
    pub blend_time: f64,
    /// Named external face-animation resources.
    pub face_effects: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}
impl ValidateLinks for ModelMetadata<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.blend_time.is_finite() || self.blend_time < 0. {
            visitor(Link::InvalidValue);
        }
    }
}
