//! Programmed-model owner metadata and skeletal property controls.
use crate::model::animation::programmed::{AnimationControl, PropertyBinding};
use crate::model::values::programmed::{BatchFlags, RegionFlags};
use crate::model::values::programmed::{BoneFlags, ModelFlags};
use crate::model::{Bounds, Construct, PropertyValue};
use crate::model::{
    FeatureId, FeatureKind, Link, NodeId, Property as Destination, Quaternion, SkinId,
    ValidateLinks,
};
use std::borrow::Cow;
/// Game-wide model behavior settings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelMetadata<'a> {
    /// Authored model name.
    pub name: Cow<'a, str>,
    /// Authored model bounding volume.
    pub bounds: Bounds,

    /// Authored animation-partition control; its relationship to split definitions is unresolved.
    pub partition_control: u32,
    /// Model behavior flags; unknown bits remain available.
    pub flags: ModelFlags,
}
/// Reset and animation behavior for a value whose initial state belongs to its owner.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PropertyControl<T: PropertyValue> {
    /// Value used when resetting the source animation property.
    pub reset: T,
    /// Authored interpolation, source identity, and sampling controls.
    pub control: AnimationControl,
    /// Typed destination used by logical animation tracks, when bound.
    pub binding: Option<Destination<T>>,
}
/// Source game controls attached directly to an actual skeleton node.
#[derive(Clone, Debug, Construct, PartialEq)]
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
    pub batching: PropertyBinding<u32>,
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

/// Preferred affine packing shared by a region's UV channels.
/// Logical coordinates equal `packed * scale / 32768 + offset`.
/// Export reuses this preference when it represents the current coordinates exactly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UvEncoding {
    /// Finite multiplier; zero encodes a constant coordinate.
    pub scale: f32,
    /// Finite offset applied after scaling.
    pub offset: f32,
}
impl Default for UvEncoding {
    fn default() -> Self {
        Self {
            scale: 16.,
            offset: 0.,
        }
    }
}

/// Region behavior supplementing the shared geometry arrays.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshControls {
    /// Optional preferred UV packing, independent of animated texture transforms.
    pub uv_encoding: Option<UvEncoding>,
    /// Skeleton binding belonging to this region definition.
    pub skin: Option<SkinId>,
    /// Definition-level placement for geometry with no authored draw instances.
    pub undrawn: Option<UndrawnRegion>,
    /// Game-defined region identity independent of graph handles.
    pub source_id: u32,
    /// Region visibility and cloth participation controls.
    pub flags: RegionFlags,
    /// Skeleton root associated with this region, when supplied.
    pub root_bone: Option<NodeId>,
    /// Unresolved authoring control word.
    pub unknown: u32,
    /// Unresolved per-region control byte.
    pub unknown_byte: u8,
}
/// Definition-level detail placement for a region that has no draws.
#[derive(Clone, Debug, PartialEq)]
pub struct UndrawnRegion {
    /// Native detail level containing this definition.
    pub detail_level: u32,
    /// Authored division instancing control.
    pub instance_count: u32,
}
/// Draw behavior supplementing a logical placement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InstanceControls {
    /// Unresolved authored division instancing control.
    pub instance_count: u32,
}
/// A native draw's behavioral controls; geometry and material handles remain common.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatchControls {
    /// Complete draw behavior mask.
    pub flags: BatchFlags,
    /// Native ordering priority.
    pub priority: u16,
    /// Skeleton property controlling whether the draw participates.
    pub batching_bone: Option<NodeId>,
    /// Structured animated bounds controlling this draw.
    pub bounds: Option<FeatureId>,
    /// Unresolved color-control selector; its target namespace is not established.
    pub color_control: u16,
}
impl ValidateLinks for MeshControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self
            .uv_encoding
            .is_some_and(|uv| !uv.scale.is_finite() || !uv.offset.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        self.skin.visit_links(visitor);
        self.root_bone.visit_links(visitor)
    }
}
impl ValidateLinks for InstanceControls {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for BatchControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.batching_bone.visit_links(visitor);
        if let Some(id) = self.bounds {
            visitor(Link::TypedFeature {
                id,
                kind: FeatureKind::M3Extent,
            });
        }
    }
}
