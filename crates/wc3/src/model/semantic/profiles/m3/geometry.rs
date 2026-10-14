//! Authored geometry behavior belongs to its mesh and placement.
use super::values::{BatchFlags, RegionFlags};
use crate::model::semantic::{FeatureId, FeatureKind, Link, NodeId, SkinId, ValidateLinks};
/// Region behavior supplementing the shared geometry arrays.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshControls {
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
