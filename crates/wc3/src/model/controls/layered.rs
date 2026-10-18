//! Layered-model ownership, sequence bounds, and hierarchy controls.
use super::BillboardFlags;
use crate::model::values::layered::{NodeFlags, SequenceFlags};
use crate::model::{Bounds, ClipId, FeatureId, Link, MeshId, ValidateLinks};
use std::borrow::Cow;

/// Authored Warcraft III extent, including an explicitly empty occupied volume.
/// Empty extents do not define a bounding box or sphere for culling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Extent {
    /// No occupied volume for this authored state.
    Empty,
    /// Finite occupied bounds and their authored sphere radius.
    Bounds(Bounds),
}
impl Default for Extent {
    fn default() -> Self {
        Self::Bounds(Bounds::default())
    }
}
impl ValidateLinks for Extent {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Bounds(bounds) = self {
            if bounds.validate().is_err() {
                visitor(Link::InvalidValue);
            }
        }
    }
}

/// Authored sequence controls beyond common animation timing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sequence {
    /// Movement speed used by the game animation controller.
    pub move_speed: f32,
    /// Selection rarity within a sequence family.
    pub rarity: f32,
    /// Animation synchronization point.
    pub sync_point: u32,
    /// Unresolved sequence behavior bits.
    pub unclassified: SequenceFlags,
    /// Authored occupied bounds, or an explicitly empty extent.
    pub extent: Extent,
}
/// Mesh-level Warcraft III controls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    /// Include this mesh in glider behavior.
    pub glider: bool,
    /// Disable picking for this mesh.
    pub unselectable: bool,
    /// Sequence-specific authored mesh bounds.
    pub sequence_bounds: Vec<(ClipId, Extent)>,
    /// Additional authored bounding volumes with no declared sequence association.
    pub unassigned_bounds: Vec<Extent>,
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
    /// Original native billboard declarations. Export retains them only when their
    /// effective behavior agrees with the node's current semantic billboard.
    pub billboard_flags: BillboardFlags,
    /// Optional deformation-joint gate controls.
    pub bone: Option<Bone>,
    /// Optional authored bind-pose matrix, without an inferred matrix convention.
    pub bind_pose: Option<[f32; 12]>,
}
/// Genuinely model-wide Warcraft III authoring controls.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelMetadata<'a> {
    /// Authored model name.
    pub name: Cow<'a, str>,
    /// Model-info bounding volume; absence means the model-info declaration is absent.
    pub bounds: Option<Bounds>,

    /// External animation-file resource.
    pub animation_file: Cow<'a, str>,
    /// Sequence transition blend duration in seconds.
    pub blend_time: f64,
    /// Named external face-animation resources.
    pub face_effects: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}
impl ValidateLinks for ModelMetadata<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.bounds.is_some_and(|bounds| bounds.validate().is_err()) {
            visitor(Link::InvalidValue);
        }
        if !self.blend_time.is_finite() || self.blend_time < 0. {
            visitor(Link::InvalidValue);
        }
    }
}

impl Default for ModelMetadata<'_> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            bounds: Some(Bounds::default()),
            animation_file: Cow::Borrowed(""),
            blend_time: 0.,
            face_effects: Vec::new(),
        }
    }
}
